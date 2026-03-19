pub type KeyboardKey = raylib::ffi::KeyboardKey;
pub type MouseButton = raylib::ffi::MouseButton;
use lazy_static::lazy_static;
use raylib::prelude::RaylibDrawHandle;
use raylib::texture::{RaylibRenderTexture2D, Texture2D};
#[allow(unused)]
use raylib::{RaylibHandle, RaylibThread};
#[allow(unused)]
use raylib::{
    color::Color,
    math::{Rectangle, Vector2},
    prelude::{RaylibDraw, RaylibTextureMode, RaylibTextureModeExt},
    text::Font,
    texture::RenderTexture2D,
};
use stabby::slice::Slice;
pub use stabby::str::Str as StabStr;
pub use stabby::string::String as StabString;
pub use stabby::vec::Vec as StabVec;
pub use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;

use crate::input::{Input, generate_input};
pub struct IOInner {
    pub handle: RaylibHandle,
    pub thread: RaylibThread,
    pub render_texture: ManuallyDrop<RenderTexture2D>,
    pub font: ManuallyDrop<Font>,
    pub input: Input,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[stabby::stabby]
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

pub struct ImageData {
    data: Box<[Col]>,
    w: i32,
    h: i32,
    tex: Option<Texture2D>,
}

#[derive(Clone, Debug, Copy)]
#[stabby::stabby]
pub struct Image {
    inner: u32,
}

pub struct FrameBuffer {
    pub objects: Vec<Drawbject>,
    pub write_buffer: Vec<Drawbject>,
    pub images: HashMap<u32, Arc<RwLock<ImageData>>>,
    pub terminal_mode: bool,
    pub input: Input,
    pub fg_color: Col,
    pub bg_color: Col,
    pub last_char: Option<char>,
    pub frame_char: Option<char>,
    pub input_string: String,
}

#[derive(Clone, Debug)]
pub enum Drawbject {
    Shift {
        dx: i16,
        dy: i16,
    },
    Move {
        x: i16,
        y: i16,
    },
    CharSeq {
        x: i16,
        y: i16,
        max_w: i16,
        user_input: bool,
        fg_color: Col,
        bg_color: Col,
        seq: String,
    },
    Rectangle {
        x: i16,
        y: i16,
        w: i16,
        h: i16,
        col: Col,
    },
    Circle {
        x: i16,
        y: i16,
        r: i16,
        col: Col,
    },
    Image {
        x: i16,
        y: i16,
        w: i16,
        h: i16,
        img: Image,
    },
    Line {
        x0: i16,
        y0: i16,
        x1: i16,
        y1: i16,
        col: Col,
    },
}
impl std::fmt::Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", "f")
    }
}
lazy_static! {
    static ref FRAME_BUFFER: Mutex<FrameBuffer> = Mutex::new(FrameBuffer::new());
}
impl IOInner {
    pub fn create() -> Self {
        let (mut handle, thread) = raylib::RaylibBuilder::default()
            .height(480 * 2)
            .width(640 * 2)
            .build();
        let render_texture =
            ManuallyDrop::new(handle.load_render_texture(&thread, 640, 480).unwrap());
        handle.set_target_fps(61);
        let font = ManuallyDrop::new(handle.load_font(&thread, "ModernDOS8x16.ttf").unwrap());
        Self {
            handle,
            render_texture,
            font,
            thread,
            input: Input::new(),
        }
    }
    pub fn update(&mut self) {
        let mut lock = FRAME_BUFFER.lock().unwrap();
        let mut draw = lock.update(
            &mut self.handle,
            &self.font,
            &self.thread,
            &mut self.render_texture,
        );
        drop(lock);
        drop(draw);
    }
}

impl Drop for IOInner {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.font);
            ManuallyDrop::drop(&mut self.render_texture);
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

