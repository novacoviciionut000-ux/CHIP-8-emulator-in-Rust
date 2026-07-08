const WINDOW_WIDTH: usize = 640;
const WINDOW_HEIGHT: usize = 320;
const SCALE: usize = 10;

const FONT_GLYPH_WIDTH: usize = 8;

const MENU_LABEL_COLOR: u32 = 0xAAAAAA;
const MENU_LABEL_X: usize = 10;
const MENU_LABEL_Y: usize = 10;

const ROM_LIST_VISIBLE_ITEMS: usize = 22;
const ROM_LIST_START_X: usize = 20;
const ROM_LIST_START_Y: usize = 30;
const ROM_LIST_ROW_HEIGHT: usize = 12;

const COLOR_SELECTED: u32 = 0x00FF00;
const COLOR_UNSELECTED: u32 = 0xFFFFFF;
const COLOR_PIXEL_ON: u32 = 0xFFFFFF;

const ROM_FILE_EXTENSIONS: [&str; 2] = ["rom", "ch8"];

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;
use std::thread;
use font8x8::BASIC_FONTS;
use font8x8::UnicodeFonts;

use crate::state::{Cpu, DISPLAY_WIDTH, DISPLAY_HEIGHT};
use crate::audio::AudioHandler;
use crate::windowing;
use crate::emulator_loop::emulate_loop;

pub enum AppState {
    Menu,
    Playing,
}

pub fn draw_text(buffer: &mut [u32], text: &str, start_x: usize, start_y: usize, color: u32, screen_width: usize) {
    let screen_height = buffer.len() / screen_width;

    for (char_idx, c) in text.chars().enumerate() {
        if let Some(glyph) = BASIC_FONTS.get(c) {
            for (y, row) in glyph.iter().enumerate() {
                for x in 0..FONT_GLYPH_WIDTH {
                    if (row & (1 << x)) != 0 {
                        let px = start_x + (char_idx * FONT_GLYPH_WIDTH) + x;
                        let py = start_y + y;

                        if px < screen_width && py < screen_height {
                            buffer[py * screen_width + px] = color;
                        }
                    }
                }
            }
        }
    }
}

pub struct Application {
    window: minifb::Window,
    buffer: Vec<u32>,
    audio: AudioHandler,
    shared_cpu: Arc<Mutex<Cpu>>,
    is_running: Arc<AtomicBool>,
    should_beep: Arc<AtomicBool>,
    state: AppState,
    roms: Vec<PathBuf>,
    selected_rom: usize,
}

impl Application {
    pub fn new() -> Self {
        let window = windowing::init_window("CHIP-8 Emulator");
        let buffer = vec![0u32; WINDOW_WIDTH * WINDOW_HEIGHT];
        let should_beep = Arc::new(AtomicBool::new(false));

        Self {
            window,
            buffer,
            audio: AudioHandler::new(Arc::clone(&should_beep)),
            shared_cpu: Arc::new(Mutex::new(Cpu::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            should_beep,
            state: AppState::Menu,
            roms: Self::get_rom_list(),
            selected_rom: 0,
        }
    }

    pub fn run(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape) {
            match self.state {
                AppState::Menu => self.update_menu(),
                AppState::Playing => self.update_playing(),
            }
        }
    }

    fn update_menu(&mut self) {
        self.buffer.fill(0);

        if !self.roms.is_empty() {
            if self.window.is_key_pressed(minifb::Key::Down, minifb::KeyRepeat::No) {
                self.selected_rom = (self.selected_rom + 1) % self.roms.len();
            }
            if self.window.is_key_pressed(minifb::Key::Up, minifb::KeyRepeat::No) {
                if self.selected_rom == 0 {
                    self.selected_rom = self.roms.len() - 1;
                } else {
                    self.selected_rom -= 1;
                }
            }
            if self.window.is_key_pressed(minifb::Key::Enter, minifb::KeyRepeat::No) {
                self.start_game();
                return;
            }
        }

        draw_text(&mut self.buffer, "SELECT ROM:", MENU_LABEL_X, MENU_LABEL_Y, MENU_LABEL_COLOR, WINDOW_WIDTH);

        let start_idx = if self.selected_rom >= ROM_LIST_VISIBLE_ITEMS {
            self.selected_rom - ROM_LIST_VISIBLE_ITEMS + 1
        } else {
            0
        };

        for i in 0..ROM_LIST_VISIBLE_ITEMS {
            let rom_idx = start_idx + i;

            if rom_idx >= self.roms.len() { break; }

            let rom = &self.roms[rom_idx];
            let color = if rom_idx == self.selected_rom { COLOR_SELECTED } else { COLOR_UNSELECTED };

            if let Some(name) = rom.file_name().and_then(|n| n.to_str()) {
                draw_text(
                    &mut self.buffer,
                    name,
                    ROM_LIST_START_X,
                    ROM_LIST_START_Y + (i * ROM_LIST_ROW_HEIGHT),
                    color,
                    WINDOW_WIDTH,
                );
            }
        }

        self.window.update_with_buffer(&self.buffer, WINDOW_WIDTH, WINDOW_HEIGHT).unwrap();
    }

    fn update_playing(&mut self) {
        let display_copy = {
            let mut cpu = self.shared_cpu.lock().unwrap();
            windowing::update_keypad(&self.window, &mut cpu.keypad);
            cpu.display
        };

        self.audio.update();

        if self.window.is_key_pressed(minifb::Key::Backspace, minifb::KeyRepeat::No) {
            self.stop_game();
            return;
        }

        self.buffer.fill(0);

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let pixel = display_copy[y * DISPLAY_WIDTH + x];

                if pixel {
                    for dy in 0..SCALE {
                        for dx in 0..SCALE {
                            let px = (x * SCALE) + dx;
                            let py = (y * SCALE) + dy;
                            self.buffer[py * WINDOW_WIDTH + px] = COLOR_PIXEL_ON;
                        }
                    }
                }
            }
        }

        self.window.update_with_buffer(&self.buffer, WINDOW_WIDTH, WINDOW_HEIGHT).unwrap();
    }

    fn start_game(&mut self) {
        if self.roms.is_empty() { return; }

        let mut cpu = self.shared_cpu.lock().unwrap();
        *cpu = Cpu::new();

        let rom_path = self.roms[self.selected_rom].to_str().expect("Invalid path characters");
        cpu.load_rom_file(rom_path).expect("Failed to load ROM");

        self.is_running.store(true, Ordering::Relaxed);
        let cpu_for_thread = Arc::clone(&self.shared_cpu);
        let running_for_thread = Arc::clone(&self.is_running);
        let should_beep_for_thread = Arc::clone(&self.should_beep);

        thread::spawn(move || {
            emulate_loop(cpu_for_thread, running_for_thread, should_beep_for_thread);
        });

        self.state = AppState::Playing;
    }

    fn stop_game(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        self.should_beep.store(false, Ordering::Relaxed);
        self.state = AppState::Menu;
    }

    fn get_rom_list() -> Vec<PathBuf> {
        let mut roms = Vec::new();
        if let Ok(entries) = std::fs::read_dir("./games") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if let Some(ext_str) = ext.to_str() {
                            if ROM_FILE_EXTENSIONS.contains(&ext_str) {
                                roms.push(path);
                            }
                        }
                    }
                }
            }
        }
        roms
    }
}