#![allow(unused_imports)]

use vm::*;
use vm::Operation::*;
use vm::Opcode::*;
use vm::VmError::*;
use vm::BranchCondition::*;

#[test]
fn empty() {
    let mut c = Context::new();
    assert_eq!(c.run(), (vec![], ThereIsNoCode));
}

#[test]
fn add() {
    let mut c = Context::new_program(vec![("main",
                                           vec![Push(15), Push(6), BinOp(Add)])]).unwrap();
    assert_eq!(c.run(), (vec![21], Halted));
}

#[test]
fn noop() {
    // Jump(1) executes the next instruction (i.e. it's a NOOP).

    let mut c = Context::new();
    c.link(vec![("main", vec![Jump(1)])]).unwrap();
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

#[test]
fn simple_ops() {
    //let mut c = Context::new();
    //c
}

#[test]
fn sum() {
    fn sum_numbers(numbers: Vec<Literal>) -> (Literal, VmError) {
        let mut c = Context::new();

        let main = numbers.iter().map(|&x| Push(x)).chain(vec![
            Push(numbers.len() as Literal),
            Call("sum")].into_iter()).collect::<Vec<Opcode<&str>>>();

        c.link(vec![("main", main),
                    ("sum", vec![
                        Branch(GreaterThan, 2),
                        Return,
                        Push(0),
                        Swap(1),
                        // :a
                        Push(-1),
                        BinOp(Add),
                        Swap(2),
                        BinOp(Add),
                        Swap(1),
                        Branch(GreaterThan, -5), // to :a
                        Pop,
                        ])]).unwrap();
        let (mut a, b) = c.run();
        assert!(a.len() == 1);
        (a.pop().unwrap(), b)
    }

    let (expected, trials) : (Vec<Literal>, Vec<Vec<Literal>>) = vec![
        (0, vec![]),
        (5, vec![5]),
        (16, vec![18, 0, -2]),
        (24, vec![32, -3, 15, -20]),
        (21816, vec![1099, 8837, -121, 12001]),
        (-664, vec![1, -200, 534, -1000, 1, 0]),
        ].into_iter().unzip();

    let (v, r) : (Vec<Literal>, Vec<VmError>) = trials.into_iter().map(|x| {
            sum_numbers(x)
        }).unzip();

    assert!(r.into_iter().all(|x| x == Halted));
    assert_eq!(v, expected);
}

#[test]
fn factorial() {
    // fac(n) for n < 0 is defined as 1
    fn fac(n : Literal) -> (Literal, VmError) {
        let mut c = Context::new();
        c.link(vec![("main",
                     vec![Push(n),
                          Call("factorial")]),
                    ("factorial", vec![
                        Push(1),
                        Swap(1),
                        // :a
                        Branch(LessEqual, 8), // to :b
                        Dup,
                        Swap(2),
                        BinOp(Mul),
                        Swap(1),
                        Push(-1),
                        BinOp(Add),
                        Jump(-7), // to :a
                        // :b
                        Pop,
                        ])]).unwrap();

        let (mut a, b) = c.run();
        assert!(a.len() == 1);
        (a.pop().unwrap(), b)
    }

    let (v, r) : (Vec<Literal>, Vec<VmError>) = (-1..13).map(|x| {
        fac(x)
    }).unzip();

    assert!(r.into_iter().all(|x| x == Halted));
    assert_eq!(v, [1, 1, 1, 2, 6, 24, 120,
                   720, 5040, 40320, 362880,
                   3628800, 39916800, 479001600]);
}