impl FrameBuffer {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            write_buffer: Vec::new(),
            images: HashMap::new(),
            terminal_mode: true,
            input: Input::new(),
            fg_color: Col::Green,
            bg_color: Col::Black,
            input_string: String::new(),
            last_char: None,
            frame_char: None,
        }
    }

    pub fn render_out<T>(
        &mut self,
        draw: &mut RaylibTextureMode<T>,
        font: &Font,
        objects: Vec<Drawbject>,
        base_x: i32,
        base_y: i32,
        _end_x: i32,
        end_y: i32,
    ) {
        fn h_round(v: i32) -> i32 {
            if v % 20 != 0 { v + 20 - v % 20 } else { v }
        }
        fn w_round(v: i32) -> i32 {
            if v % 8 != 0 { v + 8 - v % 80 } else { v }
        }
        let mut y = base_y;
        let mut x = base_x;
        if end_y - base_y > 480 {
            y -= end_y - base_y - 460;
        }
        for i in objects {
            match &i {
                Drawbject::Shift { dx: _, dy: _ } => {
                    _ = ();
                }
                Drawbject::Move { x: _, y: _ } => {
                    _ = ();
                }
                Drawbject::Line {
                    x0,
                    y0,
                    x1,
                    y1,
                    col,
                } => {
                    draw.draw_line(
                        *x0 as i32,
                        *y0 as i32,
                        *x1 as i32,
                        *y1 as i32,
                        col.as_color(),
                    );
                }
                Drawbject::CharSeq {
                    x: _,
                    y: _,
                    max_w: _,
                    user_input: _,
                    fg_color,
                    bg_color: _,
                    seq,
                } => {
                    for c in seq.chars() {
                        if c != '\n' {
                            draw.draw_text_codepoint(
                                font,
                                c as i32,
                                Vector2::new(x as f32, y as f32),
                                16.,
                                fg_color.as_color(),
                            );
                            x += 8;
                            if x >= 640 {
                                x = base_x;
                                y += 20;
                            }
                        } else {
                            x = base_x;
                            y += 20;
                        }
                    }
                }
                Drawbject::Rectangle {
                    x: _,
                    y: _,
                    w,
                    h,
                    col,
                } => {
                    draw.draw_rectangle(x, y, *w as i32, *h as i32, col.as_color());
                    x += w_round(*w as i32);
                    y += h_round(*h as i32 - 20);
                    if x > 640 {
                        x = base_x;
                        y += 20;
                    }
                }
                Drawbject::Circle { x: _, y: _, r, col } => {
                    draw.draw_circle(x, y, *r as f32, col.as_color());
                    x += w_round(*r as i32);
                    y += h_round(*r as i32);
                    if x > 640 {
                        x = base_x;
                        y += 20;
                    }
                }
                Drawbject::Image { x, y, w, h, img } => todo!(),
            }
        }
    }

    pub fn update<'a>(
        &mut self,
        handle: &'a mut RaylibHandle,
        font: &Font,
        thread: &RaylibThread,
        texture: &'a mut RenderTexture2D,
    ) -> RaylibDrawHandle<'a> {
        let should_close = self.input.window_should_close;
        if should_close {
            println!("{}", should_close);
        }
        self.input = generate_input(handle, 2., 2.);
        self.input.window_should_close = self.input.window_should_close || should_close;
        fn h_round(v: i32) -> i32 {
            if v % 20 != 0 { v + 20 - v % 20 } else { v }
        }
        fn w_round(v: i32) -> i32 {
            if v % 8 != 0 { v + 8 - v % 80 } else { v }
        }
        if self.terminal_mode {
            if let Some(c) = handle.get_char_pressed() {
                handle_char_input(
                    &mut self.objects,
                    c,
                    self.fg_color,
                    self.bg_color,
                    &mut self.input_string,
                );
                self.last_char = Some(c);
                self.frame_char = Some(c);
            } else if handle.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
                handle_char_input(
                    &mut self.objects,
                    127 as char,
                    self.fg_color,
                    self.bg_color,
                    &mut self.input_string,
                );
                self.last_char = Some(127 as char);
                self.frame_char = Some(127 as char);
            } else if handle.is_key_pressed(KeyboardKey::KEY_ENTER) {
                handle_char_input(
                    &mut self.objects,
                    '\n',
                    self.fg_color,
                    self.bg_color,
                    &mut self.input_string,
                );
                self.last_char = Some('\n');
                self.frame_char = Some('\n');
            } else {
                self.frame_char = None;
            }
        }
        let mut draw = handle.begin_texture_mode(&thread, texture);
        draw.clear_background(Color::BLACK);
        if self.terminal_mode {
            if self.objects.len() > 200 {
                let min = self.objects.len() - 100;
                self.objects = self
                    .objects
                    .clone()
                    .into_iter()
                    .enumerate()
                    .filter(|i| i.0 > min)
                    .map(|i| i.1)
                    .map(|mut i| {
                        match &mut i {
                            Drawbject::CharSeq {
                                x: _,
                                y: _,
                                max_w: _,
                                user_input: _,
                                fg_color: _,
                                bg_color: _,
                                seq,
                            } => {
                                let l = seq.len();
                                if l > 4000 {
                                    let tmp: String = seq
                                        .chars()
                                        .enumerate()
                                        .filter(|i| i.0 > l - 2000)
                                        .map(|i| i.1)
                                        .collect::<std::string::String>()
                                        .into();
                                    *seq = tmp;
                                }
                            }
                            _ => {}
                        };
                        i
                    })
                    .collect();
            }
            let mut current: Vec<Drawbject> = Vec::new();
            let mut base_x = 0;
            let mut base_y = 0;
            let mut cx = 0;
            let mut cy = 0;
            for i in self.objects.clone() {
                match &i {
                    Drawbject::Shift { dx, dy } => {
                        let tmp = self.render_out(
                            &mut draw,
                            font,
                            current.clone(),
                            base_x,
                            base_y,
                            cx,
                            cy,
                        );
                        base_x = cx + *dx as i32 * 8;
                        base_y = cy + *dy as i32 * 20;
                        current = Vec::new();
                    }
                    Drawbject::Move { x, y } => {
                        base_x = *x as i32 * 8;
                        base_y = *y as i32 * 20;
                        cx = base_x;
                        cy = base_y;
                        let tmp = self.render_out(
                            &mut draw,
                            font,
                            current.clone(),
                            base_x,
                            base_y,
                            cx,
                            cy,
                        );
                        current = Vec::new()
                    }
                    Drawbject::CharSeq {
                        x: _,
                        y: _,
                        max_w: _,
                        user_input: _,
                        fg_color: _,
                        bg_color: _,
                        seq,
                    } => {
                        for c in seq.chars() {
                            if c == '\n' {
                                cx = base_x;
                                cy += 20;
                            } else {
                                cx += 8;
                                if cx >= 640 {
                                    cx = base_x;
                                    cy += 20;
                                }
                            }
                        }
                        current.push(i.clone());
                    }
                    Drawbject::Rectangle {
                        x: _,
                        y: _,
                        w,
                        h,
                        col: _,
                    } => {
                        cx += w_round(*w as i32);
                        cy += h_round(*h as i32 - 20);
                        if cx > 640 {
                            cx = base_x;
                            cy += 20;
                        }
                        current.push(i.clone());
                    }
                    Drawbject::Line {
                        x0,
                        y0,
                        x1,
                        y1,
                        col,
                    } => {
                        current.push(i.clone());
                    }
                    Drawbject::Circle {
                        x: _,
                        y: _,
                        r,
                        col: _,
                    } => {
                        cx += w_round(*r as i32);
                        if cx > 640 {
                            cx = base_x;
                            cy += 20;
                        }
                        cy += h_round(*r as i32);
                        current.push(i.clone());
                    }
                    Drawbject::Image {
                        x: _,
                        y: _,
                        w,
                        h,
                        img: _,
                    } => {
                        cx += w_round(*w as i32);
                        cy += h_round(*h as i32);
                        if cx > 640 {
                            cx = base_x;
                            cy += 20;
                        }
                        current.push(i.clone());
                    }
                }
            }
            self.render_out(&mut draw, font, current.clone(), base_x, base_y, cx, cy);
        } else {
            for i in self.objects.clone() {
                match i {
                    Drawbject::Line {
                        x0,
                        y0,
                        x1,
                        y1,
                        col,
                    } => {
                        draw.draw_line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, col.as_color());
                    }
                    Drawbject::Shift { dx: _, dy: _ } => {}
                    Drawbject::Move { x: _, y: _ } => {}
                    Drawbject::CharSeq {
                        x,
                        y,
                        user_input: _,
                        fg_color,
                        max_w,
                        bg_color: _,
                        seq,
                    } => {
                        let mut dx = x;
                        let mut dy = y;
                        let max_x = x + max_w;
                        for i in seq.chars() {
                            draw.draw_text_codepoint(
                                &font,
                                i as i32,
                                Vector2::new(dx as f32, dy as f32),
                                16.,
                                fg_color.as_color(),
                            );
                            dx += 8;
                            dy += 20;
                            if dx > max_x {
                                dx = x;
                                dy += 20;
                            }
                        }
                    }
                    Drawbject::Rectangle { x, y, w, h, col } => {
                        draw.draw_rectangle(x as i32, y as i32, w as i32, h as i32, col.as_color());
                    }
                    Drawbject::Circle { x, y, r, col } => {
                        draw.draw_circle(x as i32, y as i32, r as f32, col.as_color());
                    }
                    Drawbject::Image {
                        x: _,
                        y: _,
                        w: _,
                        h: _,
                        img: _,
                    } => {
                        todo!()
                    }
                }
            }
        }
        drop(draw);
        let mut draw = handle.begin_drawing(&thread);
        draw.clear_background(Color::BLACK);
        draw.draw_texture_pro(
            texture.texture(),
            Rectangle::new(0.0, 0.0, 640., -480.),
            Rectangle::new(0.0, 0.0, 640. * 2., 480. * 2.),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        draw.draw_fps(1000, 80);
        draw
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.write_buffer, &mut self.objects);
        self.write_buffer.clear();
    }
}
fn handle_char_input(
    objects: &mut Vec<Drawbject>,
    ch: char,
    ifg_color: Col,
    ibg_color: Col,
    input_string: &mut String,
) {
    if let Some(mut c) = objects.pop() {
        match &mut c {
            Drawbject::CharSeq {
                x: _,
                y: _,
                max_w: _,
                user_input,
                fg_color,
                bg_color,
                seq,
            } => {
                if *user_input && *fg_color == ifg_color && *bg_color == ibg_color {
                    if ch == 127 as char {
                        seq.pop();
                        if !seq.is_empty() {
                            objects.push(c);
                        }
                        input_string.pop();
                    } else {
                        input_string.push(ch);
                        seq.push(ch);
                        objects.push(c);
                    }
                } else {
                    objects.push(c);
                    if ch != 127 as char {
                        *input_string = ch.to_string();
                        add_char(objects, ch, ifg_color, ibg_color, true);
                    }
                }
            }
            _ => {
                objects.push(c);
                if ch != 127 as char {
                    *input_string = ch.to_string();
                    add_char(objects, ch, ifg_color, ibg_color, true);
                }
            }
        }
    } else {
        if ch != 127 as char {
            *input_string = ch.to_string();
            add_char(objects, ch, ifg_color, ibg_color, true);
        }
    }
}
fn add_char_seq(
    objects: &mut Vec<Drawbject>,
    chars: &str,
    ifg_color: Col,
    ibg_color: Col,
    is_user_input: bool,
) {
    if let Some(mut x) = objects.pop() {
        match &mut x {
            Drawbject::CharSeq {
                x: _,
                y: _,
                max_w: _,
                user_input,
                fg_color,
                bg_color,
                seq,
            } => {
                if *user_input == is_user_input {
                    if *fg_color == ifg_color && *bg_color == ibg_color {
                        *seq += chars;
                        objects.push(x);
                    } else {
                        objects.push(x);
                        objects.push(Drawbject::CharSeq {
                            x: 0,
                            y: 0,
                            max_w: 0,
                            user_input: is_user_input,
                            fg_color: ifg_color,
                            bg_color: ibg_color,
                            seq: chars.to_string(),
                        });
                    }
                } else {
                    objects.push(x);
                    objects.push(Drawbject::CharSeq {
                        x: 0,
                        y: 0,
                        max_w: 0,
                        user_input: is_user_input,
                        fg_color: ifg_color,
                        bg_color: ibg_color,
                        seq: chars.to_string(),
                    });
                }
            }
            _ => {
                objects.push(x);
                objects.push(Drawbject::CharSeq {
                    x: 0,
                    y: 0,
                    max_w: 0,
                    user_input: is_user_input,
                    fg_color: ifg_color,
                    bg_color: ibg_color,
                    seq: chars.to_string(),
                });
            }
        }
    } else {
        objects.push(Drawbject::CharSeq {
            x: 0,
            y: 0,
            max_w: 0,
            user_input: is_user_input,
            fg_color: ifg_color,
            bg_color: ibg_color,
            seq: chars.to_string(),
        });
    }
}

