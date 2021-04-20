#include "curses.h"
#include <vector>
#include <optional>
#include <windows.h>
#include <stdio.h>
#include <math.h>
#include "keyboard.h"

namespace keyboard = input::keyboard;
struct Vec2 {
    float x;
    float y;

    Vec2 operator-(const Vec2 &other) const {
        return {x - other.x, y - other.y};
    }
};
enum class Direction {
    Right, Up, Left, Down
};
using SnakeBody = std::vector<Vec2>;
constexpr size_t SCREEN_WIDTH = 84;
constexpr size_t SCREEN_HEIGHT = 20;

long get_perf_counter_freq() {
    LARGE_INTEGER perf_counter_freq;
    QueryPerformanceFrequency(&perf_counter_freq);
    return perf_counter_freq.QuadPart;
}

long get_perf_counter_ticks() {
    LARGE_INTEGER current_ticks;
    QueryPerformanceCounter(&current_ticks);
    return current_ticks.QuadPart;
}

long get_microsec_timestamp() {
    long perf_counter_freq = get_perf_counter_freq();
    long current_ticks = get_perf_counter_ticks();
    long ticks_scaled_by_megahz = current_ticks * 1e6;
    long microsec_ticks = ticks_scaled_by_megahz / perf_counter_freq;
    return microsec_ticks;
}

std::optional<Direction> get_direction(keyboard::KeyboardHandler& keyboard_handler) {
        if (keyboard_handler.key_pressed_now(VK_LEFT)) {
            return Direction::Left;
        }
        else if (keyboard_handler.key_pressed_now(VK_RIGHT)) {
            return Direction::Right;
        }
        else if (keyboard_handler.key_pressed_now(VK_DOWN)) {
            return Direction::Down;
        }
        else if (keyboard_handler.key_pressed_now(VK_UP)) {
            return Direction::Up;
        }
        return {};
}

Vec2 get_new_segment(Direction dir, Vec2 last_segment) {
    switch (dir) {
        case Direction::Right:
            return {last_segment.x + 1, last_segment.y};
            break;
        case Direction::Up:
            return {last_segment.x, last_segment.y - 1};
            break;
        case Direction::Left:
            return {last_segment.x - 1, last_segment.y};
            break;
        case Direction::Down:
            return {last_segment.x, last_segment.y + 1};
            break;
    }
    return {0, 0}; // unreachable
}

// shortens the tail of the snake by moving the last segment one step towards
// the second to last segment.
void shorten_tail(SnakeBody& body) {
    if (body.size() < 2) {
        return;
    }

    Vec2 delta = body[1] - body[0];
    // horizontal
    if (delta.y == 0) {
        int sign_x = delta.x < 0 ? -1 : 1;
        body[0].x += sign_x;
        if (body[0].x == body[1].x) {
            body.erase(body.begin());
        }
    }
    // vertical
    else {
        int sign_y = delta.y < 0 ? -1 : 1;
        body[0].y += sign_y;
        if (body[0].y == body[1].y) {
            body.erase(body.begin());
        }
    }
}

void move_snake_body(SnakeBody& snake_body, Direction dir) {
    Vec2 last_segment = snake_body[snake_body.size() - 1];
    Vec2 new_segment = get_new_segment(dir, last_segment);
    shorten_tail(snake_body);
    snake_body.push_back(new_segment);
}

void draw_horizontal_line(int x, int y, int width) {
    int sign = width < 0 ? -1 : 1;
    for (int i = 0; i < abs(width); i++) {
        mvprintw(y, x + i * sign, "%c", 219);
    }
}

void draw_vertical_line(int x, int y, int height) {
    int sign = height < 0 ? -1 : 1;
    for (int i = 0; i < abs(height); i++) {
        mvprintw(y + i * sign, x, "%c", 219);
    }
}

// void draw_snake_head()

