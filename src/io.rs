#[allow(unused)]
use raylib::{RaylibHandle, RaylibThread};
use raylib::{
    color::Color,
    ffi::KeyboardKey,
    math::{Rectangle, Vector2},
    prelude::{RaylibDraw, RaylibTextureMode, RaylibTextureModeExt},
    text::Font,
    texture::RenderTexture2D,
};
use std::sync::Mutex;
pub struct IOInner {
    pub handle: RaylibHandle,
    pub thread: RaylibThread,
    pub render_texture: RenderTexture2D,
    pub font: Font,
}

#[derive(Clone, Copy)]
pub struct CharCell {
    pub bg_color: Col,
    pub fg_col: Col,
    pub is_char: bool,
    pub ch: char,
    pub cols: [[Col; 8]; 20],
}

pub struct FrameBufferInner {
    pub current_fg_color: Col,
    pub current_bg_color: Col,
    pub cursor_x: i16,
    pub cursor_y: i16,
    pub last_pressed_char: Option<char>,
    pub buffer: [[CharCell; 80]; 24],
    pub input_string: String,
}

pub static FRAME: Mutex<FrameBufferInner> = Mutex::new(FrameBufferInner::new());
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Col {
    Black,
    White,
    Red,
    Cyan,
    Violet,
    Green,
    Blue,
    Yellow,
    Orange,
    Brown,
    LightRed,
    DarkGrey,
    Grey,
    LightGreen,
    LightBlue,
    LightGrey,
    Blank,
}

impl IOInner {
    pub fn create() -> Self {
        let (mut handle, thread) = raylib::RaylibBuilder::default().build();
        let render_texture = handle.load_render_texture(&thread, 640, 480).unwrap();
        let font = handle.load_font(&thread, "ModernDOS8x16.ttf").unwrap();
        Self {
            handle,
            render_texture,
            font,
            thread,
        }
    }
    pub fn update(&mut self) {
        let mut frame = FRAME.lock().unwrap();
        if let Some(c) = self.handle.get_char_pressed() {
            frame.last_pressed_char = Some(c);
            frame.input_string.push(c);
            frame.put_char(c);
        } else if self.handle.is_key_pressed(KeyboardKey::KEY_ENTER) {
            frame.last_pressed_char = Some('\n');
            frame.input_string.push('\n');
            frame.put_char('\n');
        } else if self.handle.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            frame.last_pressed_char = Some(127 as char);
            frame.input_string.pop();
            frame.put_char(127 as char);
        }
        let mut tex = self
            .handle
            .begin_texture_mode(&self.thread, &mut self.render_texture);
        tex.clear_background(Col::Black.as_color());
        frame.render(&mut tex, &self.font);
        drop(tex);
        drop(frame);
        let mut draw = self.handle.begin_drawing(&self.thread);
        draw.clear_background(Col::Black.as_color());
        draw.draw_texture_pro(
            &self.render_texture,
            Rectangle::new(0., 0., 640., -480.),
            Rectangle::new(0., 0., 640. * 2., 480. * 2.),
            Vector2::zero(),
            0.0,
            Col::White.as_color(),
        );
    }
}

impl FrameBufferInner {
    pub const fn new() -> Self {
        Self {
            last_pressed_char: None,
            current_bg_color: Col::Black,
            current_fg_color: Col::Green,
            buffer: [[CharCell::new(); _]; _],
            cursor_x: 0,
            cursor_y: 0,
            input_string: String::new(),
        }
    }

