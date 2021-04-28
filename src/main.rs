mod graphics;

use pancurses;
use platform;
use platform::virtual_keycodes;
use graphics::WindowGraphics;
use glam::Vec2;

macro_rules! vecvec2 {
    ($({$it1:expr, $it2:expr}),*) => {
        vec![$(
            glam::f32::vec2($it1, $it2)
        ),*]
    }
}

fn main() {
    /* Initialize */
    let window = pancurses::initscr();
    pancurses::curs_set(0);
    pancurses::noecho();
    // initialize colors
    pancurses::start_color();
    for color in 16..256 {
        pancurses::init_pair(color, color, pancurses::COLOR_BLACK);
    }

    /* Run program */
    let mut keyboard_handler = platform::keyboard::KeyboardHandler::new();
    // timing
    let mut prev_time = platform::timing::get_microsec_timestamp();
    let mut elapsed_frames = 0;

    let snake_body = vecvec2![{0.0, 0.0}, {2.0, 0.0}, {2.0, 2.0}, {4.0, 2.0}];

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
        window.erase();
        pancurses::curs_set(0);
        pancurses::resize_term(0, 0);
        let top_margin = graphics::top_screen_margin();
        let left_margin = graphics::left_screen_margin();
        // draw messages
        window.mvprintw(top_margin + 5, left_margin + 2, format!("elapsed_frames = {}\n", elapsed_frames));
        window.mvprintw(top_margin + 6, left_margin + 2, format!("COLS = {}, LINES = {}\n", graphics::term_columns(), graphics::term_lines()));
        window.mvprintw(top_margin + 7, left_margin + 2, format!("elapsed seconds = {}\n", elapsed_frames / 60));
        // draw window borders
        window.draw_horizontal_line(top_margin, left_margin, graphics::BORDER_WIDTH);
        window.draw_vertical_line(top_margin + 1, left_margin, graphics::BORDER_HEIGHT - 1);
        window.draw_vertical_line(top_margin + 1, left_margin + graphics::BORDER_WIDTH - 1, graphics::BORDER_HEIGHT - 1);
        window.draw_horizontal_line(top_margin + graphics::BORDER_HEIGHT - 1, left_margin, graphics::BORDER_WIDTH);

        // test draw some stuff
        window.attrset(pancurses::COLOR_PAIR(34));
        window.draw_line_segments(&shift_line_segments(&snake_body, left_margin + 2, top_margin + 1));
        window.attroff(pancurses::COLOR_PAIR(34));

        window.refresh();
    }
    pancurses::endwin();
}

/// used for "camera", moving line segments into the part of the screen we want
/// to draw them at based on their local x,y coordinates
fn shift_line_segments(line_segments: &Vec<Vec2>, x: i32, y: i32) -> Vec<Vec2> {
    line_segments.iter().map(|seg| glam::f32::vec2(seg.x + x as f32, seg.y + y as f32)).collect()
}

