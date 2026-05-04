pub struct U8 {
    _x: std::sync::atomic::AtomicU8,
}
pub struct Char {
    _x: std::sync::atomic::AtomicU32,
    _col: std::sync::atomic::AtomicU8,
    _col_bg: std::sync::atomic::AtomicU8,
}
impl U8 {
    pub const fn new(v: u8) -> Self {
        Self {
            _x: std::sync::atomic::AtomicU8::new(v),
        }
    }
    pub fn set(&self, v: u8) {
        self._x.store(v, std::sync::atomic::Ordering::SeqCst);
    }
    pub fn get(&self) -> u8 {
        self._x.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Char {
    pub const fn new(v: char) -> Self {
        Self {
            _x: std::sync::atomic::AtomicU32::new(v as u32),
            _col: std::sync::atomic::AtomicU8::new(15),
            _col_bg: std::sync::atomic::AtomicU8::new(0),
        }
    }
    pub fn set(&self, v: char) {
        self._x.store(v as u32, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get(&self) -> char {
        char::from_u32(self._x.load(std::sync::atomic::Ordering::SeqCst)).unwrap()
    }

    pub fn get_col(&self) -> u8 {
        self._col.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_col(&self, col: u8) {
        self._col.store(col, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_col_bg(&self) -> u8 {
        self._col_bg.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_col_bg(&self, col: u8) {
        self._col_bg.store(col, std::sync::atomic::Ordering::SeqCst);
    }
}

pub struct BIOS {
    pub char_buffer: [[Char; 80]; 45],
    pub should_continue: U8,
    pub last_char: Char,
    pub wait_queue: std::sync::Mutex<Vec<std::thread::Thread>>,
    pub input_string: std::sync::Mutex<String>,
}
impl Default for BIOS {
    fn default() -> Self {
        Self::new()
    }
}
impl BIOS {
    pub const fn new() -> Self {
        Self {
            char_buffer: [const { [const { Char::new(0 as char) }; _] }; _],
            should_continue: U8::new(1),
            last_char: Char::new(0 as char),
            wait_queue: std::sync::Mutex::new(Vec::new()),
            input_string: std::sync::Mutex::new(String::new()),
        }
    }

    pub fn update(
        &self,
        guard: &mut std::io::StdoutLock,
        input: &mut std::io::StdinLock,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        use crossterm::*;
        use std::io::Write;
        _ = input;
        if let Ok(x) = event::poll(std::time::Duration::from_millis(4)) {
            if x {
                if let Ok(tmp) = crossterm::event::read() {
                    match tmp {
                        event::Event::Key(x) => match x.kind {
                            event::KeyEventKind::Press => match x.code {
                                event::KeyCode::Backspace => {
                                    self.last_char.set(127 as char);
                                }

                                event::KeyCode::Modifier(_m) => {}
                                event::KeyCode::Delete => {
                                    self.last_char.set(127 as char);
                                }
                                event::KeyCode::Char(c) => {
                                    if x.modifiers
                                        .contains(crossterm::event::KeyModifiers::CONTROL)
                                        && c == 'c'
                                    {
                                        self.should_continue.set(0);
                                    }
                                    self.last_char.set(c);
                                }
                                event::KeyCode::Esc => {
                                    self.should_continue.set(0);
                                }
                                event::KeyCode::End => {
                                    self.should_continue.set(0);
                                }
                                _ => {}
                            },
                            _ => {}
                        },

                        _ => {}
                    }
                }
            }
        }
        let _ = queue!(guard, style::SetBackgroundColor(style::Color::Reset));
        let _ = queue!(guard, style::SetForegroundColor(style::Color::Reset));

        let _ = queue!(guard, crossterm::cursor::MoveTo(0, 0));
        let _ = queue!(guard, terminal::Clear(terminal::ClearType::All));
        if should_exit() {
            let _ = guard.flush();
            let mut to_wake = match self.wait_queue.lock() {
                Ok(x) => x,
                Err(e) => e.into_inner(),
            };
            for i in to_wake.iter() {
                i.unpark();
            }
            to_wake.clear();
            return Ok(());
        }
        let mut cfg = 15;
        let mut cbg = 0;
        let _ = queue!(
            guard,
            crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(cfg))
        );
        let _ = queue!(
            guard,
            crossterm::style::SetBackgroundColor(crossterm::style::Color::AnsiValue(cbg))
        );
        let _ = queue!(guard, cursor::Hide);
        for y in 0..self.char_buffer.len() {
            for x in 0..self.char_buffer[y].len() {
                let fg = self.char_buffer[y][x].get_col();
                let bg = self.char_buffer[y][x].get_col_bg();
                let c = self.char_buffer[y][x].get();
                if fg != cfg {
                    cfg = fg;
                    let _ = queue!(
                        guard,
                        crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(
                            fg
                        ))
                    );
                }
                if bg != cbg {
                    cbg = bg;
                    let _ = queue!(
                        guard,
                        crossterm::style::SetBackgroundColor(crossterm::style::Color::AnsiValue(
                            bg
                        ))
                    );
                }
                let _ = queue!(guard, crossterm::cursor::MoveTo(x as u16, y as u16));
                let c = if c as u32 == 0 { ' ' } else { c };
                let _ = queue!(guard, crossterm::style::Print(c))?;
            }
        }
        guard.flush().unwrap();
        let base = start.elapsed();
        let y = format!("{} millis to render", base.as_millis());
        for (i, c) in y.chars().enumerate() {
            write_char(c, i as u16 + 60, 44);
        }
        while start.elapsed() < std::time::Duration::from_millis(16) {}
        let mut to_wake = match self.wait_queue.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };
        for i in to_wake.iter() {
            i.unpark();
        }
        to_wake.clear();
        Ok(())
    }

    pub fn run(&self) {
        crossterm::terminal::enable_raw_mode().unwrap();
        let mut guard = std::io::stdout().lock();
        let mut input = std::io::stdin().lock();
        while self.should_continue.get() != 0 {
            let _ = self.update(&mut guard, &mut input);
        }
        crossterm::execute!(&mut guard, crossterm::cursor::Show).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}

pub static BIOS: BIOS = BIOS::new();
pub fn setup_bios() {
    std::thread::spawn(|| {
        BIOS.run();
    });
}

pub fn write_char(c: char, x: u16, y: u16) {
    let x = x % 80;
    let y = y % 45;
    BIOS.char_buffer[y as usize][x as usize].set(c);
    BIOS.char_buffer[y as usize][x as usize].set_col(15);
    BIOS.char_buffer[y as usize][x as usize].set_col_bg(0);
}

pub fn write_char_color(c: char, x: u16, y: u16, col: u8) {
    let x = x % 80;
    let y = y % 45;
    BIOS.char_buffer[y as usize][x as usize].set(c);
    BIOS.char_buffer[y as usize][x as usize].set_col(col);
}

pub fn read_char() -> Option<char> {
    let tmp = BIOS.last_char.get();
    if tmp as u32 == 0 {
        return None;
    } else {
        BIOS.last_char.set(0 as char);
        return Some(tmp);
    }
}

pub fn should_exit() -> bool {
    BIOS.should_continue.get() == 0
}

pub fn should_continue() -> bool {
    BIOS.should_continue.get() != 0
}

pub fn draw_pixel(x: u16, y: u16, col: u8) {
    let x = x % 80;
    let y = y % 90;
    let v = '▄';
    let y_act = y / 2;
    let c = &BIOS.char_buffer[y_act as usize][x as usize];
    if y % 2 == 0 {
        c.set_col_bg(col);
        c.set(v);
    } else {
        c.set_col(col);
        c.set(v);
    }
}

pub fn sync_frame_buffer() {
    let cthread = std::thread::current();
    let mut wait_queue = match BIOS.wait_queue.lock() {
        Ok(x) => x,
        Err(e) => e.into_inner(),
    };
    wait_queue.push(cthread);
    drop(wait_queue);
    std::thread::park();
}

pub fn clear_frame_buffer() {
    for i in 0..BIOS.char_buffer.len() {
        for j in 0..BIOS.char_buffer[0].len() {
            BIOS.char_buffer[i][j].set(0 as char);
            BIOS.char_buffer[i][j].set_col(15);
            BIOS.char_buffer[i][j].set_col_bg(0);
        }
    }
}

pub fn clear_frame_buffer_color(c: u8) {
    for i in 0..BIOS.char_buffer.len() {
        for j in 0..BIOS.char_buffer[0].len() {
            BIOS.char_buffer[i][j].set(0 as char);
            BIOS.char_buffer[i][j].set_col(15);
            BIOS.char_buffer[i][j].set_col_bg(c);
        }
    }
}

pub fn draw_rectangle(x: u16, y: u16, w: u16, h: u16, col: u8) {
    for dy in y..y.wrapping_add(h) {
        for dx in x..x.wrapping_add(w) {
            draw_pixel(dx, dy, col);
        }
    }
}

pub fn draw_line(x0: u16, y0: u16, x1: u16, y1: u16, col: u8) {
    let len = (((x1 as i32 - x0 as i32) * (x1 as i32 - x0 as i32)
        + (y1 as i32 - y0 as i32) * (y1 as i32 - y0 as i32)) as f32)
        .sqrt();
    let dx = (x1 as i32 - x0 as i32) as f32 / len;
    let dy = (y1 as i32 - y0 as i32) as f32 / len;
    let ex = x0 as f32;
    let ey = y0 as f32;
    let func = |t: f32| (ex + dx * t, ey + dy * t);
    let mut dt = 0.0;
    let mut bx = x0 as i32;
    let mut by = y0 as i32;
    let squares_dist = |x: i32, y: i32, time: f32, func: &dyn Fn(f32) -> (f32, f32)| {
        let max_count = 10;
        let mut distance = 0.0;
        for delta_time in 0..max_count * 2 {
            let pos2 = func(time + (delta_time as f32) / (max_count as f32));
            let delt = (pos2.0 - x as f32) * (pos2.0 - x as f32)
                + (pos2.1 - y as f32) * (pos2.1 - y as f32);
            distance += delt;
        }
        distance.sqrt() / (max_count as f32 * 2.)
    };
    while dt < len {
        let mut nx = bx + 1;
        let mut ny = by + 1;
        let mut delta = squares_dist(nx, ny, dt, &func);
        let mut chose_x = 1;
        let mut chose_y = 1;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let tx = bx + dx;
                let ty = by + dy;
                let delt = squares_dist(tx, ty, dt, &func);
                if delt < delta {
                    chose_x = dx;
                    chose_y = dy;
                    nx = tx;
                    ny = ty;
                }
            }
        }
        let char = if chose_x == -1 {
            if chose_y == 1 {
                '/'
            } else if chose_y == 0 {
                '-'
            } else {
                '\\'
            }
        } else if chose_x == 0 {
            if chose_y == 1 {
                '|'
            } else if chose_y == 0 {
                '.'
            } else {
                '|'
            }
        } else {
            if chose_y == 1 {
                '\\'
            } else if chose_y == 0 {
                '-'
            } else {
                '/'
            }
        };
        dt += 1.0;
        if 0 <= nx && nx <= 80 && 0 <= ny && ny <= 45 {
            write_char_color(char, bx as u16, by as u16, col);
        }
        bx = nx;
        by = ny;
    }
}
