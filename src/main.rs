mod graphics;

use pancurses;
use platform;
use platform::virtual_keycodes;
use graphics::WindowGraphics;

const SCREEN_WIDTH: i32 = 84;
const SCREEN_HEIGHT: i32 = 20;
const BORDER_WIDTH: i32 = 80;
const BORDER_HEIGHT: i32 = 16;

fn main() {
    /* Initialize */
    let window = pancurses::initscr();
    pancurses::noecho();
    pancurses::start_color();
    for color in 16..256 {
        let color_black = 0;
        pancurses::init_pair(color, color, color_black);
    }

    /* Run program */
    let mut keyboard_handler = platform::keyboard::KeyboardHandler::new();
    // timing
    let mut prev_time = platform::timing::get_microsec_timestamp();
    let mut elapsed_frames = 0;

    loop {
        // Check if enough time has elapsed to run the next frame, if not
        // enough has elapsed then skip rest of the game loop
        let time_now = platform::timing::get_microsec_timestamp();
        let elapsed_frame_time = time_now - prev_time;
        if elapsed_frame_time <= (1e6/60.0) as i64 {
            continue;
        }
        prev_time = time_now;
        elapsed_frames += 1;

        /* Update */
        keyboard_handler.update();
        if keyboard_handler.key_pressed_now(virtual_keycodes::VK_ESCAPE) {
            break;
        }

        /* Draw */
        pancurses::resize_term(0, 0);
        pancurses::curs_set(0);
        window.erase();
        let top_margin = top_screen_edge() + (SCREEN_HEIGHT - BORDER_HEIGHT) / 2;
        let left_margin = left_screen_edge() + (SCREEN_WIDTH - BORDER_WIDTH) / 2;
        // draw messages
        window.mvprintw(top_margin + 5, left_margin + 2, format!("elapsed_frames = {}\n", elapsed_frames));
        window.mvprintw(top_margin + 6, left_margin + 2, format!("COLS = {}, LINES = {}\n", term_columns(), term_lines()));
        // draw window borders
        window.draw_horizontal_line(top_margin, left_margin, BORDER_WIDTH);
        window.draw_vertical_line(top_margin + 1, left_margin, BORDER_HEIGHT - 1);
        window.draw_vertical_line(top_margin + 1, left_margin + BORDER_WIDTH - 1, BORDER_HEIGHT - 1);
        window.draw_horizontal_line(top_margin + BORDER_HEIGHT - 1, left_margin, BORDER_WIDTH);

        window.refresh();
    }
    pancurses::endwin();
}

fn top_screen_edge() -> i32 {
    ((term_lines() - SCREEN_HEIGHT) / 2) as i32
}

fn left_screen_edge() -> i32 {
    ((term_columns() - SCREEN_WIDTH) / 2) as i32
}

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
