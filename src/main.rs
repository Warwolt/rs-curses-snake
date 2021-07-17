// TODO:
// [X] fix responsiveness in turning snake
// [X] make play area less wide (use 3310 dimensions?)
// [] add collision with walls

mod graphics;
#[macro_use]
mod rectilinear;
mod attributes;

use glam::i32;
use glam::IVec2;
use graphics::WindowGraphics;
use pancurses;
use platform;
use platform::keyboard::KeyboardHandler;
use platform::virtual_keycodes;
use rand::distributions::{Distribution, Uniform};
use rectilinear::ChainedLineSegment;
use rectilinear::Direction;
use rectilinear::RectilinearLine;
use std::collections::VecDeque;

#[derive(Debug)]
struct ProgramState {
    elapsed_frames: usize,
    quit_requested: bool,
    keyboard_handler: KeyboardHandler,
    ivec2_gen: IVec2Generator,
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
    apple: IVec2, // points
    wall: RectilinearLine,
    game_over: bool,
}

impl RoundState {
    fn new() -> Self {
        RoundState {
            snake: SnakeState::new(),
            apple: i32::ivec2(0, 0),
            wall: new_play_area_wall(),
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
    movement_frames: usize,
    turn_cooldown: usize,
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

/// Used for generating the position of the apples
#[derive(Debug)]
struct IVec2Generator {
    rng: rand::prelude::ThreadRng,
    x_dist: Uniform<i32>,
    y_dist: Uniform<i32>,
}

impl IVec2Generator {
    fn gen_ivec2(&mut self) -> IVec2 {
        i32::ivec2(
            self.x_dist.sample(&mut self.rng),
            self.y_dist.sample(&mut self.rng),
        )
    }
}

impl SnakeState {
    fn new() -> Self {
        SnakeState {
            movement_period: 6,
            body: RectilinearLine {
                start: i32::ivec2(graphics::screen_middle().0 / 2, 3),
                segments: VecDeque::from(vec![seg!(Direction::Down, 3)]),
            },
            color: 34,
            direction: Direction::Right,
            movement_frames: 0,
            turn_cooldown: 0,
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
    // random number generation
    let mut ivec2_gen = IVec2Generator {
        rng: rand::thread_rng(),
        x_dist: Uniform::from(1..graphics::BORDER_WIDTH - 3),
        y_dist: Uniform::from(1..graphics::BORDER_HEIGHT - 3),
    };

    /* Setup initial state */
    let mut prev_time = platform::timing::get_microsec_timestamp();
    let snake = SnakeState::new();
    let apple = generate_apple(&mut ivec2_gen, &snake.body);
    let mut program_state = ProgramState {
        elapsed_frames: 0,
        quit_requested: false,
        keyboard_handler: KeyboardHandler::new(),
        ivec2_gen,
        game_state: GameState::OngoingRound(RoundState {
            snake,
            apple,
            wall: new_play_area_wall(),
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
    let ivec2_gen = &mut program_state.ivec2_gen;
    match program_state.game_state {
        GameState::OngoingRound(round) => {
            let next_round = run_ongoing_round(round, &keyboard_handler, ivec2_gen);
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
    round: RoundState,
    keyboard_handler: &KeyboardHandler,
    ivec2_gen: &mut IVec2Generator,
) -> RoundState {
    let mut next_round = RoundState { ..round };
    let mut snake = &mut next_round.snake;

    // track frames
    snake.movement_frames += 1;
    snake.turn_cooldown = snake.turn_cooldown.saturating_sub(1);

    // turn sideways
    if let Some(new_direction) = get_direction(&keyboard_handler) {
        // make sure we're turning 90 degrees only, and not too often
        if new_direction != snake.direction.opposite() && snake.turn_cooldown == 0 {
            snake.direction = new_direction;
            snake.movement_frames = snake.movement_period;
            // add a delay to when next turn can happen, to prevent from moving
            // very fast when moving in a diagonal
            snake.turn_cooldown = snake.movement_period / 2;
        }
    }

    // move snake body
    if snake.movement_frames == snake.movement_period {
        // check if about to hit a wall
        let head_plus_one = snake.body.head() + snake.direction.unit();
        if next_round.wall.collides_with_point(head_plus_one) {
            next_round.game_over = true;
            return next_round;
        }

        snake.body.move_forward(snake.direction);
        snake.movement_frames = 0;
    }

    // check if collided with self
    if snake.body.is_self_overlapping() {
        next_round.game_over = true;
        return next_round;
    }

    // check if collision with apple
    if snake.body.head() == round.apple {
        // eat the apple
        snake.body.extend_tail();

        // make new apple
        next_round.apple = generate_apple(ivec2_gen, &snake.body);
    }

    next_round
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
    draw_wall(&window, &state.wall);
    draw_snake(&window, &state.snake);
    draw_apple(&window, state.apple);
}

fn draw_game_over_screen(state: &GameOverState, window: &pancurses::Window) {
    let (mx, my) = graphics::screen_middle();
    let attrs = match state.selection {
        GameOverSelection::Restart => (attributes::A_REVERSE, attributes::A_NORMAL),
        GameOverSelection::Exit => (attributes::A_NORMAL, attributes::A_REVERSE),
    };

    let game_over = "Game Over";
    window.mvprintw(my - 2, mx - game_over.len() as i32 / 2, game_over);

    window.attron(attrs.0);
    window.mvprintw(my + 1, mx - 7, "Restart");
    window.attroff(attrs.0);

    window.attron(attrs.1);
    window.mvprintw(my + 1, mx + 3, "Exit");
    window.attroff(attrs.1);
}

fn draw_snake(window: &pancurses::Window, snake: &SnakeState) {
    draw_rectilinear_line(window, &snake.body, snake.color);
}

fn draw_wall(window: &pancurses::Window, wall: &RectilinearLine) {
    draw_rectilinear_line(window, wall, 1);
}

fn draw_rectilinear_line(window: &pancurses::Window, line: &RectilinearLine, color: u64) {
    window.attron(pancurses::COLOR_PAIR(color));

    let mut x = graphics::left_screen_margin() + 1 + line.start.x;
    let mut y = graphics::top_screen_margin() + 1 + line.start.y;

    if line.len() == 1 {
        window.draw_horizontal_line(y, x, 1);
        window.attroff(pancurses::COLOR_PAIR(color));
        return;
    }

    for segment in &line.segments {
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
    window.attroff(pancurses::COLOR_PAIR(color));
}

fn draw_apple(window: &pancurses::Window, apple: IVec2) {
    let x = graphics::left_screen_margin() + 1 + apple.x;
    let y = graphics::top_screen_margin() + 1 + apple.y;

    window.attron(pancurses::COLOR_PAIR(88)); // red
    window.draw_horizontal_line(y, x, 1);
    window.attroff(pancurses::COLOR_PAIR(88)); // red
}

/// Creates a new apple using `generator`, while avoiding having it overlapping
/// with the `snake_body`
fn generate_apple(generator: &mut IVec2Generator, snake_body: &RectilinearLine) -> IVec2 {
    loop {
        let point = generator.gen_ivec2();
        if snake_body.collides_with_point(point) {
            continue;
        } else {
            break point;
        }
    }
}

/// Create the wall that surrounds the play area
fn new_play_area_wall() -> RectilinearLine {
    RectilinearLine {
        start: i32::ivec2(-1, -1), // we surround the play area, so we start at (-1,-1)
        segments: VecDeque::from(vec![
            seg!(Direction::Right, graphics::BORDER_WIDTH as usize - 1),
            seg!(Direction::Down, graphics::BORDER_HEIGHT as usize - 1),
            seg!(Direction::Left, graphics::BORDER_WIDTH as usize - 1),
            seg!(Direction::Up, graphics::BORDER_HEIGHT as usize - 2),
        ]),
    }
}
