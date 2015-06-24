use self::Operation::*;
use self::BranchCondition::*;
use self::Opcode::*;
use self::Error::*;

type Value = u32;

type StackPos = usize;
type JumpOffset = isize;

#[derive(Copy, Clone, Debug)]
enum Operation {
    Add,
    Mul
}

#[derive(Copy, Clone, Debug)]
enum BranchCondition {
    Equal,
    GreaterThan
}

#[derive(Copy, Clone, Debug)]
enum Opcode {
    Push(Value),
    Pop,
    Swap(StackPos),
    BinOp(Operation),
    Branch(BranchCondition, JumpOffset),
    Jump(JumpOffset),
    Halt
}

#[derive(Copy, Clone, Debug)]
enum Error {
    StackUnderflow,
    InvalidStackPosition,
    InvalidJumpOffset,
    EndOfProgram,
    Halted
}

// Instruction Result
type IResult = Result<(), Error>;

#[allow(non_upper_case_globals)]
const ret : IResult = Ok(());

#[derive(Debug)]
struct Context<'a> {
    counter: usize,
    stack: Vec<Value>,
    code: &'a [Opcode]
}

impl<'a> Context<'a> {
    fn new(code: &[Opcode]) -> Context {
        Context { counter: 0, stack: vec![], code: code }
    }

    fn run(&mut self) {
        let mut result = ret;

        let last = self.code.len() - 1;

        println!("Program: {:?}", self.code);


        while let Ok(()) = result {
            result = if self.counter > last {
                Err(EndOfProgram)
            } else {
                println!("PC {:?} {:?}\tStack: {:?}",
                         self.counter,
                         self.code[self.counter],
                         self.stack);

                self.step()
            };
        }

        println!("-");
        println!("PC {:?} Stack {:?} Exit: {:?}", self.counter, self.stack, result);
    }

    fn step(&mut self) -> IResult {
        let instr = self.code[self.counter];
        self.counter += 1;

        match instr {
            Push(val) => Ok(self.stack.push(val)),

            Swap(pos) => self.swap(pos),
            Pop => self.pop().map(|_| ()),
            BinOp(op) => self.binop(op),
            Branch(cond, offset) => self.branch(cond, offset),
            Jump(offset) => self.jump(offset),

            Halt => Err(Halted)
        }
    }

    fn pop(&mut self) -> Result<Value, Error> {
        self.stack.pop().ok_or(StackUnderflow)
    }

    fn swap(&mut self, pos: StackPos) -> IResult {
        let last = self.stack.len() - 1;

        if pos > last {
            return Err(InvalidStackPosition);
        }

        self.stack.swap(last, last - pos);

        ret
    }

    fn binop(&mut self, op: Operation) -> IResult {
        let a = try!(self.pop());
        let b = try!(self.pop());

        let v = match op {
            Add => a + b,
            Mul => a * b
        };

        self.stack.push(v);
        ret
    }

    fn jump(&mut self, offset: JumpOffset) -> IResult {
        let address = (self.counter as isize) + offset;
        let last = (self.code.len() - 1) as isize;

        if address < 0 || address > last {
            return Err(InvalidJumpOffset);
        }

        self.counter = address as usize;
        ret
    }

    fn branch(&mut self, cond: BranchCondition, offset: JumpOffset) -> IResult {
        let a = try!(self.pop());
        let b = try!(self.pop());

        match cond {
            Equal => if a == b { try!(self.jump(offset)); },
            GreaterThan => if a > b { try!(self.jump(offset)); }
        }
        ret
    }
}

fn main() {
    let program = vec![Push(1), Push(2), Jump(0), BinOp(Mul), Push(3), BinOp(Add)];

    let mut context = Context::new(&program);

    context.run();
}
