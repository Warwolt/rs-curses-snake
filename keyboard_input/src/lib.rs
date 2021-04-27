use winapi::um::winnt::SHORT;
use winapi::um::winuser::GetAsyncKeyState;
use winapi::um::winuser;

#[derive(Clone, Copy, PartialEq)]
pub enum KeyState {
    Released, JustReleased, Pressed, JustPressed
}

type RawKeyState = SHORT;
pub type KeyCode = i32;

pub struct KeyboardHandler {
    key_states : Vec<KeyState>,
}

fn raw_key_pressed(state: RawKeyState) -> bool {
    (state as u16 & 0x8000) > 0
}

fn key_is_up(state: KeyState) -> bool {
    state == KeyState::JustReleased || state == KeyState::Released
}

fn key_is_down(state: KeyState) -> bool {
    state == KeyState::JustPressed || state == KeyState::Pressed
}

fn get_key_state(prev_state: KeyState, raw_state: RawKeyState) -> KeyState {
    if raw_key_pressed(raw_state) {
        if key_is_up(prev_state) {KeyState::JustPressed} else {KeyState::Pressed}
    } else {
        if key_is_down(prev_state) {KeyState::JustReleased} else {KeyState::Released}
    }
}

fn get_raw_key_state(key: KeyCode) -> RawKeyState {
    unsafe {
        GetAsyncKeyState(key as i32)
    }
}

pub fn any_key_pressed(handler: &KeyboardHandler) -> bool {
    let non_keyboard_codes: Vec<KeyCode> = vec![winuser::VK_LBUTTON, winuser::VK_RBUTTON,
        winuser::VK_MBUTTON, winuser::VK_XBUTTON1, winuser::VK_XBUTTON2];

    for code in 0..KeyboardHandler::NUM_KEY_STATES as i32 {
        let not_keyboard_code = non_keyboard_codes.iter().any(|&other_code| other_code==code);
        if not_keyboard_code {
            continue;
        }

        if handler.key_is_down(code) {
            return true;
        }
    }

    false
}

impl KeyboardHandler {
    const NUM_KEY_STATES: usize = 256;

    pub fn new() -> Self {
        Self {key_states: vec![KeyState::Released; Self::NUM_KEY_STATES]}
    }

    pub fn update(&mut self) {
        for i in 0..Self::NUM_KEY_STATES as KeyCode {
            let prev_state = self.key_states[i as usize];
            let raw_state = get_raw_key_state(i);
            self.key_states[i as usize] = get_key_state(prev_state, raw_state);
        }
    }

    pub fn key_is_up(&self, key: KeyCode) -> bool {
        key_is_up(self.key_states[key as usize])
    }

    pub fn key_is_down(&self, key: KeyCode) -> bool {
        key_is_down(self.key_states[key as usize])
    }

    pub fn key_pressed_now(&self, key: KeyCode) -> bool {
        self.key_states[key as usize] == KeyState::JustPressed
    }

    pub fn key_released_now(&self, key: KeyCode) -> bool {
        self.key_states[key as usize] == KeyState::JustReleased
    }
}
