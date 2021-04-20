mod keyboard; // put this in a lib to get rid off unused warnings?
extern crate pdcurses;
use pdcurses::{COLS, LINES};

fn main() {
    unsafe {
        let window = pdcurses::initscr();
        let mut keyboard_handler = keyboard::KeyboardHandler::new();
        let mut i = 0;

        loop {
            keyboard_handler.update();
            if keyboard::any_key_pressed(&keyboard_handler) {
                break;
            }
            i += 1;

            // Why does this not behave AT ALL like the reference C++ program?
            // Does the ref program differ in version of PDCurses? Is the
            // bindings we're using just broken somehow?
            pdcurses::werase(window);
            pdcurses::resize_term(0, 0);
            pdcurses::curs_set(0);
            pdcurses::wprintw(window, format!("i = {}\n\0", i).as_ptr() as *const i8);
            pdcurses::wprintw(window, format!("COLS = {}, LINES = {}\n\0", COLS, LINES).as_ptr() as *const i8);
            pdcurses::wrefresh(window);
        }
        pdcurses::endwin();
    }
}

