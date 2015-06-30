#![allow(dead_code)]

use self::Operation::*;
use self::BranchCondition::*;
use self::Opcode::*;
use self::VmError::*;
use self::LinkError::*;

type Literal = i32;

/// Position in the data stack
/// Must be an unsigned integer
type StackPos = usize;

/// Position in the vector of bytecode instructions
/// Must be an unsigned integer
type ProgramPos = usize;

/// Jump offset in the vector of bytecode instructions
/// Must be a signed integer
type JumpOffset = i16;

#[derive(Copy, Clone, Debug)]
pub enum Operation {
    Add,
    Mul,
}

#[derive(Copy, Clone, Debug)]
pub enum BranchCondition {
    Equal,
    GreaterThan,
}

#[derive(Copy, Clone, Debug)]
pub enum Opcode<T> {
    Push(Literal),
    Pop,
    Swap(StackPos),
    BinOp(Operation),
    Branch(BranchCondition, JumpOffset),
    Jump(JumpOffset),
    Call(T),
    Return,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VmError {
    StackUnderflow,
    TooFewParameters,
    InvalidStackPosition,
    JumpOffsetTooLarge,
    JumpOffsetTooSmall,
    InvalidFunctionPosition,
    ThereIsNoCode,
    Halted,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LinkError {
    FunctionNameNotFound
}

type VmResult<T> = Result<T, VmError>;
type LinkResult<T> = Result<T, LinkError>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Function {
    start_pos: ProgramPos,
    end_pos: ProgramPos
}

struct FunctionTable {
    table: Vec<(Function, String)>
}

impl FunctionTable {
    fn search_name(&self, name: &str) -> Option<(Function, usize)> {
        let mut i = 0usize;
        for &(f, ref s) in &self.table {
            if name == s {
                return Some((f, i));
            }
            i += 1;
        }
        None
    }
    fn search_start_pos(&self, start_pos: ProgramPos) -> Option<(Function, usize, &str)> {
        let mut i = 0usize;
        for &(f, ref s) in &self.table {
            if start_pos == f.start_pos {
                return Some((f, i, &s));
            }
            i += 1;
        }
        None
    }
}

#[derive(Debug)]
struct CallFrame {
    return_pos: ProgramPos,
    function_start: ProgramPos
}


pub struct Context {
    counter: ProgramPos,

    current_fun: Function,
    current_fun_pos: usize,
    functions: FunctionTable,

    program: Vec<Opcode<ProgramPos>>,

    data_stack: Vec<Literal>,
    call_stack: Vec<CallFrame>,
}

impl Context {
    pub fn new() -> Context {

        Context {
            counter: 0,

            current_fun: Function { start_pos: 0, end_pos: 0 },
            current_fun_pos: 0,
            functions: FunctionTable { table: vec![] },

            program: vec![],

            data_stack: vec![],
            call_stack: vec![],
        }
    }

    pub fn link(&mut self, new_functions: Vec<(&str, Vec<Opcode<&str>>)>) -> LinkResult<()> {
        self.extend_function_table(&new_functions);

        for (_, code) in new_functions {
            try!(self.link_step(code));
        }
        Ok(())
    }

    fn extend_function_table(&mut self, new_functions: &[(&str, Vec<Opcode<&str>>)]) {
        let mut start_pos = self.program.len() as ProgramPos;

        for &(name, ref code) in new_functions {
            let end_pos = start_pos + (code.len() as ProgramPos) - 1;

            self.functions.table.push((Function {
                start_pos: start_pos,
                end_pos: end_pos
            }, name.to_owned()));

            start_pos = end_pos + 1;
        }
    }

    fn link_step(&mut self, code: Vec<Opcode<&str>>) -> LinkResult<()> {
        for v in code {
            let instr : Opcode<ProgramPos> = match v {
                Call(name) => {
                    let (function, _) = try!(self.functions
                                        .search_name(name)
                                        .ok_or(FunctionNameNotFound));

                    Call(function.start_pos)
                },

                // do not repeat yourself! this is intolerable..
                
                Push(lit) => Push(lit),
                Pop => Pop,
                Swap(pos) => Swap(pos),
                BinOp(op) => BinOp(op),
                Branch(cond, offset) => Branch(cond, offset),
                Jump(offset) => Jump(offset),
                Return => Return,
            };

            self.program.push(instr);
        }
        Ok(())
    }

    fn fun_name(&self) -> &str {
        &self.functions.table[self.current_fun_pos].1
    }

    pub fn run(&mut self) -> (Vec<Literal>, VmError) {
        let mut result = Ok(());

        if self.program.len() == 0 {
            println!("There is no code");
            return (vec![], ThereIsNoCode);
        }

        self.current_fun = self.functions.table[self.current_fun_pos].0;

        println!("Program: {:?}", self.program);
        println!("Starting at {:?}", self.fun_name());

        while let Ok(()) = result {
            result = if self.counter > self.current_fun.end_pos {
                println!("Implicit return at end of {}", self.fun_name());
                self.ret()
            } else {
                println!("{}\tPC {:?} {:?}\tStack: {:?}",
                         self.fun_name(),
                         self.counter,
                         self.program[self.counter],
                         self.data_stack);

                self.step()
            };
        }

        let error = match result {
            Err(a) => a,
            Ok(()) => unreachable!(),
        };

        println!("-");
        println!("{}\tPC {:?} Stack {:?} Exit: {:?}",
                 self.fun_name(),
                 self.counter,
                 self.data_stack, error);

        (self.data_stack.clone(), error)
    }

    fn step(&mut self) -> VmResult<()> {
        let instr = self.program[self.counter];
        self.counter += 1;

        match instr {
            Push(val) => Ok(self.data_stack.push(val)),
            Pop => self.pop().map(|_| ()),

            Swap(stack_pos) => self.swap(stack_pos),

            BinOp(op) => self.binop(op),
            Branch(cond, offset) => self.branch(cond, offset),
            Jump(offset) => self.jump(offset),

            Call(function_pos) => self.call(function_pos),
            Return => self.ret(),
        }
    }

    fn get_fun(&self, function_pos: ProgramPos) -> VmResult<(Function, usize)> {
        let (f, p, _) = try!(self.functions
                          .search_start_pos(function_pos)
                          .ok_or(InvalidFunctionPosition));

        Ok((f, p))
    }

    fn call(&mut self, function_pos: ProgramPos) -> VmResult<()> {
        // the stack was previously advanced
        self.call_stack.push(CallFrame {
            return_pos: self.counter - 1,
            function_start: self.current_fun.start_pos
        });
        self.counter = function_pos;
        let (fun, pos) = try!(self.get_fun(function_pos));
        self.current_fun = fun;
        self.current_fun_pos = pos;

        Ok(())
    }

    fn ret(&mut self) -> VmResult<()> {
        let frame = try!(self.call_stack.pop().ok_or(Halted));
        self.counter = frame.return_pos + 1;
        let (fun, pos) = try!(self.get_fun(frame.function_start));
        self.current_fun = fun;
        self.current_fun_pos = pos;

        Ok(())
    }

    fn pop(&mut self) -> VmResult<Literal> {
        self.data_stack.pop().ok_or(StackUnderflow)
    }

    fn swap(&mut self, pos: StackPos) -> VmResult<()> {
        let last = self.data_stack.len() - 1;

        if pos > last {
            return Err(InvalidStackPosition);
        }

        self.data_stack.swap(last, last - pos);
        Ok(())
    }

    fn binop(&mut self, op: Operation) -> VmResult<()> {
        let a = try!(self.pop());
        let b = try!(self.pop());

        let v = match op {
            Add => a + b,
            Mul => a * b,
        };

        self.data_stack.push(v);
        Ok(())
    }

    fn branch(&mut self, cond: BranchCondition, offset: JumpOffset) -> VmResult<()> {
        let a = try!(self.pop());
        let b = try!(self.pop());

        match cond {
            Equal => if a == b { try!(self.jump(offset)); },
            GreaterThan => if a > b { try!(self.jump(offset)); }
        }
        Ok(())
    }

    fn jump(&mut self, offset: JumpOffset) -> VmResult<()> {
        let address = ((self.counter as JumpOffset) + offset - 1) as StackPos;

        if address < self.current_fun.start_pos {
            return Err(JumpOffsetTooSmall);
        }
        if address > self.current_fun.end_pos {
            return Err(JumpOffsetTooLarge);
        }

        self.counter = address;
        Ok(())
    }
}

#[test]
fn empty() {
    let mut c = Context::new();
    assert_eq!(c.run(), (vec![], ThereIsNoCode));
}

#[test]
fn add() {
    let mut c = Context::new();
    c.link(vec![("main", vec![Push(15), Push(6), BinOp(Add)])]).unwrap();
    assert_eq!(c.run(), (vec![21], Halted));
}

#[test]
fn noop() {
    // Jump(1) executes the next instruction (i.e. it's a NOOP).
    // Just vec![Jump(1)] doesn't actually work though.

    let mut c = Context::new();
    c.link(vec![("main", vec![Jump(1), Return])]).unwrap();
    assert_eq!(c.run(), (vec![], Halted));
}

#[test]
fn double() {
    let main : Vec<Opcode<&str>> = vec![Push(27), Call("double"), Push(5), BinOp(Add)];
    let double : Vec<Opcode<&str>> = vec![Push(2), BinOp(Mul)];  

    let mut c = Context::new();
    c.link(vec![("main", main), ("double", double)]).unwrap();
    assert_eq!(c.run(), (vec![59], Halted));
}
