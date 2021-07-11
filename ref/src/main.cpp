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

int main() {
    /* Initialize */
    initscr();
    curs_set(0); // hide cursor
    noecho();
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

        clear();

        // test out attributes
        // https://unix.stackexchange.com/a/151717/354394
        attron(A_BOLD);
        addstr("Twinkle, twinkle little star\n");
        attron(A_BLINK);
        addstr("How I wonder what you are.\n");
        attroff(A_BOLD);
        addstr("Up above the world so high,\n");
        attrset(A_NORMAL);
        addstr("Like a diamond in the sky.\n");
        attron(A_REVERSE);
        addstr("Twinkle, twinkle little star\n");
        attrset(A_NORMAL);
        addstr("How I wonder what you are.\n");

        refresh();
    }

    /* Shut down */
	endwin();
    return 0;
}
