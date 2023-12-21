use crate::{cpu_internals::ProgramStatusWord, memory::WRAM};

///
/// Tracks the most recent activity of the bus for the purposes of timing
///
enum BusActivity {
    Standard,
    Long,
    Load,
    /// First store bus event
    StoreInitial,
    /// Any store bus event after the first
    StoreAfter,
}

pub struct CpuV810 {
    pc: u32,

    ///
    /// 31 general purpose registers, and r0 == 0.
    ///
    /// r0 should not be mutated.
    ///
    /// r26	Bit string destination bit offset.
    ///
    /// r27	Bit string source bit offset.
    ///
    /// r28	Bit string length.
    ///
    /// r29	Multiple uses:
    ///
    /// •	Bit string destination word address.
    ///
    /// •	Bit string number of bits skipped in a search.
    ///
    /// r30	Multiple uses:
    ///
    /// •	Stores upper 32 bits of integer multiplication results.
    ///
    /// •	Stores remainder of integer division results.
    ///
    /// •	Represents the exchange value for the CAXI instruction.
    ///
    /// •	Bit string source word address.
    ///
    /// r31	Stores the return address of the JAL instruction.
    ///
    general_purpose_reg: [u32; 32],

    /// ID 0: Exception/Interrupt PC
    ///
    /// Stores the value to restore to PC when an exception finishes processing.
    eipc: u32,

    /// ID 1: Exception/Interrupt PSW
    ///
    /// Stores the value to restore to PSW when an exception finishes processing.
    eipsw: u32,

    /// ID 2: Fatal Error PC
    ///
    /// Stores the value to restore to PC when a duplexed exception finishes processing.
    fepc: u32,

    /// ID 3: Fatal Error PSW
    ///
    /// Stores the value to restore to PSW when a duplexed exception finishes processing.
    fepse: u32,

    /// ID 4: Exception Cause Register
    ///
    /// Stores values indicating the source of exceptions or interrupts.
    ecr: u32,

    /// ID 5: Program Status Word
    ///
    /// Contains status flags and the interrupt masking level.
    psw: ProgramStatusWord,

    /// ID 6: Processor ID Register
    ///
    /// Indicates to the program what kind of processor it's being run on.
    pir: u32,

    /// ID 7: Task Control Word
    ///
    /// Specifies the behavior of floating-point instructions.
    tkcw: u32,

    /// ID 24: Cache Control Word
    ///
    /// Configures the instruction cache.
    chch: u32,

    /// ID 25: Address Trap Register for Execution
    ///
    /// Configures the execution address for the hardware breakpoint.
    adtre: u32,

    /// ID 29: Unknown register
    unknown_29: u32,

    /// ID 30: Unknown register
    unknown_30: u32,

    /// ID 31: Unknown register
    unknown_31: u32,

    bus_activity: BusActivity,
}

impl CpuV810 {
    pub fn new() -> Self {
        CpuV810 {
            pc: 0xFFFF_FFF0,
            general_purpose_reg: [0; 32],
            eipc: 0,
            eipsw: 0,
            fepc: 0,
            fepse: 0,
            ecr: 0,
            psw: ProgramStatusWord::new(),
            pir: 0,
            tkcw: 0,
            chch: 0,
            adtre: 0,
            unknown_29: 0,
            unknown_30: 0,
            unknown_31: 0,

            bus_activity: BusActivity::Standard,
        }
    }

