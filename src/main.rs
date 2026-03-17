use std::time::Duration;

use crate::io::{Col, get_line, put_rect, put_str, window_should_close};
use raylib::ffi::KeyboardKey;

pub mod input;
pub mod io;
_start! {{
    put_str(stabby::str::Str::new("hellow"));
    for i in 0..20{
        put_str(stabby::str::Str::new(&format!("[ {} ]",i)));
    }
    let s = get_line();
    println!("{:#?}",s);
}}