fn add_char(
    objects: &mut Vec<Drawbject>,
    ch: char,
    ifg_color: Col,
    ibg_color: Col,
    is_user_input: bool,
) {
    if let Some(mut x) = objects.pop() {
        match &mut x {
            Drawbject::CharSeq {
                x: _,
                y: _,
                max_w: _,
                user_input,
                fg_color,
                bg_color,
                seq,
            } => {
                if *user_input == is_user_input {
                    if *fg_color == ifg_color && *bg_color == ibg_color {
                        seq.push(ch);
                        objects.push(x);
                    } else {
                        objects.push(x);
                        objects.push(Drawbject::CharSeq {
                            x: 0,
                            y: 0,
                            max_w: 0,
                            user_input: is_user_input,
                            fg_color: ifg_color,
                            bg_color: ibg_color,
                            seq: ch.to_string(),
                        });
                    }
                } else {
                    objects.push(x);
                    objects.push(Drawbject::CharSeq {
                        x: 0,
                        y: 0,
                        max_w: 0,
                        user_input: is_user_input,
                        fg_color: ifg_color,
                        bg_color: ibg_color,
                        seq: ch.to_string(),
                    });
                }
            }
            _ => {
                objects.push(x);
                objects.push(Drawbject::CharSeq {
                    x: 0,
                    y: 0,
                    max_w: 0,
                    user_input: is_user_input,
                    fg_color: ifg_color,
                    bg_color: ibg_color,
                    seq: ch.to_string(),
                });
            }
        }
    } else {
        objects.push(Drawbject::CharSeq {
            x: 0,
            y: 0,
            max_w: 0,
            user_input: is_user_input,
            fg_color: ifg_color,
            bg_color: ibg_color,
            seq: ch.to_string(),
        });
    }
}