    ///
    /// Step one CPU instruction
    ///
    /// Returns the number of cycles consumed
    ///
    pub fn step(&mut self, wram: &mut WRAM) -> (u32, BusActivity) {
        let instruction = self.fetch_instruction_word(wram);

        let opcode = (instruction >> 11) & 0x1F;

        match opcode {
            // Register transfer
            0b01_0000 => {
                // MOV Immediate
                let reg2_index = extract_reg2_index(instruction);

                let immediate = extract_reg1_index(instruction);

                self.set_gen_purpose_reg(reg2_index, sign_extend(immediate as u32, 5));

                (1, BusActivity::Standard)
            }
            0b00_0000 => {
                // MOV Register
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];

                self.set_gen_purpose_reg(reg2_index, reg1);

                (1, BusActivity::Standard)
            }
            0b10_1000 => {
                // MOVEA Add
                // Don't modify flags
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let immediate = self.fetch_instruction_word(wram);
                let immediate = sign_extend(immediate as u32, 16);

                self.set_gen_purpose_reg(
                    reg2_index,
                    self.general_purpose_reg[reg1_index].wrapping_add(immediate),
                );

                (1, BusActivity::Standard)
            }
            0b10_1111 => {
                // MOVHI Add upper immediate
                // Don't modify flags
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let immediate = self.fetch_instruction_word(wram);

                self.set_gen_purpose_reg(
                    reg2_index,
                    self.general_purpose_reg[reg1_index].wrapping_add((immediate as u32) << 16),
                );

                (1, BusActivity::Standard)
            }

            // Load and Input
            0b11_1000 => {
                // IN.B Input single byte
                self.load_inst_16(wram, instruction, 0xFFFF_FFFF, 0xFF, 0)
            }
            0b11_1001 => {
                // IN.H Input 16 bit word
                self.load_inst_16(wram, instruction, 0xFFFF_FFFE, 0xFFFF, 0)
            }
            0b11_1011 | 0b11_0011 => {
                // IN.W/LD.W Load word
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let disp = self.fetch_instruction_word(wram) as u32;
                let disp = sign_extend(disp, 16);

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);

                let value = wram.get_u32(address);

                self.set_gen_purpose_reg(reg2_index, value);

                (self.load_inst_cycle_count(), BusActivity::Load)
            }
            0b11_0000 => {
                // LD.B Load single byte (sign extend)
                self.load_inst_16(wram, instruction, 0xFFFF_FFFF, 0xFF, 8)
            }
            0b11_0001 => {
                // LD.B Load 16 bit word (sign extend)
                self.load_inst_16(wram, instruction, 0xFFFF_FFFE, 0xFFFF, 16)
            }

            // Store and Output
            0b11_1100 | 0b11_0100 => {
                // OUT.B/ST.B Store byte
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let disp = self.fetch_instruction_word(wram) as u32;
                let disp = sign_extend(disp, 16);

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);

                wram.set_u8(address, (self.general_purpose_reg[reg2_index] & 0xFF) as u8);

