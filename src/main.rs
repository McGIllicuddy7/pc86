use std::time::Duration;

use crate::io::execute_program;
use crate::io::{Col, StabStr, get_line, put_rect, put_str, window_should_close};
use raylib::ffi::KeyboardKey;
pub mod input;
pub mod io;
pub mod managed;
_start! {{
    put_str("hellow");
    for i in 0..20{
        put_str(&format!("[ {} ]",i));
    }

    std::thread::sleep(std::time::Duration::from_secs(1));
    let out = execute_program("./main.so".into(),(["hello world!\0"].as_slice()));
    println!("a.out returned {}", out);
    let s = get_line();
    println!("{:#?}",s);
}}
