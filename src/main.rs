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
    while !window_should_close(){
        put_rect(40, 40, Col::Cyan);
        std::thread::sleep(Duration::from_millis(1000));
    }
}}
