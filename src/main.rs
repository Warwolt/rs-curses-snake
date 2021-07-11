mod graphics;
#[macro_use]
mod rectilinear;
mod attributes;

use glam::i32;
use graphics::WindowGraphics;
use pancurses;
use platform;
use platform::keyboard::KeyboardHandler;
use platform::virtual_keycodes;
use rectilinear::ChainedLineSegment;
use rectilinear::Direction;
use rectilinear::RectilinearLine;
use std::collections::VecDeque;

#[derive(Debug)]
struct ProgramState {
    elapsed_frames: usize,
    quit_requested: bool,
    keyboard_handler: KeyboardHandler,
    game_state: GameState,
}

#[derive(Debug)]
enum GameState {
    OngoingRound(RoundState),
    RoundEnd(RoundEndState),
    GameOver(GameOverState),
}

#[derive(Debug)]
struct RoundState {
    snake: SnakeState,
    game_over: bool,
    // apples
    // points
}

impl RoundState {
    fn new() -> Self {
        RoundState {
            snake: SnakeState::default(),
            game_over: false,
        }
    }
}

#[derive(Debug)]
struct RoundEndState {
    round: RoundState,
    frames: usize,
}

#[derive(Debug)]
struct SnakeState {
    movement_period: usize,
    body: RectilinearLine,
    direction: Direction,
    color: u64,
}

#[derive(Debug)]
struct GameOverState {
    selection: GameOverSelection,
}

#[derive(Debug, Clone, Copy)]
enum GameOverSelection {
    Restart,
    Exit,
}

impl SnakeState {
    fn new(body: RectilinearLine) -> Self {
        let direction = body.dir().unwrap();
        SnakeState {
            movement_period: 6,
            body,
            direction,
            color: 34, // green
        }
    }