                (
                    self.store_inst_cycle_count(),
                    self.incrementing_store_bus_activity(),
                )
            }
            0b11_1101 | 0b11_0101 => {
                // OUT.H/ST.H Store 16 bit word
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let disp = self.fetch_instruction_word(wram) as u32;
                let disp = sign_extend(disp, 16);

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);

                wram.set_u16(
                    address,
                    (self.general_purpose_reg[reg2_index] & 0xFFFF) as u16,
                );

                (
                    self.store_inst_cycle_count(),
                    self.incrementing_store_bus_activity(),
                )
            }
            0b11_1111 | 0b11_0111 => {
                // OUT.W/ST.W Store word
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let disp = self.fetch_instruction_word(wram) as u32;
                let disp = sign_extend(disp, 16);

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);

                wram.set_u32(address, self.general_purpose_reg[reg2_index]);

                (
                    self.store_inst_cycle_count(),
                    self.incrementing_store_bus_activity(),
                )
            }

            // Arithmetic
            0b01_0001 => {
                // ADD immediate
                let (immediate, reg2_index) = extract_reg1_2_index(instruction);
                let immediate = sign_extend(immediate as u32, 5);

                let reg2 = self.general_purpose_reg[reg2_index];

                self.add_inst(reg2, immediate, reg2_index);

                (1, BusActivity::Standard)
            }
            0b00_0001 => {
                // ADD reg
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                self.add_inst(reg1, reg2, reg2_index)
            }
            0b10_1001 => {
                // ADD 16 bit immediate
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);
                let immediate = sign_extend(self.fetch_instruction_word(wram) as u32, 16);

                let reg1 = self.general_purpose_reg[reg1_index];

                self.add_inst(reg1, immediate, reg2_index)
            }
            0b01_0011 => {
                // CMP immediate
                let (immediate, reg2_index) = extract_reg1_2_index(instruction);
                let immediate = sign_extend(immediate as u32, 5);

                let reg2 = self.general_purpose_reg[reg2_index];

                self.sub_inst(reg2, immediate, None)
            }
            0b00_0011 => {
                // CMP register
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                self.sub_inst(reg2, reg1, None)
            }
            0b00_1001 => {
                // DIV
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                if reg1 == 0 {
                    todo!("Handle divide by zero exception");
                }

                let (result, remainder, overflow) = if reg2 == 0x8000_0000 && reg1 == 0xFFFF_FFFF {
                    // Special case to set overflow
                    (0x8000_0000, 0, true)
                } else {
                    let reg1 = reg1 as i32;
                    let reg2 = reg2 as i32;

                    let remainder = (reg2 % reg1) as u32;
                    let result = (reg2 / reg1) as u32;

                    (result, remainder, false)
                };

                // Remainder
                self.general_purpose_reg[30] = remainder;
                self.set_gen_purpose_reg(reg2_index, result);
                self.psw.update_alu_flags(result, overflow, None);

                (38, BusActivity::Standard)
            }
            0b00_1011 => {
                // DIVU (unsigned)
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                if reg1 == 0 {
                    todo!("Handle divide by zero exception");
                }

                let remainder = reg2 % reg1;
                let result = reg2 / reg1;
                self.general_purpose_reg[30] = remainder;
                self.set_gen_purpose_reg(reg2_index, result);
                self.psw.update_alu_flags(result, false, None);

                (36, BusActivity::Standard)
            }
        }
    }

    fn fetch_instruction_word(&mut self, wram: &mut WRAM) -> u16 {
        let instruction = wram.get_u16(self.pc);

        // Increment PC by 2 bytes
        self.pc += 2;

        instruction
    }

    fn set_gen_purpose_reg(&mut self, index: usize, value: u32) {
        if index == 0 {
            // Do not write to r0
            return;
        }

        self.general_purpose_reg[index] = value;
    }

    fn load_inst_16(
        &mut self,
        wram: &mut WRAM,
        instruction: u16,
        address_mask: u32,
        value_mask: u16,
        sign_extend_count: u8,
    ) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let disp = self.fetch_instruction_word(wram) as u32;
        let disp = sign_extend(disp, 16);

        let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);
        let address = address & address_mask;

        let mut value = wram.get_u16(address);

        if address & 1 != 0 {
            // High byte in word
            value = (value >> 8) & 0xFF;
        }

        let mut value = (value & value_mask) as u32;

        if sign_extend_count != 0 {
            value = sign_extend(value, sign_extend_count);
        }

        self.set_gen_purpose_reg(reg2_index, value);

        (self.load_inst_cycle_count(), BusActivity::Load)
    }

    fn add_inst(&mut self, a: u32, b: u32, store_reg_index: usize) -> (u32, BusActivity) {
        let (result, carry) = a.overflowing_add(b);

        // Taken from rustual-boy
        let overflow = ((!(a ^ b) & (b ^ result)) & 0x80000000) != 0;

        self.psw.update_alu_flags(result, overflow, Some(carry));

        self.set_gen_purpose_reg(store_reg_index, result);

        (1, BusActivity::Standard)
    }

    fn sub_inst(
        &mut self,
        lhs: u32,
        rhs: u32,
        store_reg_index: Option<usize>,
    ) -> (u32, BusActivity) {
        let (result, carry) = lhs.overflowing_sub(rhs);

        // Taken from rustual-boy
        let overflow = (((lhs ^ rhs) & !(rhs ^ result)) & 0x80000000) != 0;

        self.psw.update_alu_flags(result, overflow, Some(carry));

        if let Some(store_reg_index) = store_reg_index {
            self.set_gen_purpose_reg(store_reg_index, result);
        }

        (1, BusActivity::Standard)
    }

    fn load_inst_cycle_count(&self) -> u32 {
        match self.bus_activity {
            BusActivity::Long => 1,
            BusActivity::Load => 4,
            // TODO: Does store warm up the memory pipeline?
            _ => 5,
        }
    }

    fn store_inst_cycle_count(&self) -> u32 {
        match self.bus_activity {
            BusActivity::StoreAfter => 4,
            // First and second stores are 1 cycle
            BusActivity::StoreInitial | _ => 1,
        }
    }

    fn incrementing_store_bus_activity(&self) -> BusActivity {
        match self.bus_activity {
            BusActivity::StoreInitial => BusActivity::StoreAfter,
            BusActivity::StoreAfter => BusActivity::StoreAfter,
            _ => BusActivity::StoreInitial,
        }
    }
}

fn sign_extend(value: u32, size: u8) -> u32 {
    (value << (32 - size)) >> (32 - size)
}

fn extract_reg1_2_index(instruction: u16) -> (usize, usize) {
    (
        extract_reg1_index(instruction),
        extract_reg2_index(instruction),
    )
}

fn extract_reg2_index(instruction: u16) -> usize {
    ((instruction >> 5) & 0x1F) as usize
}

fn extract_reg1_index(instruction: u16) -> usize {
    (instruction & 0x1F) as usize
}
