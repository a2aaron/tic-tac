extern crate tic_tac;

use std::io::Read;

fn main() {
    let text = {
        let fname = std::env::args().nth(1).expect("first argument to be filename");
        let mut text = String::new();
        let mut file = std::fs::File::open(&fname).expect(&format!("file '{}' exists", fname));
        file.read_to_string(&mut text).unwrap();
        text
    };
    let program = tic_tac::bytecode::parse::parse(&text).expect("code to parse");
    let res = program.eval(&mut std::io::stdin(), &mut std::io::stdout());
    println!();
    println!("RESULT: {:?}", res);
}
