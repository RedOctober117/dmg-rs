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
}

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
            (0x40, _) => self.ld_r8_r8(data_bits),
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
                0b100 => self.ld_c_a(),
                0b101 => self.ld_imm16_a(),
                0b110 => self.ld_a_c(),
                0b111 => self.ld_a_imm16(),
                0b000..=u8::MAX => self.jp_cond_imm16(middle_three_bits & 0b011),
            },
            (0xC0, 0b011) => match middle_three_bits {
                0b000 => self.jp_imm16(),
                0b001 => todo!(),
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

    pub fn nop(&mut self) {}

    pub fn halt(&mut self) {}
    pub fn stop(&mut self) {}

    pub fn ld_r8_r8(&mut self, suffix: u8) {
        self.general_purpose_registers[(suffix >> 3) as usize] =
            self.general_purpose_registers[(suffix & 0x07) as usize];
    }

    pub fn ld_r16_imm16(&mut self, data_bits: u8) {}

    pub fn ld_r16mem_a(&mut self, data_bits: u8) {}

    pub fn ld_a_r16mem(&mut self, data_bits: u8) {}

    pub fn ld_imm16_sp(&mut self) {}
    pub fn add_hl_r16(&mut self, data_bits: u8) {}
    pub fn inc_r8(&mut self, data_bits: u8) {}
    pub fn dec_r8(&mut self, data_bits: u8) {}
    pub fn jr_imm8(&mut self) {}
    pub fn jr_cond_imm8(&mut self, cond: u8) {}
    pub fn ld_r8_imm8(&mut self, data_bits: u8) {}

    pub fn rlca(&mut self) {}
    pub fn rrca(&mut self) {}
    pub fn rla(&mut self) {}
    pub fn rra(&mut self) {}
    pub fn daa(&mut self) {}
    pub fn cpl(&mut self) {}
    pub fn scf(&mut self) {}
    pub fn ccf(&mut self) {}

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
    pub fn ld_hl_sp_imm8(&mut self) {}
    pub fn ret_cond(&mut self) {}

    pub fn ret(&mut self) {}
    pub fn reti(&mut self) {}
    pub fn jp_hl(&mut self) {}
    pub fn ld_sp_hl(&mut self) {}
    pub fn pop_r16stk(&mut self, data_bits: u8) {}

    pub fn ld_c_a(&mut self) {}
    pub fn ld_imm16_a(&mut self) {}
    pub fn ld_a_c(&mut self) {}
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
