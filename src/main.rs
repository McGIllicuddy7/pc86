use crate::io::{Col, write_rect, write_s};

pub mod io;
fn main() {
    let mut io = io::IOInner::create();
    write_s("hello world!\ntesting 1 2 3:3");
    write_rect(40, 40, 40, 40, Col::Green);
    while !io.handle.window_should_close() {
        io.update();
    }
}
