use rand;
struct CPU {
    registers: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
    index: u16,
    program_counter: u16,
    stack_pointer: u8,
    screen_buffer: [u8; 2048],
    memory: [u8; 0xfff],
    stack: [u16; 16],
    current_opcode: u16,
}

impl CPU {
    fn new() -> Self {
        let cpu = Self {
            registers: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            index: 0,
            program_counter: 0x200,
            stack_pointer: 0,
            screen_buffer: [0; 2048],
            memory: [0; 0xfff],
            stack: [0; 16],
            current_opcode: 0,
        };
        let sprites = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];

        for (i, byte) in sprites.iter().enumerate() {
            cpu.memory[i] = *byte;
        }

        cpu
    }

    fn cycle(&mut self) {
        self.current_opcode = self.fetch_opcode();

        match self.current_opcode & 0xf000 {
            0x0000 => match self.current_opcode & 0x000f {
                0x0000 => self.clear_screen(),
                _ => self.ret(),
            },
            0x1000 => {
                self.jump();
            }
            0x2000 => {
                self.call();
            }
            0x3000 => {
                self.skip_equal_low_byte();
            }
            0x4000 => {
                self.skip_not_equal_low_byte();
            }
        }
    }

    fn fetch_opcode(&mut self) -> u16 {
        let opcode = (self.memory[self.program_counter as usize] as u16) << 8
            | (self.memory[(self.program_counter + 1) as usize] as u16);
        self.program_counter += 2;
        opcode
    }
    // nnn or addr - A 12-bit value, the lowest 12 bits of the instruction
    // n or nibble - A 4-bit value, the lowest 4 bits of the instruction
    // x - A 4-bit value, the lower 4 bits of the high byte of the instruction
    // y - A 4-bit value, the upper 4 bits of the low byte of the instruction
    // kk or byte - An 8-bit value, the lowest 8 bits of the instruction
    fn opcode_low_12_bits(&self) -> u16 {
        self.current_opcode & 0x0fff
    }

    fn opcode_low_byte_low_nibble(&self) -> u16 {
        self.current_opcode & 0x000f
    }

    fn opcode_high_byte_low_nibble(&self) -> u16 {
        self.current_opcode & 0x0f00 >> 8
    }

    fn opcode_low_byte_upper_nibble(&self) -> u16 {
        self.current_opcode & 0x00f0 >> 4
    }

    fn opcode_low_byte(&self) -> u8 {
        (self.current_opcode & 0x00ff) as u8
    }

    fn read_Vx(&self) -> u8 {
        self.registers[self.opcode_high_byte_low_nibble() as usize]
    }
    fn read_Vy(&self) -> u8 {
        self.registers[self.opcode_low_byte_upper_nibble() as usize]
    }

    fn write_Vx(&mut self, value: u8) {
        self.registers[self.opcode_high_byte_low_nibble() as usize] = value;
    }
    fn write_Vy(&mut self, value: u8) {
        self.registers[self.opcode_low_byte_upper_nibble() as usize] = value
    }

    //     00E0 - CLS
    // Clear the display.
    fn clear_screen(&mut self) {
        for byte in self.screen_buffer.into_iter() {
            *byte = 0;
        }
    }

    // 00EE - RET
    // Return from a subroutine.
    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn ret(&mut self) {
        self.program_counter = self.stack[self.stack_pointer as usize];
        self.stack_pointer -= 1;
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    fn jump(&mut self) {
        self.program_counter = self.opcode_low_12_bits();
    }

    // The interpreter sets the program counter to nnn.

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    // The interpreter increments the stack pointer, then puts the current program_counter on the top of the stack. The program_counter is then set to nnn.
    fn call(&mut self) {
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = self.opcode_low_12_bits();
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn skip_equal_low_byte(&mut self) {
        if self.read_Vx() == self.opcode_low_byte() {
            self.program_counter += 2;
        }
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn skip_not_equal_low_byte(&mut self) {
        if self.read_Vx() != self.opcode_low_byte() {
            self.program_counter += 2;
        }
    }
    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn skip_equal_Vx_Vy(&mut self) {
        if self.read_Vx() == self.read_Vy() {
            self.program_counter += 2;
        }
    }
    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    // The interpreter puts the value kk into register Vx.
    fn ld_low_byte(&mut self) {
        self.write_Vx(self.opcode_low_byte());
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn add_low_byte(&mut self) {
        self.write_Vx(self.read_Vx() + self.opcode_low_byte());
    }
    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    fn load_Vx_Vy(&mut self) {
        self.write_Vx(self.read_Vy());
    }
    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy.
    // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
    fn or_Vx_Vy(&mut self) {
        self.write_Vx(self.read_Vx() | self.read_Vy());
    }
    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise AND compares the corrseponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
    fn and_Vx_Vy(&mut self) {
        self.write_Vx(self.read_Vx() & self.read_Vy());
    }
    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    fn xor_Vx_Vy(&mut self) {
        self.write_Vx(self.read_Vx() ^ self.read_Vy());
    }
    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry.
    // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn add_Vx_Vy(&mut self) {
        let add = self.read_Vx() as u16 + self.read_Vy() as u16;
        if add > 0xff {
            self.registers[0xf] = 1;
        } else {
            self.registers[0xf] = 0;
        }
        self.write_Vx(self.read_Vx() + add as u8);
    }

    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    fn sub_Vx_Vy(&mut self) {
        if self.read_Vx() > self.read_Vy() {
            self.registers[0xf] = 1;
        } else {
            self.registers[0xf] = 0;
        }
        self.write_Vx(self.read_Vx().wrapping_sub(self.read_Vy()));
    }
    // 8xy6 - SHR Vx {, Vy}
    // Set Vx = Vx SHR 1.
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    fn shift_Vx_right(&mut self) {
        self.registers[0xf] = self.read_Vx() & 0x1;
        self.write_Vx(self.read_Vx() >> 1);
    }
    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    fn sub_Vy_Vx(&mut self) {
        if self.read_Vy() > self.read_Vx() {
            self.registers[0xf] = 1;
        } else {
            self.registers[0xf] = 0;
        }
        self.write_Vx(self.read_Vy() - self.read_Vx());
    }
    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    fn shift_Vx_left(&mut self) {
        self.registers[0xf] = self.read_Vx() & 0x80;
        self.write_Vx(self.read_Vx() << 1);
    }
    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    fn skip_not_equal_Vx_Vy(&mut self) {
        if self.read_Vx() != self.read_Vy() {
            self.program_counter += 2;
        }
    }
    // Annn - LD i, addr
    // Set i = nnn.
    // The value of register i is set to nnn.
    fn load_index_addr(&mut self) {
        self.index = self.opcode_low_12_bits();
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0.
    // The program counter is set to nnn plus the value of V0.
    fn jump_V0(&mut self) {
        self.program_counter = self.opcode_low_12_bits() + self.registers[0] as u16;
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk.
    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
    fn rand(&mut self) {
        self.write_Vx(self.opcode_low_byte() & rand::random::<u8>());
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location i at (Vx, Vy), set VF = collision.
    // The interpreter reads n bytes from memory, starting at the address stored in i. These bytes are then displayed as sprites on screen_buffer at coordinates (Vx, Vy).
    // Sprites are XORed onto the existing screen_buffer. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
    // If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen_buffer.
    // See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen_buffer and sprites.
    fn draw(&mut self) {}
    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, program_counter is increased by 2.
    fn skip_key_down() {}
    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, program_counter is increased by 2.
    fn skip_key_up() {}
    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value.
    // The value of DT is placed into Vx.

    fn load_Vx_delay_timer(&mut self) {
        self.write_Vx(self.delay_timer);
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    // All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn wait_for_key_down(&mut self){

    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx.
    // DT is set equal to the value of Vx.
    fn load_delay_timer_Vx(&mut self) {
        self.delay_timer = self.read_Vx();
    }

    // Fx18 - LD st, Vx
    // Set sound timer = Vx.
    // st is set equal to the value of Vx.
    fn load_sound_timer_Vx(&mut self) {
        self.sound_timer = self.read_Vx();
    }

    // Fx1E - ADD i, Vx
    // Set i = i + Vx.
    // The values of i and Vx are added, and the results are stored in i.
    fn add_index_Vx(&mut self) {
        self.index = self.index + self.read_Vx() as u16;
    }

    // Fx29 - LD F, Vx
    // Set i = location of sprite for digit Vx.
    // The value of i is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
    fn load_index_sprite_Vx(&mut self){

    }
    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations i, i+1, and i+2.

    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in i, the tens digit at location i+1, and the ones digit at location i+2.

    // Fx55 - LD [i], Vx
    // Store registers V0 through Vx in memory starting at location i.

    // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in i.

    // Fx65 - LD Vx, [i]
    // Read registers V0 through Vx from memory starting at location i.

    // The interpreter reads values from memory starting at location i into registers V0 through Vx.
}
