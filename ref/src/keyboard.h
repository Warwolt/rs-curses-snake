#include <vector>
#pragma once

namespace input::keyboard {

enum class KeyState {
    Released,
    JustReleased,
    Pressed,
    JustPressed
};

using KeyCode = int;

class KeyboardHandler {
public:
    static constexpr int NUM_KEY_STATES = 256;

    KeyboardHandler();
    void update();
    bool key_is_up(KeyCode key);
    bool key_is_down(KeyCode key);
    bool key_pressed_now(KeyCode key);
    bool key_released_now(KeyCode key);

private:
    using RawKeyState = short;
    std::vector<KeyState> key_states;
};

bool any_key_pressed(input::keyboard::KeyboardHandler& handler);

} // namespace input::keyboard
