mod keyboard;
extern crate pancurses;
use pancurses::{initscr, endwin};

fn main() {
    let mut keyboard_handler = keyboard::KeyboardHandler::new();
    let window = initscr();
    window.printw("Hello Rust");
    window.refresh();
    loop {
        keyboard_handler.update();
        if keyboard::any_key_pressed(&keyboard_handler) {
            break;
        }
    }
    endwin();
}

