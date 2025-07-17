use std::u8;

const GAME_MEM_START: usize = 0x0100;
const JOYPAD_INPUT: usize = 0xFF00;

pub struct Dmg {
    ram: [u8; 64_000],
    program_counter: u16,
    stack_pointer: u16,
    accumulator: u8,
    flags_register: u8,
    instruction_register: u8,
    interrupt_enable_register: u8,
    // B | C
    // D | E
    // H | L
    general_purpose_registers: [u8; 6],
    ime_flag: u8,
}

const REG_B: usize = 0;
const REG_C: usize = 1;
const REG_D: usize = 2;
const REG_E: usize = 3;
const REG_H: usize = 4;
const REG_L: usize = 5;

impl Dmg {
    pub fn cycle(&mut self) {
        let opcode = self.ram[self.program_counter as usize];

        let block = opcode & 0b11000000;
        let data_bits = opcode & 0b00111111;

        let end_three_bits = data_bits & 0b00000111;
        let middle_three_bits = (data_bits & 0b00111000) >> 3;

        self.program_counter += 1;

        match (block, end_three_bits) {
            // (0x00, 0x00) => self.nop(),
            (0x00, 0b000) => match middle_three_bits {
                0b000 => self.nop(),
                0b001 => self.ld_imm16_sp(),
                0b011 => self.jr_imm8(),
                0b010 => self.stop(),
                0b100..=u8::MAX => self.jr_cond_imm8(middle_three_bits & 0b011),
            },
            (0x00, 0b001) => match middle_three_bits & 0b001 {
                0 => self.ld_r16_imm16(middle_three_bits >> 1),
                1..=u8::MAX => self.add_hl_r16(middle_three_bits >> 1),
            },
            (0x00, 0b010) => match middle_three_bits & 0b001 {
                0 => self.ld_r16mem_a(middle_three_bits >> 1),
                1..=u8::MAX => self.ld_a_r16mem(middle_three_bits >> 1),
            },
            (0x00, 0b011) => match middle_three_bits & 0b001 {
                0 => self.inc_r16(middle_three_bits >> 1),
                1..=u8::MAX => self.dec_r16(middle_three_bits >> 1),
            },
            (0x00, 0b100) => self.inc_r8(middle_three_bits),
            (0x00, 0b101) => self.dec_r8(middle_three_bits),
            (0x00, 0b110) => self.ld_r8_imm8(middle_three_bits),
            (0x00, 0b111) => match middle_three_bits {
                0b000 => self.rlca(),
                0b001 => self.rrca(),
                0b010 => self.rla(),
                0b011 => self.rra(),
                0b100 => self.daa(),
                0b101 => self.cpl(),
                0b110 => self.scf(),
                0b111 => self.ccf(),
                _ => {}
            },

            // 0b01
            (0x40, 0b00110110) => self.halt(),
            (0x40, _) => self.ld_r8_r8(middle_three_bits, end_three_bits),

            // 0b10
            (0x80, _) => match middle_three_bits {
                0b000 => self.add_a_r8(end_three_bits),
                0b001 => self.adc_a_r8(end_three_bits),
                0b010 => self.sub_a_r8(end_three_bits),
                0b011 => self.sbc_a_r8(end_three_bits),
                0b100 => self.and_a_r8(end_three_bits),
                0b101 => self.xor_a_r8(end_three_bits),
                0b110 => self.or_a_r8(end_three_bits),
                0b111 => self.cp_a_r8(end_three_bits),
                _ => {}
            },

            // 0b11
            (0xC0, 000) => match middle_three_bits {
                0b100 => self.ldh_imm8_a(),
                0b101 => self.add_sp_imm8(),
                0b110 => self.ldh_a_imm8(),
                0b111 => self.ld_hl_sp_imm8(),
                0b000..=u8::MAX => self.ret_cond(),
            },
            (0xC0, 0b001) => match middle_three_bits {
                0b001 => self.ret(),
                0b011 => self.reti(),
                0b101 => self.jp_hl(),
                0b111 => self.ld_sp_hl(),
                0b000..=u8::MAX => self.pop_r16stk(middle_three_bits >> 1),
            },
            (0xC0, 0b010) => match middle_three_bits {
                0b100 => self.ldh_c_a(),
                0b101 => self.ld_imm16_a(),
                0b110 => self.ldh_a_c(),
                0b111 => self.ld_a_imm16(),
                0b000..=u8::MAX => self.jp_cond_imm16(middle_three_bits & 0b011),
            },
            (0xC0, 0b011) => match middle_three_bits {
                0b000 => self.jp_imm16(),
                0b001 => self.prefix_table(),
                0b110 => self.di(),
                0b111 => self.ei(),
                _ => {}
            },
            (0xC0, 0b100) => self.call_cond_imm16(middle_three_bits & 0b011),
            (0xC0, 0b101) => match middle_three_bits {
                0b001 => self.call_imm16(),
                0b000..=u8::MAX => self.push_r16stk(middle_three_bits >> 1),
            },
            (0xC0, 0b110) => match middle_three_bits {
                0b000 => self.add_a_imm8(),
                0b001 => self.adc_a_imm8(),
                0b010 => self.sub_a_imm8(),
                0b011 => self.sbc_a_imm8(),
                0b100 => self.and_a_imm8(),
                0b101 => self.xor_a_imm8(),
                0b110 => self.or_a_imm8(),
                0b111 => self.cp_a_imm8(),
                _ => {}
            },

            (0xC0, 0b111) => self.rst_tgt3(middle_three_bits),
            _ => {}
        }
    }

