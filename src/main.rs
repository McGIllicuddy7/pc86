use std::time::Duration;

use raylib::ffi::KeyboardKey;

use crate::io::{Col, put_rect, put_str, window_should_close};

pub mod input;
pub mod io;
_start! {{
    put_str("hewwo");
    for i in 0..20{
        put_str(&format!("[ {} ]",i));
    }
    let mut idx = 0usize;
    while !window_should_close(){
        for i in 0..20{
            put_str(&format!("  i  "));
        }
        idx += 20;
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}}
