use minifb::{Window, WindowOptions, Scale, Key};
use crate::state::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

const WINDOW_SCALE: Scale = Scale::X16;
const TARGET_FPS: usize = 60;

const COLOR_PIXEL_ON: u32 = 0xFFFFFF;
const COLOR_PIXEL_OFF: u32 = 0x000000;

const NUM_KEYS: usize = 16;

pub fn init_window(title: &str) -> Window {
    let mut window = Window::new(
        title,
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        WindowOptions {
            scale: WINDOW_SCALE,
            ..WindowOptions::default()
        },
    ).expect("Failed to create window");

    window.set_target_fps(TARGET_FPS);
    window
}

pub fn display_to_window(
    window: &mut Window,
    cpu_display: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    buffer: &mut [u32; DISPLAY_WIDTH * DISPLAY_HEIGHT]
) {
    for (i, &pixel) in cpu_display.iter().enumerate() {
        buffer[i] = if pixel { COLOR_PIXEL_ON } else { COLOR_PIXEL_OFF };
    }

    window
        .update_with_buffer(buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .expect("Failed to update window buffer");
}

// CHIP-8 keypad -> QWERTY keyboard, mapped by physical position:
//
//   CHIP-8          Keyboard
//   1 2 3 C         1 2 3 4
//   4 5 6 D   <->   Q W E R
//   7 8 9 E         A S D F
//   A 0 B F         Z X C V
pub fn update_keypad(window: &Window, keypad: &mut [bool; NUM_KEYS]) {
    keypad.fill(false);

    for key in window.get_keys() {
        match key {
            Key::Key1 => keypad[0x1] = true,
            Key::Key2 => keypad[0x2] = true,
            Key::Key3 => keypad[0x3] = true,
            Key::Key4 => keypad[0xC] = true,
            Key::Q => keypad[0x4] = true,
            Key::W => keypad[0x5] = true,
            Key::E => keypad[0x6] = true,
            Key::R => keypad[0xD] = true,
            Key::A => keypad[0x7] = true,
            Key::S => keypad[0x8] = true,
            Key::D => keypad[0x9] = true,
            Key::F => keypad[0xE] = true,
            Key::Z => keypad[0xA] = true,
            Key::X => keypad[0x0] = true,
            Key::C => keypad[0xB] = true,
            Key::V => keypad[0xF] = true,
            _ => (),
        }
    }
}