    // set when values are outside bounds of u8 or u16, depending on context
    pub fn set_carry_flag(&mut self, carried: bool) {
        if carried {
            self.flags_register |= 0b0001_0000
        } else {
            self.flags_register &= 0b1110_1111
        };
    }

    pub fn set_subtract_flag(&mut self, subtracted: bool) {
        if subtracted {
            self.flags_register |= 0b0100_0000
        } else {
            self.flags_register &= 0b1011_1111
        };
    }

    pub fn set_half_carry_flag(&mut self, carried: bool) {
        if carried {
            self.flags_register |= 0b0010_0000
        } else {
            self.flags_register &= 0b1101_1111
        };
    }

    pub fn set_zero_flag(&mut self, zero: bool) {
        if zero {
            self.flags_register |= 0b1000_0000
        } else {
            self.flags_register &= 0b0111_1111
        };
    }

    pub fn prefix_table(&mut self) {}

    pub fn nop(&mut self) {}

    pub fn halt(&mut self) {
        self.ime_flag = 0;
    }

    // COMPLETED ABOVE
    pub fn stop(&mut self) {}

    pub fn ld_r8_r8(&mut self, middle_three_bits: u8, end_three_bits: u8) {
        let source_value = match end_three_bits {
            0..=5 => self.general_purpose_registers[middle_three_bits as usize],
            6 => {
                self.general_purpose_registers[to_u16(
                    self.general_purpose_registers[REG_L],
                    self.general_purpose_registers[REG_H],
                ) as usize]
            }
            7 => self.accumulator,
            _ => todo!(),
        };

        let destination = match middle_three_bits {
            0..=5 => &mut self.general_purpose_registers[middle_three_bits as usize],
            6 => {
                &mut self.general_purpose_registers[to_u16(
                    self.general_purpose_registers[REG_L],
                    self.general_purpose_registers[REG_H],
                ) as usize]
            }
            7 => &mut self.accumulator,
            _ => todo!(),
        };

        *destination = source_value;
    }

    pub fn ld_r16_imm16(&mut self, data_bits: u8) {
        // 16b values are always loaded as little endian
        // LSB
        match data_bits {
            0 => {
                self.general_purpose_registers[REG_B] = self.ram[self.program_counter as usize];
                self.program_counter += 1;

                // MSB
                self.general_purpose_registers[REG_C] = self.ram[self.program_counter as usize];
                self.program_counter += 1;
            }
            1 => {
                self.general_purpose_registers[REG_D] = self.ram[self.program_counter as usize];
                self.program_counter += 1;

                // MSB
                self.general_purpose_registers[REG_E] = self.ram[self.program_counter as usize];
                self.program_counter += 1;
            }
            2 => {
                self.general_purpose_registers[REG_H] = self.ram[self.program_counter as usize];
                self.program_counter += 1;

                // MSB
                self.general_purpose_registers[REG_L] = self.ram[self.program_counter as usize];
                self.program_counter += 1;
            }
            3 => {
                let lsb = self.ram[self.program_counter as usize];
                self.program_counter += 1;

                // MSB
                let msb = self.ram[self.program_counter as usize];
                self.program_counter += 1;

                self.stack_pointer = to_u16(msb, lsb);
            }
            _ => {}
        }
    }

