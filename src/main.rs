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
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            pc: 0x200,
            i: 0,
        }
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
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match opcode & 0xF000 {
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

            _ => {
                println!("Unknown opcode: {:#06X}", opcode);
            }
        }
    }

    fn cycle(&mut self) {
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

    // Hardcoded test program
    let program: Vec<u8> = vec![
        0x6A, 0x00, // set VA = 0
        0x6B, 0x01, // VB = 1
        0x6C, 0x0A, // VC = 10
        0x6D, 0x0B, // VD == 1 (stop value)
        0x8A, 0xB4, // VA = VA + VB
        0x7B, 0x01, // VB =+ 1
        0x5B, 0xD0, // if VB == VD skip next instruction
        0x12, 0x08, // jump to 0x208
        0x6F, 0xFF, // VF = 0xFF (halt)
    ];

    chip8.load_program(&program);
    let mut i = 1;
    loop {
        println!("=== Cycle {} ===", i);
        chip8.cycle();
        chip8.print_state();

        if chip8.registers[0xF] == 0xFF {
            println!("\n=== Halt signal detected ===");
            break;
        }

        i += 1;
    }

    println!("=== Program Complete ===");
    println!("Final VA value: {}", chip8.registers[0xA]);
}
