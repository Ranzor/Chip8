use minifb::{Key, Window, WindowOptions};
use std::fs;

struct Chip8 {
    // Memory: 4096 bytes (4KB)
    memory: [u8; 4096],

    // 16 general-purpose 8-bit registers (V0 to VF)
    // VF is often used as a flag register so should be avoided.
    registers: [u8; 16],

    // Program counter
    pc: u16,

    // Index register
    i: u16,

    // 64 x 32 display, 8 pixels per byte
    display: [u8; 256],

    // Keypad input
    keys: [bool; 16],      // Current key states
    waiting_for_key: bool, // Is CPU waiting for input?
    key_register: usize,   // Which register to store key in
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            pc: 0x200,
            i: 0,
            display: [0; 256],
            keys: [false; 16],
            waiting_for_key: false,
            key_register: 0,
        }
    }

    fn get_display_buffer(&self) -> Vec<u32> {
        let mut buffer = vec![0u32; 64 * 32];

        for y in 0..32 {
            for x in 0..64 {
                let byte_index = (y * 8) + (x / 8);
                let bit_position = 7 - (x % 8);
                let pixel_on = (self.display[byte_index] & (1 << bit_position)) != 0;

                buffer[y * 64 + x] = if pixel_on { 0xFFFFFF } else { 0x000000 };
            }
        }
        buffer
    }

    fn print_display(&self) {
        for row in 0..32 {
            for byte_in_row in 0..8 {
                let byte_index = row * 8 + byte_in_row;
                let byte = self.display[byte_index];

                for bit in 0..8 {
                    let mask = 1 << (7 - bit);
                    if (byte & mask) != 0 {
                        print!("#");
                    } else {
                        print!(".");
                    }
                }
            }
            print!("\n");
        }
    }

    fn set_keys(&mut self, window: &Window) {
        // Map keyboard keys to Chip-8 keys
        // Original Chip-8 keyboard layout:
        // 1 2 3 C
        // 4 5 6 D
        // 7 8 9 E
        // A 0 B F
        //
        // Mapped to normal keyboard:
        // 1 2 3 4
        // Q W E R
        // A S D F
        // Z X C V

        self.keys[0x1] = window.is_key_down(Key::Key1);
        self.keys[0x2] = window.is_key_down(Key::Key2);
        self.keys[0x3] = window.is_key_down(Key::Key3);
        self.keys[0xC] = window.is_key_down(Key::Key4);

        self.keys[0x4] = window.is_key_down(Key::Q);
        self.keys[0x5] = window.is_key_down(Key::W);
        self.keys[0x6] = window.is_key_down(Key::E);
        self.keys[0xD] = window.is_key_down(Key::R);

        self.keys[0x7] = window.is_key_down(Key::A);
        self.keys[0x8] = window.is_key_down(Key::S);
        self.keys[0x9] = window.is_key_down(Key::D);
        self.keys[0xE] = window.is_key_down(Key::F);

        self.keys[0xA] = window.is_key_down(Key::Z);
        self.keys[0x0] = window.is_key_down(Key::X);
        self.keys[0xB] = window.is_key_down(Key::C);
        self.keys[0xF] = window.is_key_down(Key::V);
    }

    fn load_program(&mut self, program: &[u8]) {
        for (i, &byte) in program.iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }
    }

    fn fetch(&self) -> u16 {
        let high_byte = self.memory[self.pc as usize] as u16;
        let low_byte = self.memory[(self.pc + 1) as usize] as u16;
        (high_byte << 8) | low_byte
    }

    fn execute(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => {
                    // 00E0 Clear display
                    self.display = [0; 256];
                }
                _ => println!("Unknown 0x0... opcode: {:#06X}", opcode),
            },

            0x1000 => {
                // 1NNN jumps to address NNN
                println!("Jump to PC{:#05X}", nnn);
                self.pc = nnn - 2;
            }

            0x3000 => {
                // 3XNN Skips the next instruction if VX equals NN
                if self.registers[x] == nn {
                    println!("Skipping next instruction");
                    self.pc += 2;
                } else {
                    println!("Continuing next instruction");
                }
            }

            0x4000 => {
                // 4XNN Skips the next instruction of VX does NOT equal NN
                if self.registers[x] != nn {
                    println!("Skipping next instruction");
                    self.pc += 2;
                } else {
                    println!("Continuing next instruction");
                }
            }

            0x5000 => {
                // 5XY0 Skips the next instruction of VX equals VY
                if self.registers[x] == self.registers[y] {
                    println!("Skipping next instruction");
                    self.pc += 2;
                } else {
                    println!("Continuing next instruction");
                }
            }

            0x6000 => {
                // 6XNN: Set register VX to NN
                println!("Set V{:X} = {:#04X}", x, nn);
                self.registers[x] = nn;
            }

            0x7000 => {
                // 7XNN: Add NN to register VX
                println!("Add {:#04X} to V{:X}", nn, x);
                self.registers[x] = self.registers[x].wrapping_add(nn);
            }

            0x8000 => {
                // 8XY_: Register operations
                match opcode & 0x000F {
                    0x0002 => {
                        // 8XY2: Bitwise VX AND VY
                        let result = self.registers[x] & self.registers[y];
                        self.registers[x] = result;
                    }

                    0x0004 => {
                        // 8XY4: ADD VY to VX, set VF = carry
                        println!("V{:X} += V{:X}", x, y);
                        let (result, overflow) =
                            self.registers[x].overflowing_add(self.registers[y]);
                        self.registers[x] = result;
                        self.registers[0xF] = if overflow { 1 } else { 0 };
                    }

                    _ => println!("Unknown 8XY_ opcode: {:#06X}", opcode),
                }
            }

            0x9000 => {
                // 9XY0 Skips next instruction of VX does NOT equal VY
                if self.registers[x] != self.registers[y] {
                    println!("Skipping next instruction");
                    self.pc += 2;
                } else {
                    println!("Continuing next instruction");
                }
            }

            0xA000 => {
                // ANNN: Set index register I to NNN
                println!("Set I = {:#05X}", nnn);
                self.i = nnn;
            }

            0xD000 => {
                // DXYN Draw display
                let x = (self.registers[x] % 64) as usize;
                let y = (self.registers[y] % 32) as usize;
                let height = n;
                let shift = x % 8;

                self.registers[0xF] = 0; // Reset collision flag

                for row in 0..height {
                    let sprite_byte = self.memory[(self.i + row as u16) as usize];
                    let display_row = (y + row as usize) % 32;
                    let display_byte_index = (display_row * 8) + (x / 8);

                    let old = self.display[display_byte_index];
                    self.display[display_byte_index] ^= sprite_byte >> shift;

                    if old != 0 && self.display[display_byte_index] < old {
                        self.registers[0xF] = 1;
                    }

                    if shift != 0 && (x + 8) < 64 {
                        let old = self.display[display_byte_index + 1];
                        self.display[display_byte_index + 1] ^= sprite_byte << (8 - shift);

                        if old != 0 && self.display[display_byte_index + 1] < old {
                            self.registers[0xF] = 1;
                        }
                    }
                }
            }

            0xF000 => {
                match opcode & 0x00FF {
                    0x0A => {
                        // FX0A: wait for key press
                        self.waiting_for_key = true;
                        self.key_register = x;
                        self.pc -= 2;
                    }
                    _ => println!("Unknown 0xF... opcode: {:#06X}", opcode),
                }
            }

            _ => {
                println!("Unknown opcode: {:#06X}", opcode);
            }
        }
    }

    fn cycle(&mut self) {
        if self.waiting_for_key {
            for (i, &pressed) in self.keys.iter().enumerate() {
                if pressed {
                    self.registers[self.key_register] = i as u8;
                    self.waiting_for_key = false;
                    self.pc += 2;
                    break;
                }
            }
            return;
        }

        let opcode = self.fetch();

        self.execute(opcode);

        // each instruction is 2 bytes
        self.pc += 2;
    }

    fn print_state(&self) {
        println!("\n--- CPU State ---");
        println!("PC: {:#05X}", self.pc);
        println!("I: {:#05X}", self.i);
        print!("Registers: ");
        for (i, &val) in self.registers.iter().enumerate() {
            print!("V{:X}={:#04X} ", i, val);
            if i == 7 {
                print!("\n           ");
            }
        }
        println!("\n");
    }
}

fn main() {
    println!("=== Chip-8 Emulator - Starting ===\n");

    let mut chip8 = Chip8::new();

    let mut window = Window::new("Chip-8 Emulator", 640, 320, WindowOptions::default())
        .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // Read the ROM file
    let rom = fs::read("life.ch8").expect("Failed to read ROM file");

    // Load it into memory
    chip8.load_program(&rom);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.set_keys(&window);

        for _ in 0..12 {
            chip8.cycle();
            chip8.print_state();
        }

        let buffer = chip8.get_display_buffer();
        window.update_with_buffer(&buffer, 64, 32).unwrap();
    }
}
