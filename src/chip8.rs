use rand::Rng;
use std::{fs::File, io::Read};

const START_ADDRESS: u16 = 0x200;
const FONST_SET_START_ADDRESS: u16 = 0x50;

pub const VIDEO_WIDTH: usize = 64;
pub const VIDEO_HEIGHT: usize = 32;

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

#[derive(Debug)]
pub struct Chip8 {
    registers: [u8; 16],                     // 16 8bit regisgters, V0 - VF
    memory: [u8; 4096],                      // 4K bytes memory
    index: u16,                              // 16bit index register
    pc: u16,                                 // 16bit program counter
    stack: [u16; 16],                        // 16 level stack (16*16bit)
    sp: u8,                                  // 8bit stack pointer
    delay_timer: u8,                         // 8bit delay timer
    sound_timer: u8,                         // 8bit sound timer
    keypad: [u8; 16],                        // 16 input keys
    video: [u8; VIDEO_WIDTH * VIDEO_HEIGHT], // 64x32 display memory
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: START_ADDRESS,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; VIDEO_WIDTH * VIDEO_HEIGHT],
        };

        chip8._load_fontset();

        chip8
    }

    pub fn get_pixel_at(&self, row: usize, col: usize) -> u8 {
        self.video[(row + col * VIDEO_WIDTH) % (VIDEO_HEIGHT * VIDEO_WIDTH)]
    }

    pub fn load_rom(&mut self, filename: &str) {
        let mut f = File::open(filename).expect("error reading file");

        let mut buffer: Vec<u8> = Vec::new();

        f.read_to_end(&mut buffer)
            .expect("something wrong reading file");

        for (idx, data) in buffer.iter().enumerate() {
            self.memory[START_ADDRESS as usize + idx] = data.clone();
        }
    }

    fn _load_fontset(&mut self) {
        for (idx, data) in FONTSET.iter().enumerate() {
            self.memory[FONST_SET_START_ADDRESS as usize + idx] = data.clone();
        }
    }

    fn gen_random_number(&self) -> u8 {
        let mut rng = rand::thread_rng();
        rng.gen_range(u8::MIN..u8::MAX)
    }

    pub fn set_key_state(&mut self, key_index: usize, value: u8) {
        self.keypad[key_index] = value;
    }

    pub fn cycle(&mut self) {
        let opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[self.pc as usize + 1] as u16);

        self.pc += 2;

        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    // 0x00E0 -- CLS
                    0xE0 => self.memory.iter_mut().for_each(|m| *m = 0),

                    // 0x00EE -- RET
                    0xEE => {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }

                    _ => {
                        panic!("unhandled opcode {:#x}", opcode);
                    }
                }
            }

            // 1nnn -- JP nnn
            0x1000 => {
                let address = opcode & 0x0fff;
                self.pc = address;
            }

            // 2nnn -- CALL nnn
            0x2000 => {
                let address = opcode & 0x0fff;

                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;

                self.pc = address;
            }

            // 3xkk -- Skip next instruction if Vx == kk
            0x3000 => {
                let Vx = (opcode & 0x0f00) >> 8;
                let value = opcode & 0x00ff;

                if self.registers[Vx as usize] == value as u8 {
                    self.pc += 2;
                }
            }

            // 4xkk -- Skip next instruction if Vx != kk
            0x4000 => {
                let Vx = (opcode & 0x0f00) >> 8;
                let value = opcode & 0x00ff;

                if self.registers[Vx as usize] != value as u8 {
                    self.pc += 2;
                }
            }

            // 5xy0 -- Skip next instruction if Vx == Vy
            0x5000 => {
                let Vx = (opcode & 0x0f00) >> 8;
                let Vy = (opcode & 0x00f0) >> 4;

                if self.registers[Vx as usize] == self.registers[Vy as usize] {
                    self.pc += 2;
                }
            }

            // 6xkk -- LD Vx kk
            0x6000 => {
                let Vx = (opcode & 0x0f00) >> 8;
                let value = opcode & 0x00ff;

                self.registers[Vx as usize] = value as u8;
            }

            // 7xkk -- ADD Vx, kk
            0x7000 => {
                let Vx = (opcode & 0x0f00) >> 8;
                let value = opcode & 0x00ff;

                self.registers[Vx as usize] = self.registers[Vx as usize].wrapping_add(value as u8);
            }

            0x8000 => match opcode & 0x000F {
                // 8xy0 -- LD Vx, Vy
                0x0 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    let Vy = (opcode & 0x00f0) >> 4;

                    self.registers[Vx as usize] = self.registers[Vy as usize];
                }

                // 8xy1 -- OR Vx, Vy
                0x1 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    let Vy = (opcode & 0x00f0) >> 4;

                    self.registers[Vx as usize] |= self.registers[Vy as usize];
                }

                // 8xy2 -- AND Vx, Vy
                0x2 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    let Vy = (opcode & 0x00f0) >> 4;

                    self.registers[Vx as usize] &= self.registers[Vy as usize];
                }

                // 8xy3 -- XOR Vx, Vy
                0x3 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    let Vy = (opcode & 0x00f0) >> 4;

                    self.registers[Vx as usize] ^= self.registers[Vy as usize];
                }

                // 8xy4 -- ADD Vx, Vy  VF carry
                0x4 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    let Vy = (opcode & 0x00f0) >> 4;

                    if let Some(sum) =
                        self.registers[Vx as usize].checked_add(self.registers[Vy as usize])
                    {
                        // didnt overflow
                        self.registers[0xF] = 0;
                    } else {
                        // overflow
                        self.registers[0xF] = 1;
                    }

                    self.registers[Vx as usize] =
                        self.registers[Vx as usize].wrapping_add(self.registers[Vy as usize]);
                }

                // 8xy5 -- SUB Vx, Vy  VF not borrow
                0x5 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    let Vy = (opcode & 0x00f0) >> 4;

                    if self.registers[Vx as usize] > self.registers[Vy as usize] {
                        self.registers[0xF] = 1;
                    } else {
                        self.registers[0xF] = 0;
                    }

                    self.registers[Vx as usize] =
                        self.registers[Vx as usize].wrapping_sub(self.registers[Vy as usize]);
                }

                // 8xy6 -- SHR Vx      VF = Vx.LSB
                0x6 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    self.registers[0xF] = (self.registers[Vx as usize] & 0x1);

                    self.registers[Vx as usize] >>= 1;
                }

                // 8xyE -- SHL Vx {, Vy}    VF = Vx.MSB
                0xE => {
                    let Vx = (opcode & 0x0f00) >> 8;

                    self.registers[0xF] = (self.registers[Vx as usize] & 0x80) >> 7;
                    self.registers[Vx as usize] <<= 1;
                }

                _ => panic!("unhandled opcode {:#x}", opcode),
            },

            // 9xy0 -- Skip next instruction if Vx != Vy
            0x9000 => {
                let Vx = (opcode & 0x0f00) >> 8;
                let Vy = (opcode & 0x00f0) >> 4;

                if self.registers[Vx as usize] != self.registers[Vy as usize] {
                    self.pc += 2;
                }
            }

            // Annn -- LD index nnn
            0xA000 => {
                let address = opcode & 0x0fff;
                self.index = address;
            }

            // Cxkk -- RND Vx, byte
            0xC000 => {
                let Vx = (opcode & 0x0f00) >> 8;
                let value = opcode & 0x00ff;

                let rand_byte = self.gen_random_number();

                self.registers[Vx as usize] = value as u8 & rand_byte;
            }

            // Dxyn -- display n byte sprite starting at memory[index] at (Vx,Vy) set VF to collision
            0xD000 => {
                let Vx = (opcode & 0x0f00) >> 8;
                let Vy = (opcode & 0x00f0) >> 4;

                let height = opcode & 0x000f;

                let x_pos = self.registers[Vx as usize] % VIDEO_WIDTH as u8;
                let y_pos = self.registers[Vy as usize] % VIDEO_HEIGHT as u8;

                self.registers[0xF] = 0;

                for row in 0..height {
                    let sprite_byte = self.memory[self.index as usize + row as usize];

                    for col in 0..8 {
                        let sprite_pixel = sprite_byte & (0x80 >> col);

                        let screen_pixel = self.video[((y_pos + row as u8) as usize * VIDEO_WIDTH
                            + (x_pos + col) as usize)
                            % (VIDEO_HEIGHT * VIDEO_WIDTH)];

                        if sprite_pixel != 0 {
                            if screen_pixel == 0xFF {
                                self.registers[0xF] = 1;
                            }

                            self.video[(y_pos + row as u8) as usize * VIDEO_WIDTH
                                + (x_pos + col) as usize] ^= 0xFF;
                        }
                    }
                }
            }

            0xE000 => match opcode & 0x00FF {
                // ExA1 -- Skip next instruction if key(reg[Vx]) is not pressed
                0xA1 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    let key = self.registers[Vx as usize];

                    if self.keypad[key as usize] == 0 {
                        self.pc += 2;
                    }
                }

                _ => panic!("unhandled opcode {:#x}", opcode),
            },

            0xF000 => match opcode & 0x00FF {
                // Fx07 -- LD VD Vx, DT
                0x07 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    self.registers[Vx as usize] = self.delay_timer;
                }

                // Fx15 -- LD DT, Vx
                0x15 => {
                    let Vx = (opcode & 0x0f00) >> 8;

                    self.delay_timer = self.registers[Vx as usize];
                }

                // Fx18 -- LD ST, Vx
                0x18 => {
                    let Vx = (opcode & 0x0f00) >> 8;
                    self.sound_timer = self.registers[Vx as usize];
                }

                // Fx1E -- ADD Index, Vx
                0x1E => {
                    let Vx = (opcode & 0x0f00) >> 8;

                    self.index += self.registers[Vx as usize] as u16;
                }

                // Fx29 -- LD F, Vx      Index = sprite location for digit Vx
                0x29 => {
                    let Vx = (opcode & 0x0f00) >> 8;

                    let digit = self.registers[Vx as usize];

                    self.index = FONST_SET_START_ADDRESS + (5 * digit as u16);
                }

                // Fx33 -- LD B, Vx     Store BCD(binary representation) of Vx in memory Index Index+1 Index+2
                0x33 => {
                    let Vx = (opcode & 0x0f00) >> 8;

                    let mut value = self.registers[Vx as usize];

                    self.memory[self.index as usize + 2] = value % 10;
                    value /= 10;

                    self.memory[self.index as usize + 1] = value % 10;
                    value /= 10;

                    self.memory[self.index as usize] = value % 10;
                }

                // Fx55 -- LD [I], Vx      Store V0 to Vx in memory starting at Index
                0x55 => {
                    let Vx = (opcode & 0x0f00) >> 8;

                    for x in 0..=Vx {
                        self.memory[self.index as usize + x as usize] = self.registers[x as usize];
                    }
                }
                // Fx65 -- LD Vx, [I]     Read V0 to Vx into memory starting at Index
                0x65 => {
                    let Vx = (opcode & 0x0f00) >> 8;

                    for x in 0..=Vx {
                        self.registers[x as usize] = self.memory[self.index as usize + x as usize];
                    }
                }

                _ => panic!("unhandled opcode {:#x}", opcode),
            },

            _ => {
                panic!("unhandled opcode {:#x}", opcode);
            }
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        // println!("{:#x}", opcode);
    }
}
