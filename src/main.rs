mod keyboard;
extern crate pdcurses;
extern crate pancurses;
// use pdcurses::{COLS, LINES};
// use std::ffi::CString;

// const SCREEN_WIDTH: i32 = 84;
// const SCREEN_HEIGHT: i32 = 20;

fn term_lines() -> i32 {
    unsafe {
        pancurses::LINES
    }
}

fn term_columns() -> i32 {
    unsafe {
        pancurses::COLS
    }
}

fn main() {
    unsafe {
        let window = pancurses::initscr();
        let mut keyboard_handler = keyboard::KeyboardHandler::new();
        let mut i = 0;

        loop {
            keyboard_handler.update();
            if keyboard::any_key_pressed(&keyboard_handler) {
                break;
            }
            i += 1;

            window.erase();
            pancurses::resize_term(0, 0);
            pancurses::curs_set(0);
            window.printw(format!("i = {}\n", i));
            window.printw(format!("COLS = {}, LINES = {}\n", term_columns(), term_lines()));
            window.refresh();
        }
        pancurses::endwin();
    }
}

// fn top_screen_margin() -> u32 {
//     unsafe {
//         ((pdcurses::LINES - SCREEN_HEIGHT) / 2) as u32
//     }
// }

// fn left_screen_margin() -> u32 {
//     unsafe {
//         ((pdcurses::COLS - SCREEN_WIDTH) / 2) as u32
//     }
// }
