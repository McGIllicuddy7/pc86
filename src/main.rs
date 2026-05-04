pub use bios::*;
pub fn main() {
    setup_bios();
    let mut i: u32 = 0;
    let mut y = 32;
    for y in 0..16 {
        for x in 0..80 {
            draw_pixel(x, 4 + y, 1);
        }
    }
    draw_line(10, 0, 30, 32, 4);
    while should_continue() {
        write_char(((((i / 10) % 10) as u8) + '0' as u8) as char, 0, 0);
        i = i.wrapping_add(1);
        sync_frame_buffer();
    }
}
