const GAME_MEM_START: usize = 0x100;

pub struct Dmg {
    ram: [u8; 64_000],
    program_counter: u16,
    stack_pointer: u16,
    accumulator: u8,
    flags_register: u8,
    instruction_register: u8,
    interrupt_enable_register: u8,
    // cartridge: Option<GamePak>,
    // B | C
    // D | E
    // H | L
    general_purpose_registers: [u8; 6],
}

impl Dmg {
    pub fn cycle(&mut self) {
        self.program_counter += 1;
    }

    pub fn x01xxxyyy<T: Into<usize>>(&mut self, x: T, y: T) {
        self.general_purpose_registers[x.into()] = self.general_purpose_registers[y.into()];
    }

    pub fn x00xxx110(&mut self)

    pub fn load_rom(&mut self) {}
}

// pub struct GamePak {}
