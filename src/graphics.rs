use pancurses::Window;
use glam::Vec2;

const BOX_CHAR: char = '█';

pub trait WindowGraphics {
    fn draw_horizontal_line(&self, y: i32, x: i32, width: i32);
    fn draw_vertical_line(&self, y: i32, x: i32, height: i32);
    fn draw_line_segments(&self, line_segments: &Vec<Vec2>);
}

impl WindowGraphics for Window {
    fn draw_horizontal_line(&self, y: i32, x: i32, width: i32) {
        self.mv(y, x);
        self.hline(BOX_CHAR, width);
    }

    fn draw_vertical_line(&self, y: i32, x: i32, height: i32) {
        self.mv(y, x);
        self.vline(BOX_CHAR, height);
    }

    fn draw_line_segments(&self, line_segments: &Vec<Vec2>) {
        match line_segments.len() {
            0 => return,
            1 => {
                // if just one segment, draw it as a point
                let x = (line_segments[0].x).round() as i32;
                let y = (line_segments[0].y).round() as i32;
                self.draw_horizontal_line(y, x, 1);
            }
            len => {
                // draw out each len-1 segments
                for i in 0..len-1 {
                    let x = (line_segments[i].x).round() as i32;
                    let y = (line_segments[i].y).round() as i32;
                    let delta = line_segments[i+1] - line_segments[i];
                    if delta.y == 0.0 {
                        let sign_x = if delta.x < 0.0 {-1} else {1};
                        self.draw_horizontal_line(y, x, (delta.x + sign_x as f32) as i32);
                    } else {
                        let sign_y = if delta.y < 0.0 {-1} else {1};
                        self.draw_vertical_line(y, x, (delta.y + sign_y as f32) as i32);
                    }
                }
            }
        }
    }
}

// should this maybe be a struct?
pub const SCREEN_WIDTH: i32 = 84;
pub const SCREEN_HEIGHT: i32 = 20;
pub const BORDER_WIDTH: i32 = 80;
pub const BORDER_HEIGHT: i32 = 16;


pub fn top_screen_edge() -> i32 {
    ((term_lines() - SCREEN_HEIGHT) / 2) as i32
}

pub fn left_screen_edge() -> i32 {
    ((term_columns() - SCREEN_WIDTH) / 2) as i32
}

pub fn top_screen_margin() -> i32 {
    top_screen_edge() + (SCREEN_HEIGHT - BORDER_HEIGHT) / 2
}

pub fn left_screen_margin() -> i32 {
    left_screen_edge() + (SCREEN_WIDTH - BORDER_WIDTH) / 2
}

pub fn term_lines() -> i32 {
    unsafe {
        pancurses::LINES
    }
}

pub fn term_columns() -> i32 {
    unsafe {
        pancurses::COLS
    }
}
