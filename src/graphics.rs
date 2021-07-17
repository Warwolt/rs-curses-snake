use glam::IVec2;
use pancurses::Window;

const BOX_CHAR: char = '█';

pub trait WindowGraphics {
    fn draw_horizontal_line(&self, y: i32, x: i32, width: i32);
    fn draw_vertical_line(&self, y: i32, x: i32, height: i32);
    fn draw_line_segments(&self, line_segments: &Vec<IVec2>);
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

    fn draw_line_segments(&self, line_segments: &Vec<IVec2>) {
        match line_segments.len() {
            // if empty, nothing to draw
            0 => return,
            // if just one segment, draw it as a point
            1 => {
                let x = line_segments[0].x;
                let y = line_segments[0].y;
                self.draw_horizontal_line(y, x, 1);
            }
            // else draw out each len-1 segments
            len => {
                for i in 0..len - 1 {
                    let x = line_segments[i].x;
                    let y = line_segments[i].y;
                    let delta = line_segments[i + 1] - line_segments[i];
                    if delta.y == 0 {
                        let sign_x = if delta.x < 0 { -1 } else { 1 };
                        self.draw_horizontal_line(y, x, delta.x + sign_x);
                    } else {
                        let sign_y = if delta.y < 0 { -1 } else { 1 };
                        self.draw_vertical_line(y, x, delta.y + sign_y);
                    }
                }
            }
        }
    }
}

// should this maybe be a struct?
pub const SCREEN_WIDTH: i32 = 84;
pub const SCREEN_HEIGHT: i32 = 20;
pub const BORDER_HEIGHT: i32 = 12;
pub const BORDER_WIDTH: i32 = 5 * BORDER_HEIGHT;

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

/// Returns the middle screen (x, y) coordinate
pub fn screen_middle() -> (i32, i32) {
    (
        left_screen_margin() + BORDER_WIDTH / 2,
        top_screen_margin() + BORDER_HEIGHT / 2,
    )
}

pub fn term_lines() -> i32 {
    unsafe { pancurses::LINES }
}

pub fn term_columns() -> i32 {
    unsafe { pancurses::COLS }
}
