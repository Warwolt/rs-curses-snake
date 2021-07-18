// TODO
// [x] show final score on game over screen
// [x] add main menu with difficulty options and logo
// [x] add round start state that displays "get ready!" a few frames
// [ ] add a brief "good bye!" screen on exit
// [ ] add pause menu
// [ ] generic menu infrastructure (i.e. not hard coded menus)
// [ ] fix bug where tail can extend into body when eating apples

mod graphics;
#[macro_use]
mod rectilinear;
mod attributes;
mod menu;

use enum_iterator::IntoEnumIterator;
use glam::i32;
use glam::IVec2;
use graphics::WindowGraphics;
use menu::ItemList;
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
    StartMenu(StartMenuState),
    RoundStart(RoundStartState),
    OngoingRound(RoundState),
    RoundEnd(RoundEndState),
    GameOver(GameOverState),
    ProgramExit(usize), // frames
}

#[derive(Debug)]
struct StartMenuState {
    focused_area: StartMenuArea,
    menu_items: menu::ItemList<StartMenuItem>,
    difficulty_items: menu::ItemList<GameDifficulty>,
    difficulty: GameDifficulty,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum StartMenuArea {
    Main,
    Difficulty,
}

#[derive(Debug, Copy, Clone, IntoEnumIterator)]
enum StartMenuItem {
    Start,
    Difficulty,
    Exit,
}

#[derive(Debug, Copy, Clone, IntoEnumIterator)]
enum GameDifficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, PartialEq)]
enum QuitRequested {
    Yes,
    No,
}

#[derive(Debug, PartialEq)]
enum ExitMenu {
    Yes,
    No,
}

#[derive(Debug)]
struct RoundStartState {
    frames: usize,
    difficulty: GameDifficulty,
}

#[derive(Debug)]
struct RoundState {
    snake: SnakeState,
    apple: IVec2,
    wall: RectilinearLine,
    score: usize,
    game_over: bool,
    difficulty: GameDifficulty,
}

