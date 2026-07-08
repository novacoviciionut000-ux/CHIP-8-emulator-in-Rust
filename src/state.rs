use std::fs;

pub const MEMORY_SIZE: usize = 4096;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const START_ADDR: u16 = 0x200;

// General hardware sizing
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FLAG_REGISTER: usize = 0xF;

// Font data
const FONTSET_SIZE: usize = 80;
const FONT_SPRITE_HEIGHT: u16 = 5; // bytes per character sprite, used by Fx29
const FONT_START_ADDR: usize = 0x0;

// Opcode decoding masks and shifts
const OPCODE_MASK_TYPE: u16 = 0xF000;
const OPCODE_MASK_X: u16 = 0x0F00;
const OPCODE_MASK_Y: u16 = 0x00F0;
const OPCODE_MASK_N: u16 = 0x000F;
const OPCODE_MASK_NN: u16 = 0x00FF;
const OPCODE_MASK_NNN: u16 = 0x0FFF;
const SHIFT_TYPE: u16 = 12;
const SHIFT_X: u16 = 8;
const SHIFT_Y: u16 = 4;

// Opcode type nibbles (n1)
const OP_TYPE_SYS_OR_MISC: u8 = 0x0;
const OP_TYPE_JP: u8 = 0x1;
const OP_TYPE_CALL: u8 = 0x2;
const OP_TYPE_SE_VX_NN: u8 = 0x3;
const OP_TYPE_SNE_VX_NN: u8 = 0x4;
const OP_TYPE_SE_VX_VY: u8 = 0x5;
const OP_TYPE_LD_VX_NN: u8 = 0x6;
const OP_TYPE_ADD_VX_NN: u8 = 0x7;
const OP_TYPE_ALU: u8 = 0x8;
const OP_TYPE_SNE_VX_VY: u8 = 0x9;
const OP_TYPE_LD_I: u8 = 0xA;
const OP_TYPE_JP_V0: u8 = 0xB;
const OP_TYPE_RND: u8 = 0xC;
const OP_TYPE_DRW: u8 = 0xD;
const OP_TYPE_KEY: u8 = 0xE;
const OP_TYPE_MISC: u8 = 0xF;

// 0x0___ sub-opcodes
const OP_0_CLS_N3: usize = 0xE;
const OP_0_CLS_N4: u8 = 0x0;
const OP_0_RET_N3: usize = 0xE;
const OP_0_RET_N4: u8 = 0xE;

// 0x8XY_ ALU sub-opcodes (n4)
const ALU_LD: u8 = 0x0;
const ALU_OR: u8 = 0x1;
const ALU_AND: u8 = 0x2;
const ALU_XOR: u8 = 0x3;
const ALU_ADD: u8 = 0x4;
const ALU_SUB: u8 = 0x5;
const ALU_SHR: u8 = 0x6;
const ALU_SUBN: u8 = 0x7;
const ALU_SHL: u8 = 0xE;

// 0xEX__ key sub-opcodes
const KEY_SKP_N3: usize = 0x9;
const KEY_SKP_N4: u8 = 0xE;
const KEY_SKNP_N3: usize = 0xA;
const KEY_SKNP_N4: u8 = 0x1;

// 0xFX__ misc sub-opcodes (n3, n4)
const MISC_LD_VX_DT_N3: usize = 0x0;
const MISC_LD_VX_DT_N4: u8 = 0x7;
const MISC_LD_VX_K_N3: usize = 0x0;
const MISC_LD_VX_K_N4: u8 = 0xA;
const MISC_LD_DT_VX_N3: usize = 0x1;
const MISC_LD_DT_VX_N4: u8 = 0x5;
const MISC_LD_ST_VX_N3: usize = 0x1;
const MISC_LD_ST_VX_N4: u8 = 0x8;
const MISC_ADD_I_VX_N3: usize = 0x1;
const MISC_ADD_I_VX_N4: u8 = 0xE;
const MISC_LD_F_VX_N3: usize = 0x2;
const MISC_LD_F_VX_N4: u8 = 0x9;
const MISC_LD_B_VX_N3: usize = 0x3;
const MISC_LD_B_VX_N4: u8 = 0x3;
const MISC_LD_I_VX_N3: usize = 0x5;
const MISC_LD_I_VX_N4: u8 = 0x5;
const MISC_LD_VX_I_N3: usize = 0x6;
const MISC_LD_VX_I_N4: u8 = 0x5;

// BCD conversion (Fx33)
const BCD_HUNDREDS_DIVISOR: u8 = 100;
const BCD_TENS_DIVISOR: u8 = 10;
const BCD_ONES_MODULUS: u8 = 10;

