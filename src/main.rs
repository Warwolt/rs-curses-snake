mod graphics;
#[macro_use]
mod rectilinear;

use glam::i32;
use glam::IVec2;
use graphics::WindowGraphics;
use pancurses;
use platform;
use platform::keyboard::KeyboardHandler;
use platform::virtual_keycodes;
use rectilinear::ChainedLineSegment;
use rectilinear::Direction;
use rectilinear::RectilinearLine;
use std::collections::VecDeque;

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
    let movement_period = 6;
    let mut snake_body = RectilinearLine {
        start: i32::ivec2(2, 2),
        segments: VecDeque::from(vec![
            seg!(Direction::Right, 2),
            seg!(Direction::Down, 2),
            seg!(Direction::Right, 2),
            seg!(Direction::Up, 4),
            seg!(Direction::Right, 5),
        ]),
    };
    let mut current_dir = snake_body.dir().unwrap();

    loop {
        // Check if enough time has elapsed to run the next frame, if not
        // enough has elapsed then skip rest of the game loop
        let time_now = platform::timing::get_microsec_timestamp();
        let elapsed_frame_time = time_now - prev_time;
        if elapsed_frame_time <= (1e6 / 60.0) as i64 {
            continue;
        }
        prev_time = time_now;
        elapsed_frames += 1;

        window.erase(); // erasing here so we can debug print during update

        /* Update */
        keyboard_handler.update();
        if keyboard_handler.key_pressed_now(virtual_keycodes::VK_ESCAPE) {
            break; // quit program if escape is pressed
        }

        // update movement variables
        if let Some(dir) = get_direction(&keyboard_handler) {
            // only allow turning 90 degrees, not 180
            if dir != current_dir.opposite() {
                current_dir = dir;
            }
        }
        // move snake body
        if elapsed_frames % movement_period == 0 {
            snake_body.move_forward(current_dir)
        }

        /* Draw */
        pancurses::curs_set(0);
        pancurses::resize_term(0, 0);
        let top_margin = graphics::top_screen_margin();
        let left_margin = graphics::left_screen_margin();
        // draw messages
        let messages = [
            format!("elapsed_frames = {}\n", elapsed_frames),
            format!(
                "COLS = {}, LINES = {}\n",
                graphics::term_columns(),
                graphics::term_lines()
            ),
            format!("direction = {:?}", current_dir),
        ];
        for i in 0..messages.len() {
            window.mvprintw(top_margin + 5 + i as i32, left_margin + 2, &messages[i]);
        }
        // draw window borders
        window.draw_horizontal_line(top_margin + 0, left_margin + 0, graphics::BORDER_WIDTH);
        window.draw_vertical_line(top_margin + 1, left_margin + 0, graphics::BORDER_HEIGHT - 1);
        window.draw_vertical_line(
            top_margin + 1,
            left_margin + graphics::BORDER_WIDTH - 1,
            graphics::BORDER_HEIGHT - 1,
        );
        window.draw_horizontal_line(
            top_margin + graphics::BORDER_HEIGHT - 1,
            left_margin,
            graphics::BORDER_WIDTH,
        );

        // draw snake
        let snake_color = if snake_body.is_self_overlapping() { 88 } else { 34 };
        window.attrset(pancurses::COLOR_PAIR(snake_color));
        draw_snake(&window, &snake_body);
        window.attroff(pancurses::COLOR_PAIR(snake_color));

        window.refresh();
    }
    pancurses::endwin();
}

fn draw_snake(window: &pancurses::Window, snake_body: &RectilinearLine) {
    let mut x = graphics::left_screen_margin() + 1 + snake_body.start.x;
    let mut y = graphics::top_screen_margin() + 2 + snake_body.start.y;

    if snake_body.len() == 1 {
        window.draw_horizontal_line(y, x, 1);
        return;
    }

    for segment in &snake_body.segments {
        let len = segment.len as i32;
        match segment.dir {
            Direction::Up => {
                window.draw_vertical_line(y - len, x, len + 1);
                y -= len;
            }
            Direction::Down => {
                window.draw_vertical_line(y, x, len + 1);
                y += len;
            }
            Direction::Left => {
                window.draw_horizontal_line(y, x - len, len + 1);
                x -= len;
            }
            Direction::Right => {
                window.draw_horizontal_line(y, x, len + 1);
                x += len;
            }
        }
    }
}

/// Used for "camera", moving line segments into the part of the screen we want
/// to draw them at based on their local x,y coordinates
fn _shift_line_segments(line_segments: &Vec<IVec2>, x: i32, y: i32) -> Vec<IVec2> {
    line_segments
        .iter()
        .map(|seg| i32::ivec2(seg.x + x, seg.y + y))
        .collect()
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