pub fn put_str(s: &str) {
    let mut lock = FRAME_BUFFER.lock().unwrap();
    let fg = lock.fg_color;
    let bg = lock.bg_color;
    if lock.terminal_mode {
        add_char_seq(&mut lock.objects, s, fg, bg, false);
    } else {
        return;
    }
}

pub fn put_rect(w: i32, h: i32, col: Col) {
    let mut lock = FRAME_BUFFER.lock().unwrap();
    if lock.terminal_mode {
        lock.objects.push(Drawbject::Rectangle {
            x: 0,
            y: 0,
            w: w as i16,
            h: h as i16,
            col,
        })
    } else {
        return;
    }
}

pub fn window_should_close() -> bool {
    let out = FRAME_BUFFER.lock().unwrap().input.window_should_close;
    out
}

#[macro_export]
macro_rules! _start {
    ($blck:tt) => {
        fn main() {
            let thread = std::thread::spawn(|| $blck);
            io::run(thread);
        }
    };
}

pub fn run(thread: JoinHandle<()>) {
    let mut io = IOInner::create();
    while !thread.is_finished() {
        io.update();
        if window_should_close() {
            break;
        }
    }
}

pub fn get_char() -> i32 {
    loop {
        let mut frame = FRAME_BUFFER.lock().unwrap();
        if frame.input.window_should_close {
            return 0;
        } else if let Some(c) = frame.last_char.take() {
            return c as i32;
        }
        drop(frame);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
}

pub fn get_line() -> String {
    loop {
        let mut frame = FRAME_BUFFER.lock().unwrap();
        if frame.input.window_should_close {
            return String::new().into();
        } else if let Some((a, b)) = frame.input_string.split_once('\n') {
            let out = a.to_string();
            let rem = b.to_string();
            frame.input_string = rem;
            return out;
        }
        drop(frame);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
}

pub fn get_input_char() -> i32 {
    if let Some(x) = FRAME_BUFFER.lock().unwrap().frame_char {
        x as i32
    } else {
        0
    }
}

pub fn execute_program(to_run: &str, args: &[&str]) -> i8 {
    unsafe {
        let f = libloading::Library::new(to_run).unwrap();
        let mut iargs: StabVec<StabStr> = StabVec::new();
        iargs.push(to_run.into());
        for i in args {
            iargs.push((*i).into());
        }
        let x: stabby::slice::Slice<'_, StabStr> = iargs.as_slice().into();
        let s: Result<
            libloading::Symbol<'_, unsafe extern "C" fn(stabby::slice::Slice<'_, StabStr>) -> i32>,
            _,
        > = f.get("_prog_start");

        let Ok(func) = s else {
            println!("failed to find _prog_start");
            return -1;
        };
        println!("running");
        (*func)(x);
        return 0;
    }
}

#[used]
pub static SYS_PUT_STR: unsafe extern "C" fn(stabby::str::Str) = sys_put_str;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_put_str(s: stabby::str::Str) {
    put_str(s.as_str());
}

#[used]
pub static SYS_TEST: unsafe extern "C" fn() = sys_test;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_test() {
    println!("hello from c");
}

#[used]
pub static SYS_WINDOW_SHOULD_CLOSE: unsafe extern "C" fn() -> bool = sys_window_should_close;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_window_should_close() -> bool {
    window_should_close()
}

#[used]
pub static SYS_PUT_RECT: unsafe extern "C" fn(i32, i32, Col) = sys_put_rect;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_put_rect(w: i32, h: i32, col: Col) {
    put_rect(w, h, col);
}
#[used]
pub static SYS_GET_CHAR: unsafe extern "C" fn() -> i32 = sys_get_char;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_get_char() -> i32 {
    get_char()
}

#[used]
pub static SYS_GET_LINE: unsafe extern "C" fn() -> StabString = sys_get_line;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_get_line() -> StabString {
    get_line().into()
}

#[used]
pub static SYS_GET_FRAME_CHAR: unsafe extern "C" fn() -> i32 = sys_get_input_char;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_get_input_char() -> i32 {
    get_input_char()
}

#[used]
pub static SYS_EXEC_PROGRAM: unsafe extern "C" fn(StabStr, &Slice<StabStr>) -> i8 =
    sys_exec_program;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_exec_program(to_run: StabStr, args: &Slice<StabStr>) -> i8 {
    let mut iargs = Vec::new();
    for i in args.iter() {
        iargs.push((*i).into());
    }
    execute_program(to_run.into(), &iargs)
}

pub fn display_set_graphics_modes() {
    FRAME_BUFFER.lock().unwrap().terminal_mode = false;
}

pub fn display_set_text_mode() {
    FRAME_BUFFER.lock().unwrap().terminal_mode = true;
}

pub fn display_swap_buffers() {
    let mut frame = FRAME_BUFFER.lock().unwrap();
    frame.swap_buffers();
}

pub fn display_draw_text(text: &str, x: i32, y: i32, w: i32) {
    let mut frame = FRAME_BUFFER.lock().unwrap();
    if frame.terminal_mode {
        todo!();
    } else {
        let fg = frame.fg_color;
        let bg = frame.bg_color;
        frame.write_buffer.push(Drawbject::CharSeq {
            x: x as i16,
            y: y as i16,
            max_w: w as i16,
            user_input: false,
            fg_color: fg,
            bg_color: bg,
            seq: text.into(),
        });
    }
}
pub fn display_draw_rect(x: i32, y: i32, w: i32, h: i32, col: Col) {
    let mut frame = FRAME_BUFFER.lock().unwrap();
    if frame.terminal_mode {
        todo!();
    } else {
        frame.write_buffer.push(Drawbject::Rectangle {
            x: x as i16,
            y: y as i16,
            w: w as i16,
            h: h as i16,
            col,
        });
    }
}

pub fn display_draw_circle(x: i32, y: i32, r: i32, col: Col) {
    let mut frame = FRAME_BUFFER.lock().unwrap();
    if frame.terminal_mode {
        todo!();
    } else {
        frame.write_buffer.push(Drawbject::Circle {
            x: x as i16,
            y: y as i16,
            r: r as i16,
            col,
        });
    }
}

pub fn display_draw_line(x0: i32, y0: i16, x1: i16, y1: i16, col: Col) {
    let mut frame = FRAME_BUFFER.lock().unwrap();
    if frame.terminal_mode {
        todo!();
    } else {
        frame.write_buffer.push(Drawbject::Line {
            x0: x0 as i16,
            x1: x1 as i16,
            y0: y0 as i16,
            y1: y1 as i16,
            col,
        });
    }
}
