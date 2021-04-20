#include "keyboard.h"
#include <windows.h>
#include <algorithm>
#include <vector>

namespace {

using RawKeyState = short;
using KeyState = input::keyboard::KeyState;

bool _raw_key_is_pressed(RawKeyState raw_key_state) {
    return raw_key_state & 0x8000;
}

bool _key_is_up(KeyState key_state) {
    return key_state == KeyState::JustReleased || key_state == KeyState::Released;
}

bool _key_is_down(KeyState key_state) {
    return key_state == KeyState::JustPressed || key_state == KeyState::Pressed;
}

KeyState get_key_state(KeyState prev_state, short raw_state) {
    if (_raw_key_is_pressed(raw_state)) {
        return _key_is_up(prev_state) ? KeyState::JustPressed : KeyState::Pressed;
    } else {
        return _key_is_down(prev_state) ? KeyState::JustReleased : KeyState::Released;
    }
}

} // namespace

namespace input::keyboard {

KeyboardHandler::KeyboardHandler() : key_states(NUM_KEY_STATES) {
}

void KeyboardHandler::update() {
    for (int i = 0; i < key_states.size(); i++) {
        KeyState prev_state = key_states[i];
        short raw_state = GetAsyncKeyState(i);
        key_states[i] = get_key_state(prev_state, raw_state);
    }
}

bool KeyboardHandler::key_is_up(KeyCode key) {
    return _key_is_up(key_states[key]);
}

bool KeyboardHandler::key_is_down(KeyCode key) {
    return _key_is_down(key_states[key]);
}

bool KeyboardHandler::key_pressed_now(KeyCode key) {
    return key_states[key] == KeyState::JustPressed;
}

bool KeyboardHandler::key_released_now(KeyCode key) {
    return key_states[key] == KeyState::JustReleased;
}

bool any_key_pressed(input::keyboard::KeyboardHandler& handler) {
    std::vector<SHORT> non_keyboard_codes = {
        VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2
    };
    for (int i = 0; i < input::keyboard::KeyboardHandler::NUM_KEY_STATES; i++) {
        auto pos = std::find(non_keyboard_codes.begin(), non_keyboard_codes.end(), i);
        if (pos != non_keyboard_codes.end()) {
            continue;
        }

        if (handler.key_pressed_now(i)) {
            return true;
        }
    }
    return false;
}

} // namespace input::keyboard
