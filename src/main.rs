mod graphics;

use pancurses;
use platform;
use platform::keyboard::{KeyboardHandler};
use platform::virtual_keycodes;
use graphics::WindowGraphics;
use glam::IVec2;
use glam::f32;
use glam::i32;

enum Direction {Up, Left, Down, Right}

macro_rules! vecivec2 {
    ($(($it1:expr, $it2:expr)),*) => {
        vec![$(
            i32::ivec2($it1, $it2)
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
    let mut keyboard_handler = KeyboardHandler::new();
    // timing
    let mut prev_time = platform::timing::get_microsec_timestamp();
    let mut elapsed_frames = 0;
    // snake
    let mut snake_body = vecivec2![(0, 0)];

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
        window.erase(); // erase here so we can debug print

        /* Update */
        keyboard_handler.update();
        if keyboard_handler.key_pressed_now(virtual_keycodes::VK_ESCAPE) {
            break;
        }
        // move head if direction key pressed
        if let Some(dir) = get_direction(&keyboard_handler) {
            snake_body[0] += translation_vec(dir, 500);
        }

        /* Draw */
        pancurses::curs_set(0);
        pancurses::resize_term(0, 0);
        let top_margin = graphics::top_screen_margin();
        let left_margin = graphics::left_screen_margin();
        // draw messages
        let messages = [
            format!("elapsed_frames = {}\n", elapsed_frames),
            format!("COLS = {}, LINES = {}\n", graphics::term_columns(), graphics::term_lines()),
            format!("pos = ({}, {})", snake_body[0].x as f32 / 1000.0, snake_body[0].y as f32 / 1000.0)];
        for i in 0..messages.len() {
            window.mvprintw(top_margin + 5 + i as i32, left_margin + 2, &messages[i]);
        }
        // draw window borders
        window.draw_horizontal_line(top_margin + 0, left_margin + 0, graphics::BORDER_WIDTH);
        window.draw_vertical_line(top_margin + 1, left_margin + 0, graphics::BORDER_HEIGHT - 1);
        window.draw_vertical_line(top_margin + 1, left_margin + graphics::BORDER_WIDTH - 1, graphics::BORDER_HEIGHT - 1);
        window.draw_horizontal_line(top_margin + graphics::BORDER_HEIGHT - 1, left_margin, graphics::BORDER_WIDTH);

        // draw snake
        window.attrset(pancurses::COLOR_PAIR(34));
        fn scale_segment(seg: &i32::IVec2) -> i32::IVec2 {
            i32::ivec2((seg.x as f32 / 1000.0).round() as i32,
                (seg.y as f32 / 1000.0).round() as i32)
        }
        let top_border = top_margin + 1;
        let left_border = left_margin + 1;
        let scaled_body = snake_body.iter().map(scale_segment).collect::<Vec<_>>();
        let shifted_body = shift_line_segments(&scaled_body, left_border, top_border);
        window.draw_line_segments(&shifted_body);
        window.attroff(pancurses::COLOR_PAIR(34));

        window.refresh();
    }
    pancurses::endwin();
}

/// Used for "camera", moving line segments into the part of the screen we want
/// to draw them at based on their local x,y coordinates
fn shift_line_segments(line_segments: &Vec<IVec2>, x: i32, y: i32) -> Vec<IVec2> {
    line_segments.iter().map(|seg| i32::ivec2(seg.x + x, seg.y + y)).collect()
}

/// Get which direction key is pressed, if any
fn get_direction(keyboard_handler: &KeyboardHandler) -> Option<Direction> {
    // todo extract keycodes into a lookup table parameter?
    if keyboard_handler.key_pressed_now(virtual_keycodes::VK_RIGHT) {
        Some(Direction::Right)
    } else if keyboard_handler.key_pressed_now(virtual_keycodes::VK_LEFT) {
        Some(Direction::Left)
    } else if keyboard_handler.key_pressed_now(virtual_keycodes::VK_UP) {
        Some(Direction::Up)
    } else if keyboard_handler.key_pressed_now(virtual_keycodes::VK_DOWN) {
        Some(Direction::Down)
    } else {
        None
    }
}

fn translation_vec(dir: Direction, x_speed: i32) -> IVec2 {
    // since characters are taller than they are wide we must scale the y-axis
    // to get consistent movement speed in all directions.
    let y_speed = (x_speed as f32 * 11.0/24.0).round() as i32;
    match dir {
        Direction::Right => i32::ivec2(x_speed, 0),
        Direction::Left => i32::ivec2(-x_speed, 0),
        Direction::Down => i32::ivec2(0, y_speed),
        Direction::Up => i32::ivec2(0, -y_speed),
    }
}

