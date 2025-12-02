pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,

    stack_pointer: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],

    delay_timer: u8,
    sound_timer: u8,
}

impl Emu {
    pub fn new() -> Self {
        Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
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

    pub fn fetch(&mut self) {
        let opcode = {
            let a = self.ram[self.pc as usize] as u16;
            let b = self.ram[self.pc as usize + 1] as u16;
            (a << 8) | b
        };

        self.pc += 2;

        match opcode & 0xF000 {
            0x0000 => {
                if opcode == 0x00e0 {
                    self.screen_clear();
                } else {
                    unimplemented!("op code {} is not implement decode and execute", opcode)
                }
            }
            0x1000 => {
                let addr = opcode & 0x0FFF;
                self.pc = addr;
            }
            0x6000 => {
                let x = (opcode & 0x0F00) >> 8;
                let val = opcode & 0x00FF;
                self.v_reg[x as usize] = val as u8;
            }
            0x7000 => {
                let reg = (opcode & 0x0F00) >> 8;
                let val = opcode & 0x00FF;
                self.v_reg[reg as usize] += val as u8;
            }
            0xA000 => {
                let val = opcode & 0x0FFF;
                self.i_reg = val;
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
            _ => {
                unimplemented!("op code {:X} is not implement decode and execute", opcode)
            }
        }
    }

    pub fn load_rom(&mut self, buffer: &Vec<u8>) {
        for (i, &byte) in buffer.iter().enumerate() {
            if 0x200 + i < 4096 {
                self.ram[0x200 + i] = byte;
            }
        }
    }

    pub fn display(&self) {
        print!("\x1B[1;1H");

        // 上下の枠
        println!("┌{}┐", "─".repeat(SCREEN_WIDTH));

        for y in 0..SCREEN_HEIGHT {
            // print!("{:2}: │", y);
            print!("|");
            for x in 0..SCREEN_WIDTH {
                let index = y * SCREEN_WIDTH + x;
                if self.screen[index] {
                    print!("█");
                } else {
                    print!(" ");
                }
            }
            println!("│");
        }

        println!("└{}┘", "─".repeat(SCREEN_WIDTH));
    }

    fn screen_clear(&mut self) {
        for i in 0..SCREEN_WIDTH * SCREEN_HEIGHT {
            self.screen[i] = false;
        }
    }
}