    // POSSIBLE BUG: registers may need to be loaded as little endian, currently big endian
    pub fn ld_r16mem_a(&mut self, data_bits: u8) {
        match data_bits {
            0 => {
                self.general_purpose_registers[REG_B] = 0x00;
                self.general_purpose_registers[REG_C] = self.accumulator;
            }
            1 => {
                self.general_purpose_registers[REG_D] = 0x00;
                self.general_purpose_registers[REG_E] = self.accumulator;
            }
            2 => {
                self.general_purpose_registers[REG_H] = 0x00;
                self.general_purpose_registers[REG_L] = self.accumulator + 1;
            }
            3 => {
                self.general_purpose_registers[REG_H] = 0x00;
                self.general_purpose_registers[REG_L] = self.accumulator - 1;
            }
            _ => {}
        }
    }

    // POSSIBLE BUG: endianess
    pub fn ld_a_r16mem(&mut self, data_bits: u8) {
        match data_bits {
            0 => {
                self.accumulator = self.ram[to_u16(
                    self.general_purpose_registers[REG_C],
                    self.general_purpose_registers[REG_B],
                ) as usize];
            }
            1 => {
                self.accumulator = self.ram[to_u16(
                    self.general_purpose_registers[REG_E],
                    self.general_purpose_registers[REG_D],
                ) as usize];
            }
            2 => {
                self.accumulator = self.ram[to_u16(
                    self.general_purpose_registers[REG_H],
                    self.general_purpose_registers[REG_L],
                ) as usize]
                    + 1;
            }
            3 => {
                self.accumulator = self.ram[to_u16(
                    self.general_purpose_registers[REG_H],
                    self.general_purpose_registers[REG_L],
                ) as usize]
                    - 1;
            }
            _ => {}
        }
    }

    pub fn ld_imm16_sp(&mut self) {
        let lsb = self.ram[self.program_counter as usize];
        self.program_counter += 1;
        let msb = self.ram[self.program_counter as usize];
        self.program_counter += 1;

        let addr = to_u16(msb, lsb) as usize;

        self.ram[addr] = (self.stack_pointer & 0x00FF) as u8;
        self.ram[addr + 1] = ((self.stack_pointer & 0xFF00) >> 8) as u8;
    }

    pub fn add_hl_r16(&mut self, data_bits: u8) {
        let hl = to_u16(
            self.general_purpose_registers[REG_H],
            self.general_purpose_registers[REG_L],
        );
        let value = match data_bits {
            0 => to_u16(
                self.general_purpose_registers[REG_C],
                self.general_purpose_registers[REG_B],
            ),
            1 => to_u16(
                self.general_purpose_registers[REG_E],
                self.general_purpose_registers[REG_D],
            ),
            2 => to_u16(
                self.general_purpose_registers[REG_L],
                self.general_purpose_registers[REG_H],
            ),
            3 => self.stack_pointer,
            _ => todo!(),
        };

        let (sum, overflowed) = hl.overflowing_add(value);

        // MSB goes in higher letter MAYBE?
        self.general_purpose_registers[REG_H] = ((sum & 0xFF00) >> 8) as u8;
        self.general_purpose_registers[REG_L] = (sum & 0x00FF) as u8;

        self.set_carry_flag(overflowed);
    }

    pub fn inc_r8(&mut self, data_bits: u8) {
        let register = match data_bits {
            0..=5 => &mut self.general_purpose_registers[data_bits as usize],

            6 => {
                &mut self.general_purpose_registers[to_u16(
                    self.general_purpose_registers[REG_L],
                    self.general_purpose_registers[REG_H],
                ) as usize]
            }
            7 => &mut self.accumulator,
            _ => todo!(),
        };

        let (sum, overflowed) = register.overflowing_add(1);
        *register = sum;

        self.set_carry_flag(overflowed);
        self.set_subtract_flag(false);
        if sum == 0 {
            self.set_zero_flag(true);
        }
    }

