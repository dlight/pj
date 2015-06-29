use std::collections::HashMap;

#[macro_use]
mod vm;

use vm::Function;
use vm::Context;

use vm::Opcode;
use vm::Opcode::*;
use vm::Operation::*;

macro_rules! program2 {
    ($vec:expr, $pos:expr, fn $name:ident { $($values:expr),* }) => {{
        let fun = vec![$($values),*];
        let fun_len = fun.len();
        let fun_pos = $pos;
        $vec.extend(fun);
        hashmap! {
            stringify!($name) => Function {
                name: stringify!($name),
                code: &$vec[fun_pos .. fun_pos + fun_len]
            }
        }
    }}
}

#[allow(dead_code)]
fn main() {
    let mut p : Vec<Opcode> = vec![];

    let q = program2! {
        p, 0,
        fn main {
            Push(1), Call("double"), Push(5), BinOp(Add)
        }
    };

    println!("{:?}", q);


    let mut prog = vec![];
    let main = vec![Push(1), Call("double"), Push(5), BinOp(Add)];
    let main_len = main.len();
    let main_pos = 0;

    let double = vec![Push(2), BinOp(Mul)];
    let double_len = double.len();
    let double_pos = main_pos + main_len;

    prog.extend(main);
    prog.extend(double);

    let program = hashmap! {
        "main" => Function {
            name: "main",
            code: &prog[main_pos .. main_pos + main_len],
        },
        "double" => Function {
            name: "double",
            code: &prog[double_pos .. double_pos + double_len],
        }
    };


    let mut context = Context::new(program).unwrap();

    let _ = context.run();
}




    /*let (codes, program) = {
        let mut codes_1 = Vec::new();
        let mut map = HashMap::new();
        let vec_1 = vec![Push(1), Call("double"), Push(5), BinOp(Add)];
        codes_1.push(vec_1);
        map.insert("main", Function { name: "main", code: &vec_1 });
        let vec_2 = vec![Push(2), BinOp(Mul)];
        codes_1.push(vec_2);
        map.insert("double", Function { name: "double", code: &vec_2 });
        (codes_1, map)
    };

    println!("{:?} {:?}", codes, program);

    macro_rules! program2 {
        (fn $name:ident { $($values:expr),* }) => {{
            (stringify!($name), vec![$($values),*])
        }}
    }

    println!("{:?}", program2! {
        fn main {
            1,
            2,
            3
        }
    });*/
