use pancurses;
use platform;
use winapi;
use winapi::shared::ntdef::LARGE_INTEGER;
use winapi::um::profileapi::{QueryPerformanceFrequency, QueryPerformanceCounter};
use platform::virtual_keycodes;

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

fn get_perf_counter_freq() -> i64 {
    unsafe {
        let mut perf_counter_freq = LARGE_INTEGER::default();
        QueryPerformanceFrequency(&mut perf_counter_freq);
        *perf_counter_freq.QuadPart()
    }
}

fn get_perf_counter_ticks() -> i64 {
    unsafe {
        let mut perf_counter_freq = LARGE_INTEGER::default();
        QueryPerformanceCounter(&mut perf_counter_freq);
        *perf_counter_freq.QuadPart()

    }
}

fn get_microsec_timestamp() -> i64 {
    let perf_counter_freq = get_perf_counter_freq();
    let current_ticks = get_perf_counter_ticks();
    let ticks_scaled_by_megahz = current_ticks * (1e6 as i64);
    let microsec_ticks = ticks_scaled_by_megahz / perf_counter_freq;
    microsec_ticks
}

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
    let mut prev_time = get_microsec_timestamp();
    let mut elapsed_frames = 0;

    loop {
        // Check if enough time has elapsed to run the next frame, if not
        // enough has elapsed then skip rest of the game loop
        let time_now = get_microsec_timestamp();
        let elapsed_frame_time = time_now - prev_time;
        if elapsed_frame_time <= (1e6/60.0) as i64 {
            continue;
        }
        prev_time = time_now;
        elapsed_frames += 1;

        keyboard_handler.update();
        if keyboard_handler.key_pressed_now(virtual_keycodes::VK_ESCAPE) {
            break;
        }

        pancurses::resize_term(0, 0);
        pancurses::curs_set(0);
        window.erase();
        window.printw(format!("elapsed_frames = {}\n", elapsed_frames));
        window.printw(format!("COLS = {}, LI2NES = {}\n", term_columns(), term_lines()));
        window.refresh();
    }
    pancurses::endwin();
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
