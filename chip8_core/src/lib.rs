use std::{panic, u8};

use crossterm::event::{Event, KeyCode, KeyEventKind, read};
use rand::{Rng, rng};

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONT_BASE_ADDR: usize = 0xD00;
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,

    stack_pointer: u16,
    stack: [u16; STACK_SIZE],
    pub keys: [bool; NUM_KEYS],

    delay_timer: u8,
    sound_timer: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut ram = [0; RAM_SIZE];
        for i in 0..FONTSET_SIZE {
            ram[FONT_BASE_ADDR + i] = FONTSET[i];
        }

        Self {
            pc: START_ADDR,
            ram,
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn fetch(&mut self) -> Result<(), ()> {
        let opcode = {
            let a = self.ram[self.pc as usize] as u16;
            let b = self.ram[self.pc as usize + 1] as u16;
            (a << 8) | b
        };

        //println!("PC = {:X}", self.pc);
        //println!("opcode = {:X}", opcode);
        self.pc += 2;

        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x0000 => { /* NOP */ }
                    0x00E0 => {
                        self.screen_clear();
                    }
                    0x00EE => {
                        self.pc = self.pop();
                    }
                    _ => {
                        panic!("opcode {:X} is not exists", opcode)
                    }
                }
            }
            0x1000 => {
                let addr = opcode & 0x0FFF;
                self.pc = addr;
            }
            0x2000 => {
                let addr = opcode & 0x0FFF;
                self.push(self.pc);
                self.pc = addr;
            }
            0x3000 => {
                let x = (opcode & 0x0F00) >> 8;
                let target = opcode & 0x00FF;
                if target as u8 == self.v_reg[x as usize] {
                    self.pc += 2;
                }
            }
            0x4000 => {
                let x = (opcode & 0x0F00) >> 8;
                let target = opcode & 0x00FF;
                if target as u8 != self.v_reg[x as usize] {
                    self.pc += 2;
                }
            }
            0x5000 => {
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                if self.v_reg[x as usize] == self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                let x = (opcode & 0x0F00) >> 8;
                let val = opcode & 0x00FF;
                self.v_reg[x as usize] = val as u8;
            }
            0x7000 => {
                let reg = (opcode & 0x0F00) >> 8;
                let val = opcode & 0x00FF;
                self.v_reg[reg as usize] = self.v_reg[reg as usize].wrapping_add(val as u8);
            }
            0x8000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                match opcode & 0x000F {
                    0x0 => {
                        self.v_reg[x] = self.v_reg[y];
                    }
                    0x1 => {
                        self.v_reg[x] |= self.v_reg[y];
                    }
                    0x2 => {
                        self.v_reg[x] &= self.v_reg[y];
                    }
                    0x3 => {
                        self.v_reg[x] ^= self.v_reg[y];
                    }
                    0x4 => {
                        let a = self.v_reg[x];
                        let b = self.v_reg[y];
                        let (val, of) = a.overflowing_add(b);
                        self.v_reg[x] = val;
                        self.v_reg[0xf] = if of { 1 } else { 0 };
                    }
                    0x5 => {
                        if self.v_reg[x] >= self.v_reg[y] {
                            self.v_reg[0xf] = 1;
                        } else {
                            self.v_reg[0xf] = 0;
                        }

                        self.v_reg[x] = self.v_reg[x].overflowing_sub(self.v_reg[y]).0;
                    }
                    0x6 => {
                        self.v_reg[0xf] = self.v_reg[x] & 1;
                        self.v_reg[x] >>= 1;
                    }
                    0x7 => {
                        if self.v_reg[y] >= self.v_reg[x] {
                            self.v_reg[0xf] = 1;
                        } else {
                            self.v_reg[0xf] = 0;
                        }
                        self.v_reg[x] = self.v_reg[y].overflowing_sub(self.v_reg[x]).0;
                    }
                    0xE => {
                        self.v_reg[0xf] = (self.v_reg[x] >> 7) & 1;
                        self.v_reg[x] <<= 1;
                    }
                    _ => {
                        unreachable!("opcode {:X} is not exists.", opcode)
                    }
                }
            }
            0x9000 => {
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                if self.v_reg[x as usize] != self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }
            0xA000 => {
                let val = opcode & 0x0FFF;
                self.i_reg = val;
            }
            0xB000 => {
                self.pc = self.v_reg[0] as u16 + opcode & 0x0FFF;
            }
            0xC000 => {
                let x = (opcode & 0x0F00) >> 8;
                let mask = opcode & 0x00FF;
                let mut rng = rng();
                self.v_reg[x as usize] = (rng.random_range(0..256) & mask) as u8;
            }
            0xD000 => {
                self.v_reg[0xf] = 0;

                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                let n = opcode & 0x000F;

                let vx = self.v_reg[x as usize] as usize % SCREEN_WIDTH;
                let vy = self.v_reg[y as usize] as usize % SCREEN_HEIGHT;

                for i in 0..n as usize {
                    if vy + i == SCREEN_HEIGHT {
                        break;
                    }
                    let sprite = self.ram[self.i_reg as usize + i];
                    for j in (0..8).rev() {
                        let targ = (sprite >> j) & 1;
                        if targ == 1 {
                            let idx = vx + (7 - j) + (vy + i) * SCREEN_WIDTH;
                            self.screen[idx] = !self.screen[idx];
                            if !self.screen[idx] {
                                self.v_reg[0xf] = 1;
                            }
                        }
                    }
                }
            }
            0xE000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                match opcode & 0x00FF {
                    0x9E => {
                        if self.keys[self.v_reg[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        if !self.keys[self.v_reg[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        panic!("op code {:X} can not decode and execute", opcode)
                    }
                }
            }
            0xF000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                match opcode & 0x00FF {
                    0x07 => {
                        self.v_reg[x] = self.delay_timer;
                    }
                    0x0A => loop {
                        if let Ok(Event::Key(key_event)) = read() {
                            if key_event.kind != KeyEventKind::Press {
                                continue;
                            }

                            let key = match key_event.code {
                                KeyCode::Char('1') => 0x1,
                                KeyCode::Char('2') => 0x2,
                                KeyCode::Char('3') => 0x3,
                                KeyCode::Char('4') => 0xC,

                                KeyCode::Char('q') => 0x4,
                                KeyCode::Char('w') => 0x5,
                                KeyCode::Char('e') => 0x6,
                                KeyCode::Char('r') => 0xD,

                                KeyCode::Char('a') => 0x7,
                                KeyCode::Char('s') => 0x8,
                                KeyCode::Char('d') => 0x9,
                                KeyCode::Char('f') => 0xE,

                                KeyCode::Char('z') => 0xA,
                                KeyCode::Char('x') => 0x0,
                                KeyCode::Char('c') => 0xB,
                                KeyCode::Char('v') => 0xF,

                                KeyCode::Esc => return Err(()),

                                _ => continue,
                            };
                            self.v_reg[x] = key;
                            break;
                        }
                    },
                    0x15 => {
                        self.delay_timer = self.v_reg[x];
                    }
                    0x18 => {
                        self.sound_timer = self.v_reg[x];
                    }
                    0x1E => {
                        self.i_reg += self.v_reg[x] as u16;
                    }
                    0x29 => {
                        let val = self.v_reg[x];
                        self.i_reg = (FONT_BASE_ADDR + val as usize * 5) as u16;
                    }
                    0x33 => {
                        let mut val = self.v_reg[x];
                        for i in (0..3).rev() {
                            self.ram[self.i_reg as usize + i] = val % 10;
                            val /= 10;
                        }
                    }
                    0x55 => {
                        for i in 0..=x {
                            self.ram[self.i_reg as usize + i] = self.v_reg[i];
                        }
                    }
                    0x65 => {
                        for i in 0..=x {
                            self.v_reg[i] = self.ram[self.i_reg as usize + i];
                        }
                    }
                    _ => {
                        panic!("op code {:X} can not decode and execute", opcode)
                    }
                }
            }
            _ => {
                panic!("op code {:X} can not decode and execute", opcode)
            }
        }
        Ok(())
    }

    pub fn load_rom(&mut self, buffer: &Vec<u8>) {
        for (i, &byte) in buffer.iter().enumerate() {
            if 0x200 + i < 4096 {
                self.ram[0x200 + i] = byte;
            }
        }
    }

    pub fn reset_keys(&mut self) {
        for i in 0..NUM_KEYS {
            self.keys[i] = false;
        }
    }

    fn screen_clear(&mut self) {
        for i in 0..SCREEN_WIDTH * SCREEN_HEIGHT {
            self.screen[i] = false;
        }
    }
}