impl RoundState {
    fn new(generator: &mut IVec2Generator, difficulty: GameDifficulty) -> Self {
        let mut snake = SnakeState::new();
        snake.movement_period = match difficulty {
            GameDifficulty::Easy => 8,
            GameDifficulty::Normal => 6,
            GameDifficulty::Hard => 4,
        };
        let apple = generate_apple(generator, &snake.body);
        RoundState {
            snake,
            apple,
            wall: new_play_area_wall(),
            score: 0,
            game_over: false,
            difficulty,
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
    final_score: usize,
    difficulty: GameDifficulty,
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
        let body = RectilinearLine {
            start: i32::ivec2(graphics::screen_middle().0 / 2, 0),
            segments: VecDeque::from(vec![seg!(Direction::Down, 3)]),
        };
        let direction = body.dir().unwrap();
        SnakeState {
            movement_period: 6,
            body,
            color: 34,
            direction,
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
    let ivec2_gen = IVec2Generator {
        rng: rand::thread_rng(),
        x_dist: Uniform::from(1..graphics::BORDER_WIDTH - 3),
        y_dist: Uniform::from(1..graphics::BORDER_HEIGHT - 3),
    };

    /* Setup initial state */
    let mut prev_time = platform::timing::get_microsec_timestamp();
    let mut program_state = ProgramState {
        elapsed_frames: 0,
        quit_requested: false,
        keyboard_handler: KeyboardHandler::new(),
        ivec2_gen,
        game_state: GameState::StartMenu(StartMenuState {
            focused_area: StartMenuArea::Main,
            menu_items: ItemList::new(StartMenuItem::into_enum_iter(), 0),
            difficulty_items: ItemList::new(GameDifficulty::into_enum_iter(), 1),
            difficulty: GameDifficulty::Normal,
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
        GameState::StartMenu(menu_state) => {
            if menu_state.focused_area == StartMenuArea::Main {
                let (menu_state, selected_item) = run_start_menu(menu_state, &keyboard_handler);
                let (game_state, quit) = transition_start_menu(menu_state, selected_item);
                program_state.game_state = if quit == QuitRequested::Yes {
                    GameState::ProgramExit(0)
                } else {
                    game_state
                }
            } else {
                let (menu_state, exit) = run_difficulty_menu(menu_state, &keyboard_handler);
                program_state.game_state = GameState::StartMenu(StartMenuState {
                    focused_area: if exit == ExitMenu::Yes {
                        StartMenuArea::Main
                    } else {
                        StartMenuArea::Difficulty
                    },
                    ..menu_state
                })
            }
        }
        GameState::RoundStart(start_state) => {
            let mut next_start_state = start_state;
            let wait_period = 90; // frames
            next_start_state.frames += 1;
            program_state.game_state = if next_start_state.frames > wait_period {
                GameState::OngoingRound(RoundState::new(ivec2_gen, next_start_state.difficulty))
            } else {
                GameState::RoundStart(next_start_state)
            };
        }
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
                    final_score: next_round.round.score,
                    selection: GameOverSelection::Restart,
                    difficulty: next_round.round.difficulty,
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
                        GameOverSelection::Restart => {
                            let generator = &mut program_state.ivec2_gen;
                            let difficulty = game_over_state.difficulty;
                            GameState::OngoingRound(RoundState::new(generator, difficulty))
                        }
                        GameOverSelection::Exit => {
                            GameState::ProgramExit(0)
                        }
                    }
                } else {
                    GameState::GameOver(GameOverState {
                        selection,
                        ..game_over_state
                    })
                }
        }
        GameState::ProgramExit(frames) => {
            let wait_period = 30; // frames
            if frames + 1 > wait_period {
                program_state.quit_requested = true;
            }
            program_state.game_state = GameState::ProgramExit(frames + 1);
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
        GameState::StartMenu(menu_state) => {
            draw_start_menu(&menu_state, &window);
        }
        GameState::RoundStart(_) => {
            draw_round_start(&window);
        }
        GameState::OngoingRound(round_state) => {
            draw_ongoing_round(round_state, &window);
        }
        GameState::RoundEnd(end_state) => {
            draw_ongoing_round(&end_state.round, &window);
        }
        GameState::GameOver(game_over_state) => {
            draw_game_over_screen(&game_over_state, &window);
        }
        GameState::ProgramExit(_) => {
            draw_program_exit(&window);
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

fn run_start_menu(
    mut menu_state: StartMenuState,
    keyboard_handler: &KeyboardHandler,
) -> (StartMenuState, Option<StartMenuItem>) {
    if keyboard_handler.key_pressed_now(virtual_keycodes::VK_UP) {
        menu_state.menu_items.move_back();
    }

    if keyboard_handler.key_pressed_now(virtual_keycodes::VK_DOWN) {
        menu_state.menu_items.move_forward();
    }

    let selected_item = if keyboard_handler.key_pressed_now(virtual_keycodes::VK_RETURN) {
        Some(menu_state.menu_items.current_item())
    } else {
        None
    };

    (menu_state, selected_item)
}

fn run_difficulty_menu(
    mut menu_state: StartMenuState,
    keyboard_handler: &KeyboardHandler,
) -> (StartMenuState, ExitMenu) {
    if keyboard_handler.key_pressed_now(virtual_keycodes::VK_LEFT) {
        menu_state.difficulty_items.move_back();
    }

    if keyboard_handler.key_pressed_now(virtual_keycodes::VK_RIGHT) {
        menu_state.difficulty_items.move_forward();
    }

    menu_state.difficulty = menu_state.difficulty_items.current_item();

    let menu_return = if keyboard_handler.key_pressed_now(virtual_keycodes::VK_RETURN) {
        ExitMenu::Yes
    } else {
        ExitMenu::No
    };

    (menu_state, menu_return)
}

fn transition_start_menu(
    next_state: StartMenuState,
    selected_item: Option<StartMenuItem>,
) -> (GameState, QuitRequested) {
    match selected_item {
        Some(selected_item) => match selected_item {
            StartMenuItem::Start => (
                GameState::RoundStart(RoundStartState {
                    frames: 0,
                    difficulty: next_state.difficulty,
                }),
                QuitRequested::No,
            ),
            StartMenuItem::Difficulty => (
                GameState::StartMenu(StartMenuState {
                    focused_area: match next_state.focused_area {
                        StartMenuArea::Main => StartMenuArea::Difficulty,
                        StartMenuArea::Difficulty => StartMenuArea::Main,
                    },
                    ..next_state
                }),
                QuitRequested::No,
            ),
            StartMenuItem::Exit => (GameState::StartMenu(next_state), QuitRequested::Yes),
        },
        None => (GameState::StartMenu(next_state), QuitRequested::No),
    }
}

// fn transition

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
        next_round.score += 100;

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

fn draw_start_menu(menu_state: &StartMenuState, window: &pancurses::Window) {
    let (mx, my) = graphics::screen_middle();
    let attributes = {
        let mut attributes = [attributes::A_NORMAL; 3];
        if menu_state.focused_area == StartMenuArea::Main {
            attributes[menu_state.menu_items.current_index()] = attributes::A_REVERSE;
        }
        attributes
    };

    // let game_over = "Rust Snake";
    window.attron(pancurses::COLOR_PAIR(34));
    draw_logo(window, mx - 29 / 2, my - 5);
    window.attroff(pancurses::COLOR_PAIR(34));

    let start_game = "Start";
    window.attron(attributes[StartMenuItem::Start as usize]);
    window.mvprintw(my + 1, mx - start_game.len() as i32 / 2, start_game);
    window.attroff(attributes[StartMenuItem::Start as usize]);

    let difficulty = "Difficulty:";
    window.attron(attributes[StartMenuItem::Difficulty as usize]);
    window.mvprintw(my + 2, mx - 9, difficulty);
    window.attroff(attributes[StartMenuItem::Difficulty as usize]);

    let difficulty_attr = if menu_state.focused_area == StartMenuArea::Difficulty {
        attributes::A_REVERSE
    } else {
        attributes::A_NORMAL
    };
    window.attron(difficulty_attr);
    window.mvprintw(my + 2, mx + 3, format!("{:?}", menu_state.difficulty));
    window.attroff(difficulty_attr);

    let exit = "Exit";
    window.attron(attributes[StartMenuItem::Exit as usize]);
    window.mvprintw(my + 3, mx - exit.len() as i32 / 2, exit);
    window.attroff(attributes[StartMenuItem::Exit as usize]);
}

fn draw_logo(window: &pancurses::Window, x: i32, y: i32) {
    // █████ █   █ █████ █   █ █████
    // █     ██  █ █   █ █  █  █
    // █████ █ █ █ █████ ███   █████
    //     █ █  ██ █   █ █  █  █
    // █████ █   █ █   █ █   █ █████

    // S
    window.draw_horizontal_line(y, x, 5);
    window.draw_horizontal_line(y + 1, x, 1);
    window.draw_horizontal_line(y + 2, x, 5);
    window.draw_horizontal_line(y + 3, x + 4, 1);
    window.draw_horizontal_line(y + 4, x, 5);

    // N
    window.draw_vertical_line(y, x + 6, 5);
    window.draw_vertical_line(y + 1, x + 6 + 1, 1);
    window.draw_vertical_line(y + 2, x + 6 + 2, 1);
    window.draw_vertical_line(y + 3, x + 6 + 3, 1);
    window.draw_vertical_line(y, x + 6 + 4, 5);

    // A
    window.draw_vertical_line(y, x + 12, 5);
    window.draw_horizontal_line(y, x + 13, 3);
    window.draw_horizontal_line(y + 2, x + 13, 3);
    window.draw_vertical_line(y, x + 16, 5);

    // K
    window.draw_vertical_line(y, x + 18, 5);
    window.draw_horizontal_line(y + 2, x + 19, 2);
    window.draw_horizontal_line(y + 1, x + 21, 1);
    window.draw_horizontal_line(y + 3, x + 21, 1);
    window.draw_horizontal_line(y + 0, x + 22, 1);
    window.draw_horizontal_line(y + 4, x + 22, 1);

    // E
    window.draw_vertical_line(y, x + 24, 5);
    window.draw_horizontal_line(y, x + 25, 4);
    window.draw_horizontal_line(y + 2, x + 25, 4);
    window.draw_horizontal_line(y + 4, x + 25, 4);
}

fn draw_round_start(window: &pancurses::Window) {
    let (mx, my) = graphics::screen_middle();
    let get_ready = "Get Ready!";
    window.mvprintw(my, mx - get_ready.len() as i32 / 2, get_ready);
}

fn draw_ongoing_round(state: &RoundState, window: &pancurses::Window) {
    draw_wall(&window, &state.wall);
    draw_snake(&window, &state.snake);
    draw_apple(&window, state.apple);
    draw_score(&window, state.score);
}

fn draw_game_over_screen(state: &GameOverState, window: &pancurses::Window) {
    let (mx, my) = graphics::screen_middle();
    let attrs = match state.selection {
        GameOverSelection::Restart => (attributes::A_REVERSE, attributes::A_NORMAL),
        GameOverSelection::Exit => (attributes::A_NORMAL, attributes::A_REVERSE),
    };

    let game_over = "Game Over";
    window.mvprintw(my - 4, mx - game_over.len() as i32 / 2, game_over);

    let game_over = format!("Final Score: {}", state.final_score);
    window.mvprintw(my - 1, mx - game_over.len() as i32 / 2, game_over);

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

fn draw_score(window: &pancurses::Window, score: usize) {
    let top = graphics::top_screen_margin();
    let left = graphics::left_screen_margin();
    window.mvprintw(top - 2, left, format!("score: {}", score));
}

fn draw_program_exit(window: &pancurses::Window) {
    let (mx, my) = graphics::screen_middle();
    let good_bye = "Good Bye!";
    window.mvprintw(my, mx - good_bye.len() as i32 / 2, good_bye);
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