// Sprite drawing (Dxyn)
const SPRITE_WIDTH_BITS: usize = 8;
const SPRITE_PIXEL_MSB_MASK: u8 = 0x80;

// Bit manipulation for shift opcodes (8XY6 / 8XYE)
const SINGLE_BIT_MASK: u8 = 0x1;
const BYTE_MSB_SHIFT: u8 = 7;

pub struct Cpu {
    pub memory: [u8; MEMORY_SIZE],
    pub v: [u8; NUM_REGISTERS],
    pub i: u16,
    pub pc: u16,
    pub stack: [u16; STACK_SIZE],
    pub sp: usize,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pub keypad: [bool; NUM_KEYS],
}

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80
];

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Self {
            memory: [0; MEMORY_SIZE],
            v: [0; NUM_REGISTERS],
            i: 0,
            pc: START_ADDR,
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; DISPLAY_HEIGHT * DISPLAY_WIDTH],
            keypad: [false; NUM_KEYS],
        };
        for i in 0..FONTSET.len() {
            cpu.memory[FONT_START_ADDR + i] = FONTSET[i];
        }
        cpu
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + rom.len();

        if end > MEMORY_SIZE {
            panic!("ROM is too large to fit in memory");
        }
        self.memory[start..end].copy_from_slice(rom);
    }

    pub fn load_rom_file(&mut self, rom_path: &str) -> Option<bool> {
        let rom_bytes = match fs::read(rom_path) {
            Ok(bytes) => bytes,
            Err(_) => return None,
        };

        self.load_rom(&rom_bytes);
        Some(true)
    }

    pub fn fetch_instruction_and_increment_pc(&mut self) -> u16 {
        let byte_1: u8 = self.memory[self.pc as usize];
        let byte_2: u8 = self.memory[self.pc as usize + 1];
        self.pc += 2;
        (byte_1 as u16) << 8 | (byte_2 as u16)
    }

    pub fn execute(&mut self, opcode: u16) {
        let n1 = ((opcode & OPCODE_MASK_TYPE) >> SHIFT_TYPE) as u8;
        let n2 = ((opcode & OPCODE_MASK_X) >> SHIFT_X) as usize;
        let n3 = ((opcode & OPCODE_MASK_Y) >> SHIFT_Y) as usize;
        let n4 = (opcode & OPCODE_MASK_N) as u8;

        let nn = (opcode & OPCODE_MASK_NN) as u8;
        let nnn = opcode & OPCODE_MASK_NNN;

        match (n1, n2, n3, n4) {
            (OP_TYPE_SYS_OR_MISC, 0, OP_0_CLS_N3, OP_0_CLS_N4) => {
                self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT]
            }
            (OP_TYPE_SYS_OR_MISC, 0, OP_0_RET_N3, OP_0_RET_N4) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            }
            (OP_TYPE_JP, _, _, _) => self.pc = nnn,
            (OP_TYPE_CALL, _, _, _) => {
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            (OP_TYPE_SE_VX_NN, x, _, _) => {
                if self.v[x] == nn { self.pc += 2; }
            }
            (OP_TYPE_SNE_VX_NN, x, _, _) => {
                if self.v[x] != nn { self.pc += 2; }
            }
            (OP_TYPE_SE_VX_VY, x, y, 0) => {
                if self.v[x] == self.v[y] { self.pc += 2; }
            }
            (OP_TYPE_LD_VX_NN, x, _, _) => self.v[x] = nn,
            (OP_TYPE_ADD_VX_NN, x, _, _) => self.v[x] = self.v[x].wrapping_add(nn),
            (OP_TYPE_ALU, x, y, ALU_LD) => self.v[x] = self.v[y],
            (OP_TYPE_ALU, x, y, ALU_OR) => self.v[x] |= self.v[y],
            (OP_TYPE_ALU, x, y, ALU_AND) => self.v[x] &= self.v[y],
            (OP_TYPE_ALU, x, y, ALU_XOR) => self.v[x] ^= self.v[y],
            (OP_TYPE_ALU, x, y, ALU_ADD) => {
                let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = res;
                self.v[FLAG_REGISTER] = if overflow { 1 } else { 0 };
            }
            (OP_TYPE_ALU, x, y, ALU_SUB) => {
                let (res, borrow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = res;
                self.v[FLAG_REGISTER] = if borrow { 0 } else { 1 };
            }
            (OP_TYPE_ALU, x, _, ALU_SHR) => {
                let lsb = self.v[x] & SINGLE_BIT_MASK;
                self.v[x] >>= 1;
                self.v[FLAG_REGISTER] = lsb;
            }
            (OP_TYPE_ALU, x, y, ALU_SUBN) => {
                let (res, borrow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = res;
                self.v[FLAG_REGISTER] = if borrow { 0 } else { 1 };
            }
            (OP_TYPE_ALU, x, _, ALU_SHL) => {
                let msb = (self.v[x] >> BYTE_MSB_SHIFT) & SINGLE_BIT_MASK;
                self.v[x] <<= 1;
                self.v[FLAG_REGISTER] = msb;
            }
            (OP_TYPE_SNE_VX_VY, x, y, 0) => {
                if self.v[x] != self.v[y] { self.pc += 2; }
            }
            (OP_TYPE_LD_I, _, _, _) => self.i = nnn,
            (OP_TYPE_JP_V0, _, _, _) => self.pc = nnn + (self.v[0] as u16),
            (OP_TYPE_RND, x, _, _) => {
                let random_byte: u8 = rand::random();
                self.v[x] = random_byte & nn;
            }
            (OP_TYPE_DRW, x, y, n) => {
                self.draw_sprite(x, y, n);
            }
            (OP_TYPE_KEY, x, KEY_SKP_N3, KEY_SKP_N4) => {
                let key = self.v[x] as usize;
                if self.keypad[key] { self.pc += 2; }
            }
            (OP_TYPE_KEY, x, KEY_SKNP_N3, KEY_SKNP_N4) => {
                let key = self.v[x] as usize;
                if !self.keypad[key] { self.pc += 2; }
            }
            (OP_TYPE_MISC, x, MISC_LD_VX_DT_N3, MISC_LD_VX_DT_N4) => self.v[x] = self.delay_timer,
            (OP_TYPE_MISC, x, MISC_LD_VX_K_N3, MISC_LD_VX_K_N4) => {
                let mut pressed = false;
                for i in 0..NUM_KEYS {
                    if self.keypad[i] {
                        self.v[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            }
            (OP_TYPE_MISC, x, MISC_LD_DT_VX_N3, MISC_LD_DT_VX_N4) => self.delay_timer = self.v[x],
            (OP_TYPE_MISC, x, MISC_LD_ST_VX_N3, MISC_LD_ST_VX_N4) => self.sound_timer = self.v[x],
            (OP_TYPE_MISC, x, MISC_ADD_I_VX_N3, MISC_ADD_I_VX_N4) => {
                self.i = self.i.wrapping_add(self.v[x] as u16);
            }
            (OP_TYPE_MISC, x, MISC_LD_F_VX_N3, MISC_LD_F_VX_N4) => {
                self.i = (self.v[x] as u16) * FONT_SPRITE_HEIGHT;
            }
            (OP_TYPE_MISC, x, MISC_LD_B_VX_N3, MISC_LD_B_VX_N4) => {
                let value = self.v[x];
                self.memory[self.i as usize] = value / BCD_HUNDREDS_DIVISOR;
                self.memory[(self.i + 1) as usize] = (value / BCD_TENS_DIVISOR) % BCD_ONES_MODULUS;
                self.memory[(self.i + 2) as usize] = value % BCD_ONES_MODULUS;
            }
            (OP_TYPE_MISC, x, MISC_LD_I_VX_N3, MISC_LD_I_VX_N4) => {
                for idx in 0..=x {
                    self.memory[(self.i as usize) + idx] = self.v[idx];
                }
            }
            (OP_TYPE_MISC, x, MISC_LD_VX_I_N3, MISC_LD_VX_I_N4) => {
                for idx in 0..=x {
                    self.v[idx] = self.memory[(self.i as usize) + idx];
                }
            }
            _ => panic!("Unknown opcode: {:#06X}", opcode),
        }
    }

    fn draw_sprite(&mut self, x: usize, y: usize, n: u8) {
        let start_x: usize = self.v[x] as usize;
        let start_y: usize = self.v[y] as usize;
        self.v[FLAG_REGISTER] = 0;
        for row in 0..n {
            let sprite_address = self.i + (row as u16);
            let sprite_byte = self.memory[sprite_address as usize];
            for col in 0..SPRITE_WIDTH_BITS {
                let sprite_pixel = sprite_byte & (SPRITE_PIXEL_MSB_MASK >> col);

                if sprite_pixel != 0 {
                    let target_x = (start_x + col) % DISPLAY_WIDTH;
                    let target_y = (start_y + (row as usize)) % DISPLAY_HEIGHT;

                    let index = (target_y * DISPLAY_WIDTH) + target_x;

                    if self.display[index] {
                        self.display[index] = false;
                        self.v[FLAG_REGISTER] = 1;
                    } else {
                        self.display[index] = true;
                    }
                }
            }
        }
    }

    pub fn fetch_instruction_increment_execute(&mut self) {
        let curr_instruct: u16 = self.fetch_instruction_and_increment_pc();
        self.execute(curr_instruct);
    }
}