use std::collections::HashMap;

mod vm;

use vm::Function;
use vm::Context;

use vm::Opcode::*;
use vm::Operation::*;

// this should be in the standard library ...
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = HashMap::new();
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
