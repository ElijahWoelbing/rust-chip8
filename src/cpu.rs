pub struct CPU {
    v: [u8; 16],
    dt: u8,
    st: u8,
    i: u16,
    pc: u16,
    sp: u8,
    screen_buffer: [u32; 2048],
    memory: [u8; 0xfff],
    stack: [u16; 16],
    pub keys: [u8; 16],
    current_opcode: u16,
    pub update_screen: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            dt: 0,
            st: 0,
            i: 0,
            pc: 0x200,
            sp: 0,
            screen_buffer: [0; 2048],
            memory: [0; 0xfff],
            stack: [0; 16],
            keys: [0; 16],
            current_opcode: 0,
            update_screen: false,
        }
    }

    pub fn initialize(&mut self) {
        self.pc = 0x200;
        self.current_opcode = 0;
        self.i = 0;
        self.sp = 0;
        // clear stack, input and regesters
        for i in 0..16 {
            self.stack[i] = 0;
            self.keys[i] = 0;
            self.v[i] = 0;
        }
        // clear memory
        for byte in self.memory.iter_mut() {
            *byte = 0;
        }
        let sprites = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];
        // load sprites into memory
        for (i, byte) in sprites.iter().enumerate() {
            self.memory[i] = *byte;
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        let rom = std::fs::read(rom_path).expect("file not found");
        for (i, byte) in rom.into_iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }
    }

    pub fn get_screen_buffer(&self) -> &[u32; 2048] {
        &self.screen_buffer
    }

    pub fn cycle(&mut self) {
        self.current_opcode = self.fetch_opcode();

        match self.current_opcode & 0xf000 {
            0x0000 => match self.current_opcode & 0x000f {
                0x0000 => self.cls(),
                0x000e => self.ret(),
                _ => panic!("invalid opcode {:x}", self.current_opcode),
            },
            0x1000 => self.jp(),
            0x2000 => self.call(),
            0x3000 => self.se_vx_byte(),
            0x4000 => self.sne_vx_byte(),
            0x5000 => self.se_vx_vy(),
            0x6000 => self.ld_vx_byte(),
            0x7000 => self.add_vx_byte(),
            0x8000 => match self.current_opcode & 0x000f {
                0x0000 => self.ld_vx_vy(),
                0x0001 => self.or_vx_vy(),
                0x0002 => self.and_vx_vy(),
                0x0003 => self.xor_vx_vy(),
                0x0004 => self.add_vx_vy(),
                0x0005 => self.sub_vx_vy(),
                0x0006 => self.shr_vx_vy(),
                0x0007 => self.subn_vx_vy(),
                0x000e => self.shl_vx_vy(),
                _ => panic!("invalid opcode {:x}", self.current_opcode),
            },
            0x9000 => self.sne_vx_vy(),
            0xa000 => self.ld_i_addr(),
            0xb000 => self.jp_v0_addr(),
            0xc000 => self.rnd_vx_byte(),
            0xd000 => self.drw_vx_vy_nibble(),
            0xe000 => match self.current_opcode & 0x00ff {
                0x009e => self.skp_vx(),
                0x00a1 => self.sknp_vx(),
                _ => panic!("invalid opcode {:x}", self.current_opcode),
            },
            0xf000 => match self.current_opcode & 0x00ff {
                0x0007 => self.ld_vx_dt(),
                0x000a => self.ld_vx_k(),
                0x0015 => self.ld_dt_vx(),
                0x0018 => self.ld_st_vx(),
                0x001e => self.add_i_vx(),
                0x0029 => self.ld_f_vx(),
                0x0033 => self.ld_b_vx(),
                0x0055 => self.ld_addr_i_vx(),
                0x0065 => self.ld_vx_addr_i(),
                _ => panic!("invalid opcode {:x}", self.current_opcode),
            },

            _ => {
                unreachable!("how did you get here");
            }
        }
    }

    fn fetch_opcode(&mut self) -> u16 {
        let opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);
        self.pc += 2;
        opcode
    }
    // nnn or addr - A 12-bit value, the lowest 12 bits of the instruction
    // n or nibble - A 4-bit value, the lowest 4 bits of the instruction
    // x - A 4-bit value, the lower 4 bits of the high byte of the instruction
    // y - A 4-bit value, the upper 4 bits of the low byte of the instruction
    // kk or byte - An 8-bit value, the lowest 8 bits of the instruction
    fn compute_nnn(&self) -> u16 {
        self.current_opcode & 0x0fff
    }

    fn compute_n(&self) -> u16 {
        self.current_opcode & 0x000f
    }

    fn compute_x(&self) -> u16 {
        (self.current_opcode & 0x0f00) >> 8
    }

    fn compute_y(&self) -> u16 {
        (self.current_opcode & 0x00f0) >> 4
    }

    fn compute_kk(&self) -> u8 {
        (self.current_opcode & 0x00ff) as u8
    }

    fn read_vx(&self) -> u8 {
        self.v[self.compute_x() as usize]
    }
    fn read_vy(&self) -> u8 {
        self.v[self.compute_y() as usize]
    }

    fn write_vx(&mut self, value: u8) {
        self.v[self.compute_x() as usize] = value;
    }

    // 00E0 - CLS
    // Clear the display.
    fn cls(&mut self) {
        for byte in self.screen_buffer.iter_mut() {
            *byte = 0;
        }
        self.update_screen = true;
    }

    // 00EE - RET
    // Return from a subroutine.
    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    fn jp(&mut self) {
        self.pc = self.compute_nnn();
    }

    // The interpreter sets the program counter to nnn.

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    // The interpreter increments the stack pointer, then puts the current pc on the top of the stack. The pc is then set to nnn.
    fn call(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.compute_nnn();
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn se_vx_byte(&mut self) {
        if self.read_vx() == self.compute_kk() {
            self.pc += 2;
        }
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn sne_vx_byte(&mut self) {
        if self.read_vx() != self.compute_kk() {
            self.pc += 2;
        }
    }
    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn se_vx_vy(&mut self) {
        if self.read_vx() == self.read_vy() {
            self.pc += 2;
        }
    }
    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    // The interpreter puts the value kk into register Vx.
    fn ld_vx_byte(&mut self) {
        self.write_vx(self.compute_kk());
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn add_vx_byte(&mut self) {
        self.write_vx(self.read_vx().wrapping_add(self.compute_kk()));
    }
    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    fn ld_vx_vy(&mut self) {
        self.write_vx(self.read_vy());
    }
    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy.
    // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
    fn or_vx_vy(&mut self) {
        self.write_vx(self.read_vx() | self.read_vy());
    }
    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise AND compares the corrseponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
    fn and_vx_vy(&mut self) {
        self.write_vx(self.read_vx() & self.read_vy());
    }
    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    fn xor_vx_vy(&mut self) {
        self.write_vx(self.read_vx() ^ self.read_vy());
    }
    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry.
    // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn add_vx_vy(&mut self) {
        let add = self.read_vx() as u16 + self.read_vy() as u16;
        if add > 0xff {
            self.v[0xf] = 1;
        } else {
            self.v[0xf] = 0;
        }
        self.write_vx(self.read_vx().wrapping_add(self.read_vy()));
    }

    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    fn sub_vx_vy(&mut self) {
        if self.read_vx() > self.read_vy() {
            self.v[0xf] = 1;
        } else {
            self.v[0xf] = 0;
        }
        self.write_vx(self.read_vx().wrapping_sub(self.read_vy()));
    }
    // 8xy6 - SHR Vx {, Vy}
    // Set Vx = Vx SHR 1.
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    fn shr_vx_vy(&mut self) {
        self.v[0xf] = self.read_vx() & 0x1;
        self.write_vx(self.read_vx() >> 1);
    }
    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    fn subn_vx_vy(&mut self) {
        if self.read_vy() > self.read_vx() {
            self.v[0xf] = 1;
        } else {
            self.v[0xf] = 0;
        }
        self.write_vx(self.read_vy().wrapping_sub(self.read_vx()));
    }
    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    fn shl_vx_vy(&mut self) {
        self.v[0xf] = (self.read_vx() >> 7) & 1;
        self.write_vx(self.read_vx() << 1);
    }
    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    fn sne_vx_vy(&mut self) {
        if self.read_vx() != self.read_vy() {
            self.pc += 2;
        }
    }
    // Annn - LD i, addr
    // Set i = nnn.
    // The value of register i is set to nnn.
    fn ld_i_addr(&mut self) {
        self.i = self.compute_nnn();
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0.
    // The program counter is set to nnn plus the value of V0.
    fn jp_v0_addr(&mut self) {
        self.pc = self.compute_nnn() + self.v[0] as u16;
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk.
    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
    fn rnd_vx_byte(&mut self) {
        self.write_vx(self.compute_kk() & rand::random::<u8>());
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location i at (Vx, Vy), set VF = collision.
    // The interpreter reads n bytes from memory, starting at the address stored in i. These bytes are then displayed as sprites on screen_buffer at coordinates (Vx, Vy).
    // Sprites are XORed onto the existing screen_buffer. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
    // If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen_buffer.
    // See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen_buffer and sprites.
    fn drw_vx_vy_nibble(&mut self) {
        let x_pos = self.read_vx() as u16;
        let y_pos = self.read_vy() as u16;
        let sprite_bytes = self.compute_n();
        let mut sprite_byte;
        self.v[0xF] = 0;
        for sprite_row in 0..sprite_bytes {
            sprite_byte = self.memory[(self.i + sprite_row) as usize];
            for sprite_col in 0..8 {
                if (sprite_byte & (0x80 >> sprite_col)) != 0 {
                    let pixel_pos =
                        (x_pos + sprite_col + ((y_pos + sprite_row) * 64)) as usize % 2048;
                    if self.screen_buffer[pixel_pos] == 0x00FF00 {
                        self.v[0xf] = 1;
                    }
                    self.screen_buffer[pixel_pos] ^= 0x00FF00;
                }
            }
        }
        self.update_screen = true;
    }
    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, pc is increased by 2.
    fn skp_vx(&mut self) {
        if self.keys[self.read_vx() as usize] == 1 {
            self.pc += 2;
        }
    }
    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, pc is increased by 2.
    fn sknp_vx(&mut self) {
        if self.keys[self.read_vx() as usize] == 0 {
            self.pc += 2;
        }
    }
    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value.
    // The value of DT is placed into Vx.

    fn ld_vx_dt(&mut self) {
        self.write_vx(self.dt);
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    // All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn ld_vx_k(&mut self) {
        loop {
            for key in self.keys.iter() {
                if *key == 1 {
                    return;
                }
            }
        }
    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx.
    // DT is set equal to the value of Vx.
    fn ld_dt_vx(&mut self) {
        self.dt = self.read_vx();
    }

    // Fx18 - LD st, Vx
    // Set sound timer = Vx.
    // st is set equal to the value of Vx.
    fn ld_st_vx(&mut self) {
        self.st = self.read_vx();
    }

    // Fx1E - ADD i, Vx
    // Set i = i + Vx.
    // The values of i and Vx are added, and the results are stored in i.
    fn add_i_vx(&mut self) {
        let added = self.i.wrapping_add(self.read_vx() as u16);
        self.v[0xf] = match added > 0xfff {
            true => 1,
            false => 0,
        };
        self.i = added;
    }

    // Fx29 - LD F, Vx
    // Set i = location of sprite for digit Vx.
    // The value of i is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
    fn ld_f_vx(&mut self) {
        self.i = (self.read_vx() * 5) as u16;
    }
    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations i, i+1, and i+2.
    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in i, the tens digit at location i+1, and the ones digit at location i+2.
    fn ld_b_vx(&mut self) {
        let vx = self.read_vx();
        self.memory[self.i as usize] = vx / 100;
        self.memory[(self.i + 1) as usize] = (vx / 10) % 10;
        self.memory[(self.i + 2) as usize] = vx % 10;
    }

    // Fx55 - LD [i], Vx
    // Store v V0 through Vx in memory starting at location i.
    // The interpreter copies the values of v V0 through Vx into memory, starting at the address in i.
    fn ld_addr_i_vx(&mut self) {
        let x = self.compute_x();
        for i in 0..=x {
            self.memory[(self.i + i) as usize] = self.v[i as usize];
        }
    }

    // Fx65 - LD Vx, [i]
    // Read v V0 through Vx from memory starting at location i.
    // The interpreter reads values from memory starting at location i into v V0 through Vx.
    fn ld_vx_addr_i(&mut self) {
        let x = self.compute_x();
        for i in 0..=x {
            self.v[i as usize] = self.memory[(self.i + i) as usize];
        }
    }

    pub fn sub_dt(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
    }

    pub fn sub_st(&mut self) {
        if self.st > 0 {
            self.st -= 1;
        }
    }
}
