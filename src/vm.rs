use std::collections::HashMap;

use self::Operation::*;
use self::BranchCondition::*;
use self::Opcode::*;
use self::Error::*;

type Literal = i32;

type StackPos = usize;
type CounterPos = usize;
type JumpOffset = isize;

#[derive(Copy, Clone, Debug)]
enum Operation {
    Add,
    Mul,
}

#[derive(Copy, Clone, Debug)]
enum BranchCondition {
    Equal,
    GreaterThan,
}

#[derive(Copy, Clone, Debug)]
enum Opcode<'a> {
    Push(Literal),
    Pop,
    Swap(StackPos),
    BinOp(Operation),
    Branch(BranchCondition, JumpOffset),
    Jump(JumpOffset),
    Call(&'a str),
    Return,
}

#[derive(Copy, Clone, Debug)]
enum Error {
    StackUnderflow,
    TooFewParameters,
    InvalidStackPosition,
    InvalidJumpOffset,
    FunctionNotFound,
    Halted,
}

type Result<T> = std::result::Result<T, Error>;

const MAIN_FUNCTION : &'static str = "main";

#[derive(Copy, Clone, Debug)]
struct ReturnValue<'a> {
    function: &'a str,
    counter: CounterPos,
}

#[derive(Copy, Clone, Debug)]
struct Function<'a> {
    name: &'a str,
    code: &'a [Opcode<'a>],
}

#[derive(Debug)]
struct Context<'a> {
    counter: CounterPos,
    current_fun: Function<'a>,
    program: HashMap<&'a str, Function<'a>>,

    data_stack: Vec<Literal>,
    call_stack: Vec<ReturnValue<'a>>,
}

impl<'a> Context<'a> {
    fn new(program: HashMap<&'a str, Function<'a>>) -> Option<Context<'a>> {
        program.get(MAIN_FUNCTION).map(|v| *v).map(|q| {
            Context {
                counter: 0,
                current_fun: q,
                program: program,
                data_stack: vec![],
                call_stack: vec![],
            }
        })
    }

    fn run(&mut self) {
        let mut result = Ok(());

        println!("Program: {:?}", self.program);
        println!("Starting at {:?}", MAIN_FUNCTION);

        while let Ok(()) = result {
            let last = self.current_fun.code.len() - 1;

            result = if self.counter > last {
                println!("Implicit return at end of {}", self.current_fun.name);
                self.ret()
            } else {
                println!("{}\tPC {:?} {:?}\tStack: {:?}",
                         self.current_fun.name,
                         self.counter,
                         self.current_fun.code[self.counter],
                         self.data_stack);

                self.step()
            };
        }

        println!("-");
        println!("{}\tPC {:?} Stack {:?} Exit: {:?}",
                 self.current_fun.name,
                 self.counter,
                 self.data_stack, result);
    }

    fn step(&mut self) -> Result<()> {
        let instr = self.current_fun.code[self.counter];
        self.counter += 1;

        match instr {
            Push(val) => Ok(self.data_stack.push(val)),
            Pop => self.pop().map(|_| ()),

            Swap(pos) => self.swap(pos),

            BinOp(op) => self.binop(op),
            Branch(cond, offset) => self.branch(cond, offset),
            Jump(offset) => self.jump(offset),

            Call(fun_name) => self.call(fun_name),
            Return => self.ret(),
        }
    }


    fn get_function(&mut self, fun_name: &str) -> Result<Function<'a>> {
        self.program.get(fun_name).map(|f| *f).ok_or(FunctionNotFound)
    }

    fn call(&mut self, fun_name: &'a str) -> Result<()> {
        let fun = try!(self.get_function(fun_name));

        self.call_stack.push(ReturnValue {
            function: self.current_fun.name,
            counter: self.counter,
        });
        self.counter = 0;
        self.current_fun = fun;

        Ok(())
    }

    fn ret(&mut self) -> Result<()> {
        let val = try!(self.call_stack.pop().ok_or(Halted));

        self.current_fun = try!(self.get_function(val.function));

        self.counter = val.counter;
        Ok(())
    }



    fn pop(&mut self) -> Result<Literal> {
        self.data_stack.pop().ok_or(StackUnderflow)
    }

    fn swap(&mut self, pos: StackPos) -> Result<()> {
        let last = self.data_stack.len() - 1;

        if pos > last {
            return Err(InvalidStackPosition);
        }

        self.data_stack.swap(last, last - pos);
        Ok(())
    }

    fn binop(&mut self, op: Operation) -> Result<()> {
        let a = try!(self.pop());
        let b = try!(self.pop());

        let v = match op {
            Add => a + b,
            Mul => a * b,
        };

        self.data_stack.push(v);
        Ok(())
    }

    fn branch(&mut self, cond: BranchCondition, offset: JumpOffset) -> Result<()> {
        let a = try!(self.pop());
        let b = try!(self.pop());

        match cond {
            Equal => if a == b { try!(self.jump(offset)); },
            GreaterThan => if a > b { try!(self.jump(offset)); }
        }
        Ok(())
    }

    fn jump(&mut self, offset: JumpOffset) -> Result<()> {
        let address = (self.counter as JumpOffset) + offset;
        let last = (self.current_fun.code.len() - 1) as JumpOffset;

        if address < 0 || address > last {
            return Err(InvalidJumpOffset);
        }

        self.counter = address as StackPos;
        Ok(())
    }
}

// this should be in the standard library ...
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

fn main() {
    let main = vec![Push(1), Call("double"), Push(5), BinOp(Add)];
    let double = vec![Push(2), BinOp(Mul)];

    let program = hashmap! {
        "main" => Function {
            name: "main",
            code: &main
        },
        "double" => Function {
            name: "double",
            code: &double
        }
    };

    let mut context = Context::new(program).unwrap();

    context.run();
}