void draw_snake_body(SnakeBody& snake_body) {
    if (snake_body.empty()) {
        return;
    }

    int top_margin = (LINES - SCREEN_HEIGHT)/2;
    int left_margin = (COLS - SCREEN_WIDTH)/2;

    attron(COLOR_PAIR(34));
    if (snake_body.size() == 1) {
        int x = round(left_margin + 2 + snake_body[0].x);
        int y = round(top_margin + 1 + snake_body[0].y);
        draw_horizontal_line(x, y, 1);
        attroff(COLOR_PAIR(34));
        return;
    }

    for (int i = 0; i < snake_body.size() - 1; i++) {
        int x = round(left_margin + 2 + snake_body[i].x);
        int y = round(top_margin + 1 + snake_body[i].y);
        Vec2 delta = snake_body[i+1] - snake_body[i];
        if (delta.y == 0) {
            int sign_x = delta.x < 0 ? -1 : 1;
            draw_horizontal_line(x, y, delta.x + sign_x);
        } else {
            int sign_y = delta.y < 0 ? -1 : 1;
            draw_vertical_line(x, y, delta.y + sign_y);
        }
    }
    attroff(COLOR_PAIR(34));
}

int main() {
    initscr();
    keyboard::KeyboardHandler keyboard_handler;
    int i = 0;

    while (1) {
        keyboard_handler.update();
        if (keyboard::any_key_pressed(keyboard_handler)) {
            break;
        }
        i += 1;

        erase();
        resize_term(0, 0);
        curs_set(0); // hide cursor
        printw("i = %d\n", i);
        printw("COLS = %d, LINES = %d!\n", COLS, LINES);
        refresh();
    }

    endwin();
}


int _main() {
    /* Initialize */
    initscr();
    curs_set(0); // hide cursor
    noecho();
    timeout(0); // non blocking getch
    // initialize colors
    start_color();
    for (int color = 16; color < 256; color++) {
        init_pair(color, color, COLOR_BLACK);
    }

    /* Run program */
    keyboard::KeyboardHandler keyboard_handler;
    // timing
    long prev_time = get_microsec_timestamp();
    int elapsed_frames = 0;
    // snake
    SnakeBody snake_body = {{0,0}, {2,0}, {2,2}, {4,2}, {4,4}, {6,4}};
    float pos_dx = 0;
    float pos_dy = 0;
    float pos_x = 0;
    float pos_y = 0;

    while (1) {
        long time_now = get_microsec_timestamp();
        long elapsed_time = time_now - prev_time;
        if (elapsed_time <= 1e6/60.0) {
            continue;
        }
        prev_time = time_now;
        elapsed_frames++;

        curs_set(0); // hide cursor

        keyboard_handler.update();
        if (keyboard_handler.key_pressed_now(VK_ESCAPE)) {
            break;
        }


        /* Update */
        // move snake body
        std::optional<Direction> dir = get_direction(keyboard_handler);
        if (dir.has_value()) {
            move_snake_body(snake_body, dir.value());
        }

        /* Draw */
        erase();
        resize_term(0, 0); // without this window resizes messes up printing
        int top_margin = (LINES - SCREEN_HEIGHT)/2;
        int left_margin = (COLS - SCREEN_WIDTH)/2;

        // draw surrounding box
        draw_horizontal_line(left_margin, top_margin, SCREEN_WIDTH);
        draw_horizontal_line(left_margin, top_margin + SCREEN_HEIGHT - 1, SCREEN_WIDTH);
        draw_vertical_line(left_margin, top_margin + 1, SCREEN_HEIGHT - 1);
        draw_vertical_line(left_margin + SCREEN_WIDTH - 1, top_margin + 1, SCREEN_HEIGHT - 1);
        // draw messages
        mvprintw(top_margin + 5, left_margin + 2, "elapsed time = %zu\n", elapsed_time);
        mvprintw(top_margin + 6, left_margin + 2, "elapsed frames = %d\n", elapsed_frames);
        mvprintw(top_margin + 7, left_margin + 2, "elapsed seconds = %f\n", elapsed_frames / 60.0);
        // draw snake body
        draw_snake_body(snake_body);

        refresh();
    }

    /* Shut down */
	endwin();
    return 0;
}