    pub fn dec_r8(&mut self, data_bits: u8) {
        let (sum, overflowed) =
            self.general_purpose_registers[data_bits as usize].overflowing_sub(1);

        self.general_purpose_registers[data_bits as usize] = sum;
        self.set_carry_flag(overflowed);
    }

    pub fn jr_imm8(&mut self) {
        let destination = self.ram[self.program_counter as usize] as i8;
        self.program_counter += 1;
        let (sum, overflowed) = self
            .program_counter
            .overflowing_add_signed(destination as i16);
        self.program_counter = sum;

        self.set_carry_flag(overflowed);
    }

    pub fn jr_cond_imm8(&mut self, cond: u8) {
        let destination = self.ram[self.program_counter as usize] as i8;
        self.program_counter += 1;
        if cond != 0 {
            let (sum, overflowed) = self
                .program_counter
                .overflowing_add_signed(destination as i16);
            self.program_counter = sum;

            self.set_carry_flag(overflowed);
        }
    }

    pub fn ld_r8_imm8(&mut self, data_bits: u8) {
        self.general_purpose_registers[data_bits as usize] =
            self.ram[self.program_counter as usize];
        self.program_counter += 1;
    }

    pub fn rlca(&mut self) {
        if self.accumulator == u8::MAX {
            self.set_carry_flag(true);
        }
        self.accumulator <<= 1;
    }

    pub fn rrca(&mut self) {
        if self.accumulator & 1 != 0 {
            self.set_carry_flag(true);
        }
        self.accumulator >>= 1;
    }

    pub fn rla(&mut self) {
        self.accumulator <<= 1;
    }

    pub fn rra(&mut self) {
        self.accumulator >>= 1;
    }

    pub fn daa(&mut self) {
        let mut adjustment = 0;
        if self.flags_register & 0b0100_0000 != 0 {
            if self.flags_register & 0b0010_0000 != 0 {
                adjustment += 0x6;
            }
            if self.flags_register & 0b001_0000 != 0 {
                adjustment += 0x60;
            }
        } else {
            if (self.flags_register & 0b0010_0000 != 0) | (self.accumulator & 0xF > 0x9) {
                adjustment += 0x6;
            }
            if (self.flags_register & 0b001_0000 != 0) | (self.accumulator > 0x99) {
                adjustment += 0x60;
                self.set_carry_flag(true);
            }
        }
        let result = self.accumulator - adjustment;
        self.set_zero_flag(result == 0);
        self.accumulator = result;
    }

    pub fn cpl(&mut self) {
        self.accumulator = !self.accumulator;
        self.set_subtract_flag(true);
        self.set_half_carry_flag(true);
    }

    pub fn scf(&mut self) {
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(true);
    }

    pub fn ccf(&mut self) {
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.flags_register ^= 0b0001_0000;
    }

    pub fn inc_r16(&mut self, data_bits: u8) {
        match data_bits {
            0 => {
                let byte = to_u16(
                    self.general_purpose_registers[REG_C],
                    self.general_purpose_registers[REG_B],
                )
                .wrapping_add(1);
                self.general_purpose_registers[REG_B] = (byte & 0x00FF) as u8;
                self.general_purpose_registers[REG_C] = (byte & 0xFF00) as u8 >> 8;
            }
            1 => {
                let byte = to_u16(
                    self.general_purpose_registers[REG_E],
                    self.general_purpose_registers[REG_D],
                )
                .wrapping_add(1);
                self.general_purpose_registers[REG_D] = (byte & 0x00FF) as u8;
                self.general_purpose_registers[REG_E] = (byte & 0xFF00) as u8 >> 8;
            }
            2 => {
                let byte = to_u16(
                    self.general_purpose_registers[REG_L],
                    self.general_purpose_registers[REG_H],
                )
                .wrapping_add(1);
                self.general_purpose_registers[REG_L] = (byte & 0x00FF) as u8;
                self.general_purpose_registers[REG_H] = (byte & 0xFF00) as u8 >> 8;
            }
            3 => {
                let byte = to_u16(
                    self.general_purpose_registers[REG_L],
                    self.general_purpose_registers[REG_H],
                )
                .wrapping_add(1);
                self.stack_pointer = byte;
            }
            _ => {}
        }
    }

