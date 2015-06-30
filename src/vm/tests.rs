#![allow(unused_imports)]

use vm::*;
use vm::Operation::*;
use vm::Opcode::*;
use vm::VmError::*;

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