    pub fn render<T>(&mut self, handle: &mut RaylibTextureMode<'_, T>, font: &Font) {
        for i in 0..self.buffer.len() {
            for j in 0..self.buffer[0].len() {
                let c = self.buffer[i][j];
                if c.is_char {
                    if c.ch == 0 as char {
                        continue;
                    }

                    handle.draw_rectangle(
                        j as i32 * 8,
                        i as i32 * 20,
                        20,
                        8,
                        c.bg_color.as_color(),
                    );

                    handle.draw_text_codepoint(
                        font,
                        c.ch as i32,
                        Vector2::new(j as f32 * 8. + 2., i as f32 * 20. + 2.),
                        16.,
                        c.fg_col.as_color(),
                    );
                } else {
                    for k in 0..20 {
                        for l in 0..8 {
                            handle.draw_pixel(
                                (j * 8 + l) as i32,
                                (i * 20 + k) as i32,
                                c.cols[k][l].as_color(),
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn shift_chars(&mut self) {
        for i in 0..23 {
            for j in 0..80 {
                if self.buffer[i + 1][j].is_char {
                    self.buffer[i][j] = self.buffer[i + 1][j];
                } else {
                    self.buffer[i][j] = CharCell::new();
                }
            }
        }
    }

    pub fn del_char(&mut self) {
        if self.cursor_x == 0 {
            if self.cursor_y == 0 {
                return;
            }
            self.cursor_y -= 1;
            self.cursor_x = 79;
        } else {
            self.cursor_x -= 1;
        }
        if self.buffer[self.cursor_y as usize][self.cursor_x as usize].is_char {
            self.buffer[self.cursor_y as usize][self.cursor_x as usize] = CharCell::new();
        }
    }

    pub fn put_char(&mut self, c: char) {
        if c == '\r' {
            self.cursor_x = 0;
            return;
        }
        if c == '\n' {
            if self.cursor_y == 23 {
                self.shift_chars();
                self.cursor_x = 0;
            } else {
                self.cursor_x = 0;
                self.cursor_y += 1;
            }
            return;
        } else if c == 127 as char {
            self.del_char();
            return;
        }
        if self.cursor_x == 79 {
            if self.cursor_y == 23 {
                self.shift_chars();
                self.cursor_x = 0;
            } else {
                self.cursor_x = 0;
                self.cursor_y += 1;
            }
        }
        self.buffer[self.cursor_y as usize][self.cursor_x as usize] = CharCell {
            bg_color: self.current_bg_color,
            fg_col: self.current_fg_color,
            is_char: true,
            ch: c,
            cols: [[Col::Black; _]; _],
        };
        self.cursor_x += 1;
        if self.cursor_x >= 80 {
            self.cursor_x = 79;
        }
        if self.cursor_y >= 24 {
            self.cursor_y = 23;
        }
    }

    pub fn write_s(&mut self, s: &str) {
        for i in s.chars() {
            self.put_char(i);
        }
    }

    pub fn put_pixel(&mut self, x: i16, y: i16, col: Col) {
        let x1 = x / 8;
        let xoff = x % 8;
        let y1 = y / 20;
        let yoff = y % 20;
        self.buffer[y1 as usize][x1 as usize].cols[yoff as usize][xoff as usize] = col;
        self.buffer[y1 as usize][x1 as usize].is_char = false;
    }
}

impl CharCell {
    pub const fn new() -> Self {
        CharCell {
            fg_col: Col::Green,
            bg_color: Col::Black,
            ch: 0 as char,
            is_char: true,
            cols: [[Col::Black; _]; _],
        }
    }
}

impl Col {
    pub fn as_color(&self) -> Color {
        match self {
            Col::Black => Color::new(0, 0, 0, 255),
            Col::White => Color::new(255, 255, 255, 255),
            Col::Red => Color::new(136, 0, 0, 255),
            Col::Cyan => Color::new(170, 255, 238, 255),
            Col::Violet => Color::new(204, 67, 204, 255),
            Col::Green => Color::new(0, 204, 85, 255),
            Col::Blue => Color::new(0, 0, 170, 255),
            Col::Yellow => Color::new(238, 238, 119, 255),
            Col::Orange => Color::new(221, 136, 85, 255),
            Col::Brown => Color::new(102, 68, 0, 255),
            Col::LightRed => Color::new(255, 119, 119, 255),
            Col::DarkGrey => Color::new(51, 51, 51, 255),
            Col::Grey => Color::new(119, 119, 119, 255),
            Col::LightGreen => Color::new(170, 255, 102, 255),
            Col::LightBlue => Color::new(0, 136, 255, 255),
            Col::LightGrey => Color::new(187, 187, 187, 255),
            Col::Blank => Color::new(0, 0, 0, 0),
        }
    }
    pub fn from_color(c: &Color) -> Self {
        let mut min_delta = 255 * 255 * 3;
        let mut min_color = Col::Blank;
        if c.a < 128 {
            return min_color;
        }
        for i in COLORS {
            let c2 = i.as_color();
            let dr = c.r as i32 - c2.r as i32;
            let dg = c.g as i32 - c2.g as i32;
            let db = c.b as i32 - c2.b as i32;
            let delt = dr * dr + dg * dg + db * db;
            if delt < min_delta {
                min_delta = delt;
                min_color = i;
            }
        }
        min_color
    }
}

pub const COLORS: [Col; 17] = [
    Col::Black,
    Col::White,
    Col::Red,
    Col::Cyan,
    Col::Violet,
    Col::Green,
    Col::Blue,
    Col::Yellow,
    Col::Orange,
    Col::Brown,
    Col::LightRed,
    Col::DarkGrey,
    Col::Grey,
    Col::LightGreen,
    Col::LightBlue,
    Col::LightGrey,
    Col::Blank,
];

pub fn write_s(s: &str) {
    FRAME.lock().unwrap().write_s(s);
}

pub fn write_pixel(x: i16, y: i16, col: Col) {
    FRAME.lock().unwrap().put_pixel(x, y, col);
}

pub fn write_rect(x: i16, y: i16, w: i16, h: i16, col: Col) {
    let mut frame = FRAME.lock().unwrap();
    for y1 in y..y + h {
        for x1 in x..x + w {
            frame.put_pixel(x1, y1, col);
        }
    }
}