    pub fn dec_r16(&mut self, data_bits: u8) {
        match data_bits {
            0 => {
                let byte = to_u16(
                    self.general_purpose_registers[REG_C],
                    self.general_purpose_registers[REG_B],
                )
                .wrapping_sub(1);
                self.general_purpose_registers[REG_B] = (byte & 0x00FF) as u8;
                self.general_purpose_registers[REG_C] = (byte & 0xFF00) as u8 >> 8;
            }
            1 => {
                let byte = to_u16(
                    self.general_purpose_registers[REG_E],
                    self.general_purpose_registers[REG_D],
                )
                .wrapping_sub(1);
                self.general_purpose_registers[REG_D] = (byte & 0x00FF) as u8;
                self.general_purpose_registers[REG_E] = (byte & 0xFF00) as u8 >> 8;
            }
            2 => {
                let byte = to_u16(
                    self.general_purpose_registers[REG_L],
                    self.general_purpose_registers[REG_H],
                )
                .wrapping_sub(1);
                self.general_purpose_registers[REG_L] = (byte & 0x00FF) as u8;
                self.general_purpose_registers[REG_H] = (byte & 0xFF00) as u8 >> 8;
            }
            3 => {
                let byte = to_u16(
                    self.general_purpose_registers[REG_L],
                    self.general_purpose_registers[REG_H],
                )
                .wrapping_sub(1);
                self.stack_pointer = byte;
            }
            _ => {}
        }
    }

    pub fn add_a_r8(&mut self, data_bits: u8) {}
    pub fn adc_a_r8(&mut self, data_bits: u8) {}
    pub fn sub_a_r8(&mut self, data_bits: u8) {}
    pub fn sbc_a_r8(&mut self, data_bits: u8) {}
    pub fn and_a_r8(&mut self, data_bits: u8) {}
    pub fn xor_a_r8(&mut self, data_bits: u8) {}
    pub fn or_a_r8(&mut self, data_bits: u8) {}
    pub fn cp_a_r8(&mut self, data_bits: u8) {}

    pub fn adc_a_imm8(&mut self) {}
    pub fn add_a_imm8(&mut self) {}
    pub fn sub_a_imm8(&mut self) {}
    pub fn sbc_a_imm8(&mut self) {}
    pub fn and_a_imm8(&mut self) {}
    pub fn xor_a_imm8(&mut self) {}
    pub fn or_a_imm8(&mut self) {}
    pub fn cp_a_imm8(&mut self) {}

    pub fn ldh_imm8_a(&mut self) {}
    pub fn add_sp_imm8(&mut self) {}
    pub fn ldh_a_imm8(&mut self) {}
    pub fn ld_hl_sp_imm8(&mut self) {}
    pub fn ret_cond(&mut self) {}

    pub fn ret(&mut self) {}
    pub fn reti(&mut self) {}
    pub fn jp_hl(&mut self) {}
    pub fn ld_sp_hl(&mut self) {}
    pub fn pop_r16stk(&mut self, data_bits: u8) {}

    pub fn ldh_c_a(&mut self) {}
    pub fn ld_imm16_a(&mut self) {}
    pub fn ldh_a_c(&mut self) {}
    pub fn ld_a_imm16(&mut self) {}
    pub fn jp_cond_imm16(&mut self, data_bits: u8) {}

    pub fn call_cond_imm16(&mut self, data_bits: u8) {}

    pub fn call_imm16(&mut self) {}
    pub fn push_r16stk(&mut self, data_bits: u8) {}

    pub fn rst_tgt3(&mut self, data_bits: u8) {}

    pub fn jp_imm16(&mut self) {}
    pub fn di(&mut self) {}
    pub fn ei(&mut self) {}
    // pub fn x00xxx110(&mut self) {}

    // pub fn x00xx0001(&mut self, x_0: u8, x_1: u8) {
    //     self.general_purpose_registers[x_0 as usize] = self.ram[self.program_counter as usize];
    //     self.program_counter += 1;

    //     self.general_purpose_registers[x_1 as usize] = self.ram[self.program_counter as usize];
    //     self.program_counter += 1;
    // }

    pub fn load_rom(&mut self) {}
}

pub fn to_u16(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | (lsb as u16)
}