    fn default() -> Self {
        SnakeState {
            movement_period: 6,
            body: RectilinearLine {
                start: i32::ivec2(0, 0),
                segments: VecDeque::from(vec![seg!(Direction::Right, 5)]),
            },
            color: 34,
            direction: Direction::Right,
        }
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

    /* Setup initial state */
    let mut prev_time = platform::timing::get_microsec_timestamp();
    let mut program_state = ProgramState {
        elapsed_frames: 0,
        quit_requested: false,
        keyboard_handler: KeyboardHandler::new(),
        game_state: GameState::OngoingRound(RoundState {
            snake: SnakeState::new(RectilinearLine {
                start: i32::ivec2(2, 2),
                segments: VecDeque::from(vec![
                    seg!(Direction::Right, 2),
                    seg!(Direction::Down, 2),
                    seg!(Direction::Right, 2),
                    seg!(Direction::Up, 4),
                    seg!(Direction::Right, 5),
                ]),
            }),
            game_over: false,
        }),
    };

    /* Run program */
    loop {
        // timing variables
        let time_now = platform::timing::get_microsec_timestamp();
        let elapsed_frame_time = time_now - prev_time;
        let frame_period_60_fps = (1e6 / 60.0) as i64;

        // run update at 60 fps
        if elapsed_frame_time > frame_period_60_fps {
            prev_time = time_now;
            program_state.elapsed_frames += 1;

            if program_state.quit_requested {
                break;
            }

            program_state = update(program_state);
            draw(&program_state, &window);
        }
    }

    pancurses::endwin();
}

fn update(mut program_state: ProgramState) -> ProgramState {
    /* Update inputs */
    let keyboard_handler = &mut program_state.keyboard_handler;
    keyboard_handler.update();
    if keyboard_handler.key_pressed_now(virtual_keycodes::VK_ESCAPE) {
        return ProgramState {
            quit_requested: true,
            ..program_state
        };
    }

    /* Run current state */
    let elapsed_frames = program_state.elapsed_frames;
    match program_state.game_state {
        GameState::OngoingRound(round) => {
            let next_round = run_ongoing_round(round, &keyboard_handler, elapsed_frames);
            program_state.game_state = if next_round.game_over {
                GameState::RoundEnd(RoundEndState {
                    round: next_round,
                    frames: 0,
                })
            } else {
                GameState::OngoingRound(next_round)
            }
        }
        GameState::RoundEnd(round_end_state) => {
            let next_round = run_round_ending(round_end_state);
            program_state.game_state = if next_round.frames < 80 {
                GameState::RoundEnd(next_round)
            } else {
                GameState::GameOver(GameOverState {
                    selection: GameOverSelection::Restart,
                })
            }
        }
        GameState::GameOver(game_over_state) => {
            let selection = if keyboard_handler.key_pressed_now(virtual_keycodes::VK_RIGHT) {
                GameOverSelection::Exit
            } else if keyboard_handler.key_pressed_now(virtual_keycodes::VK_LEFT) {
                GameOverSelection::Restart
            } else {
                game_over_state.selection
            };

            program_state.game_state =
                if keyboard_handler.key_pressed_now(virtual_keycodes::VK_RETURN) {
                    match selection {
                        GameOverSelection::Restart => GameState::OngoingRound(RoundState::new()),
                        GameOverSelection::Exit => {
                            program_state.quit_requested = true;
                            GameState::GameOver(GameOverState {
                                selection: GameOverSelection::Exit,
                            })
                        }
                    }
                } else {
                    GameState::GameOver(GameOverState { selection })
                }
        }
    }

    /* Return next state */
    ProgramState { ..program_state }
}

/// output the current program state to the window
fn draw(program_state: &ProgramState, window: &pancurses::Window) {
    window.clear();
    pancurses::curs_set(0);
    pancurses::resize_term(0, 0);

    match &program_state.game_state {
        GameState::OngoingRound(round_state) => {
            draw_ongoing_round(round_state, &window);
        }
        GameState::RoundEnd(end_state) => {
            draw_ongoing_round(&end_state.round, &window);
        }
        GameState::GameOver(game_over_state) => {
            draw_game_over_screen(&game_over_state, &window);
        }
    }

    window.refresh();
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

fn run_ongoing_round(
    mut state: RoundState,
    keyboard_handler: &KeyboardHandler,
    elapsed_frames: usize,
) -> RoundState {
    let snake = &mut state.snake;

    // update movement variables
    if let Some(new_direction) = get_direction(&keyboard_handler) {
        // only allow turning 90 degrees, not 180
        if new_direction != snake.direction.opposite() {
            snake.direction = new_direction;
        }
    }

    // move snake body
    if elapsed_frames % snake.movement_period == 0 {
        snake.body.move_forward(snake.direction);
    }

    // check if overlapped
    if snake.body.is_self_overlapping() {
        state.game_over = true;
    }

    state
}

fn run_round_ending(mut state: RoundEndState) -> RoundEndState {
    state.frames += 1;
    let blink_period = 5;
    let elapsed_periods = state.frames / blink_period;

    state.round.snake.color = if elapsed_periods < 8 {
        /* Blink a few times */
        if elapsed_periods % 2 == 0 {
            88 // red
        } else {
            34 // green
        }
    } else {
        /* Remain red */
        88
    };
    state
}

fn draw_ongoing_round(state: &RoundState, window: &pancurses::Window) {
    let top_margin = graphics::top_screen_margin();
    let left_margin = graphics::left_screen_margin();

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
    let snake = &state.snake;
    window.attrset(pancurses::COLOR_PAIR(snake.color));
    draw_snake(&window, &snake.body);
    window.attroff(pancurses::COLOR_PAIR(snake.color));
}

fn draw_game_over_screen(state: &GameOverState, window: &pancurses::Window) {
    let (mx, my) = graphics::screen_middle();
    let attrs = match state.selection {
        GameOverSelection::Restart => (attributes::A_REVERSE, attributes::A_NORMAL),
        GameOverSelection::Exit => (attributes::A_NORMAL, attributes::A_REVERSE),
    };

    let game_over = "Game Over";
    window.mvprintw(my - 1, mx - game_over.len() as i32 / 2, game_over);

    window.attron(attrs.0);
    window.mvprintw(my + 2, mx - 7, "Restart");
    window.attroff(attrs.0);

    window.attron(attrs.1);
    window.mvprintw(my + 2, mx + 3, "Exit");
    window.attroff(attrs.1);
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
