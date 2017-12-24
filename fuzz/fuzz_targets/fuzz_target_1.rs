#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate tic_tac;

use tic_tac::bytecode;
use std::io;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(&data) {
        if let Ok(program) = bytecode::parse::parse(s) {
            // println!("{:?}", s);
        }
    }
});
