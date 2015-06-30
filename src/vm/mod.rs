#![allow(dead_code)]

mod tests;

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
}

include!("vm_link.rs");
include!("vm_run.rs");
