use std::collections::HashMap;

#[macro_use]
mod vm;

use vm::Function;
use vm::Context;

use vm::Opcode::*;
use vm::Operation::*;

#[allow(dead_code)]
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

    let _ = context.run();
}
