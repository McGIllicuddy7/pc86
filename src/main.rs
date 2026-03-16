use std::time::Duration;

use raylib::ffi::KeyboardKey;

use crate::io::{Col, put_str, window_should_close};

pub mod input;
pub mod io;
_start! {{
    put_str("hewwo");
    for i in 0..100{
        put_str(&format!("{}\n",i));
    }
    while !window_should_close(){
    }
}}
