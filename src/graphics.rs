use pancurses::Window;

const BOX_CHAR: char = '█';

pub trait WindowGraphics {
    fn draw_horizontal_line(&self, y: i32, x: i32, width: i32);
    fn draw_vertical_line(&self, y: i32, x: i32, height: i32);
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
}
