use std::time::Duration;

use crate::io::execute_program;
use crate::io::{Col, get_line, put_rect,window_should_close, StabStr, put_str};
use raylib::ffi::KeyboardKey;

pub mod input;
pub mod io;
_start! {{
    put_str(stabby::str::Str::new("hellow"));
    for i in 0..20{
        put_str(stabby::str::Str::new(&format!("[ {} ]",i)));
    }
     
    std::thread::sleep(std::time::Duration::from_secs(1));
    let out = execute_program("./main.so".into(),([StabStr::new("hello world!\0")].as_slice()).into());
    println!("a.out returned {}", out); 
    let s = get_line();
    println!("{:#?}",s);
}}
