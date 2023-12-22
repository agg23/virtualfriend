use bitvec::prelude::Lsb0;
use bitvec::{array::BitArray, order::LocalBits};

use crate::{bus::Bus, cpu_internals::ProgramStatusWord};

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
    fepsw: u32,

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
    chcw: u32,

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
            fepsw: 0,
            ecr: 0,
            psw: ProgramStatusWord::new(),
            pir: 0,
            tkcw: 0,
            chcw: 0,
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
    pub fn step(&mut self, bus: &mut Bus) -> (u32, BusActivity) {
        let instruction = self.fetch_instruction_word(bus);

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

                let immediate = self.fetch_instruction_word(bus);
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

                let immediate = self.fetch_instruction_word(bus);

                self.set_gen_purpose_reg(
                    reg2_index,
                    self.general_purpose_reg[reg1_index].wrapping_add((immediate as u32) << 16),
                );

                (1, BusActivity::Standard)
            }

            // Load and Input
            0b11_1000 => {
                // IN.B Input single byte
                self.load_inst_16(bus, instruction, 0xFFFF_FFFF, 0xFF, 0)
            }
            0b11_1001 => {
                // IN.H Input 16 bit word
                self.load_inst_16(bus, instruction, 0xFFFF_FFFE, 0xFFFF, 0)
            }
            0b11_1011 | 0b11_0011 => {
                // IN.W/LD.W Load word
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let disp = self.fetch_instruction_word(bus) as u32;
                let disp = sign_extend(disp, 16);

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);

                let value = bus.get_u32(address);

                self.set_gen_purpose_reg(reg2_index, value);

                (self.load_inst_cycle_count(), BusActivity::Load)
            }
            0b11_0000 => {
                // LD.B Load single byte (sign extend)
                self.load_inst_16(bus, instruction, 0xFFFF_FFFF, 0xFF, 8)
            }
            0b11_0001 => {
                // LD.B Load 16 bit word (sign extend)
                self.load_inst_16(bus, instruction, 0xFFFF_FFFE, 0xFFFF, 16)
            }

            // Store and Output
            0b11_1100 | 0b11_0100 => {
                // OUT.B/ST.B Store byte
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let disp = self.fetch_instruction_word(bus) as u32;
                let disp = sign_extend(disp, 16);

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);

                bus.set_u8(address, (self.general_purpose_reg[reg2_index] & 0xFF) as u8);

                (
                    self.store_inst_cycle_count(),
                    self.incrementing_store_bus_activity(),
                )
            }
            0b11_1101 | 0b11_0101 => {
                // OUT.H/ST.H Store 16 bit word
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let disp = self.fetch_instruction_word(bus) as u32;
                let disp = sign_extend(disp, 16);

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);

                bus.set_u16(
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

                let disp = self.fetch_instruction_word(bus) as u32;
                let disp = sign_extend(disp, 16);

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);

                bus.set_u32(address, self.general_purpose_reg[reg2_index]);

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
                let immediate = sign_extend(self.fetch_instruction_word(bus) as u32, 16);

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
                // DIV (signed)
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
                self.psw.update_alu_flags_u32(result, overflow, None);

                (38, BusActivity::Long)
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
                self.psw.update_alu_flags_u32(result, false, None);

                (36, BusActivity::Long)
            }
            0b00_1000 => {
                // MUL (signed)
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index] as i32;
                let reg2 = self.general_purpose_reg[reg2_index] as i32;

                let (result, overflow) = (reg1 as i64).overflowing_mul(reg2 as i64);
                let result = result as u64;

                self.set_gen_purpose_reg(30, (result >> 32) as u32);
                self.set_gen_purpose_reg(reg2_index, (result & 0xFFFF_FFFF) as u32);
                self.psw.update_alu_flags_u64(result, overflow, None);

                (13, BusActivity::Long)
            }
            0b00_1010 => {
                // MUL (unsigned)
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                let (result, overflow) = (reg1 as u64).overflowing_mul(reg2 as u64);

                self.set_gen_purpose_reg(30, (result >> 32) as u32);
                self.set_gen_purpose_reg(reg2_index, (result & 0xFFFF_FFFF) as u32);
                self.psw.update_alu_flags_u64(result, overflow, None);

                (13, BusActivity::Long)
            }
            0b00_0010 => {
                // SUB
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                self.sub_inst(reg2, reg1, Some(reg2_index))
            }

            // Bitwise
            0b00_1101 => {
                // AND
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                let result = reg2 & reg1;

                self.set_gen_purpose_reg(reg2_index, result);
                self.psw.update_alu_flags_u32(result, false, None);

                (1, BusActivity::Standard)
            }
            0b10_1101 => {
                // ANDI immediate, zero extended
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let immediate = self.fetch_instruction_word(bus) as u32;

                let result = reg1 & (immediate as u32);

                self.set_gen_purpose_reg(reg2_index, result);
                self.psw.update_alu_flags_u32(result, false, None);
                // Special case, sign is always false
                self.psw.sign = false;

                (1, BusActivity::Standard)
            }
            0b00_1111 => {
                // NOT
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];

                // Interestingly, Rust uses ! for bitwise NOT
                let result = !reg1;

                self.set_gen_purpose_reg(reg2_index, result);
                self.psw.update_alu_flags_u32(result, false, None);

                (1, BusActivity::Standard)
            }
            0b00_1100 => {
                // OR
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                let result = reg2 | reg1;

                self.set_gen_purpose_reg(reg2_index, result);
                self.psw.update_alu_flags_u32(result, false, None);

                (1, BusActivity::Standard)
            }
            0b10_1100 => {
                // ORI immediate, zero extend
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let immediate = self.fetch_instruction_word(bus) as u32;

                let result = reg1 | immediate;

                self.set_gen_purpose_reg(reg2_index, result);
                self.psw.update_alu_flags_u32(result, false, None);

                (1, BusActivity::Standard)
            }
            0b01_0111 => {
                // SAR Shift arthmetic right by immediate
                let (_, reg2_index) = extract_reg1_2_index(instruction);

                let reg2 = self.general_purpose_reg[reg2_index];
                let immediate = self.fetch_instruction_word(bus);

                self.sar_inst(reg2, immediate as u32, reg2_index)
            }
            0b00_0111 => {
                // SAR Shift arthmetic right by register
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                self.sar_inst(reg2, reg1, reg2_index)
            }
            0b01_0100 => {
                // SHL Shift logical left by immediate
                let (_, reg2_index) = extract_reg1_2_index(instruction);

                let reg2 = self.general_purpose_reg[reg2_index];
                let immediate = self.fetch_instruction_word(bus);

                self.shl_inst(reg2, immediate as u32, reg2_index)
            }
            0b00_0100 => {
                // SHL Shift logical left by register
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                self.shl_inst(reg2, reg1, reg2_index)
            }
            0b01_0101 => {
                // SHR Shift logical right by immediate
                let (_, reg2_index) = extract_reg1_2_index(instruction);

                let reg2 = self.general_purpose_reg[reg2_index];
                let immediate = self.fetch_instruction_word(bus);

                self.shr_inst(reg2, immediate as u32, reg2_index)
            }
            0b00_0101 => {
                // SHR Shift logical right by register
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                self.shr_inst(reg2, reg1, reg2_index)
            }
            0b00_1110 => {
                // XOR
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let reg2 = self.general_purpose_reg[reg2_index];

                let result = reg2 ^ reg1;

                self.set_gen_purpose_reg(reg2_index, result);
                self.psw.update_alu_flags_u32(result, false, None);

                (1, BusActivity::Standard)
            }
            0b10_1110 => {
                // XOR immediate, zero extend
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];
                let immediate = self.fetch_instruction_word(bus);

                let result = reg1 ^ (immediate as u32);

                self.set_gen_purpose_reg(reg2_index, result);
                self.psw.update_alu_flags_u32(result, false, None);

                (1, BusActivity::Standard)
            }

            // CPU Control
            // 0b10_0 + 4 bits
            0x20..=0x27 => {
                let condition = (instruction >> 9) & 0xF;
                let disp = instruction & 0x1FF;

                let condition = match condition {
                    0 => {
                        // BV Branch overflow
                        self.psw.overflow
                    }
                    1 => {
                        // BC, BL Branch carry/lower
                        self.psw.carry
                    }
                    2 => {
                        // BE, BZ Branch equal/zero
                        self.psw.zero
                    }
                    3 => {
                        // BNH Branch not higher
                        self.psw.carry || self.psw.zero
                    }
                    4 => {
                        // BN Branch negative
                        self.psw.sign
                    }
                    5 => {
                        // BR Branch always
                        true
                    }
                    6 => {
                        // BLT Branch less than
                        self.psw.overflow ^ self.psw.sign
                    }
                    7 => {
                        // BLE Branch less than or equal
                        (self.psw.overflow ^ self.psw.sign) || self.psw.zero
                    }
                    8 => {
                        // BNV Branch not overflow
                        !self.psw.overflow
                    }
                    9 => {
                        // BNC, BNL Branch not carry/lower
                        !self.psw.carry
                    }
                    10 => {
                        // BNE, BNZ Branch not equal/zero
                        !self.psw.zero
                    }
                    11 => {
                        // BH Branch higher
                        !(self.psw.carry || self.psw.zero)
                    }
                    12 => {
                        // BP Branch positive
                        !self.psw.sign
                    }
                    13 => {
                        // NOP
                        false
                    }
                    14 => {
                        // BGE Branch greater than or equal
                        !(self.psw.overflow ^ self.psw.sign)
                    }
                    15 => {
                        // BGT Branch greater than
                        !((self.psw.overflow ^ self.psw.sign) || self.psw.zero)
                    }
                    _ => unreachable!(),
                };

                self.conditional_jump(disp as u32, condition)
            }
            0b01_1010 => {
                // HALT
                todo!("Implement halt")
            }
            0b10_1011 => {
                // JAL Jump and link
                self.jump(bus, instruction, true)
            }
            0b00_0110 => {
                // JMP Jump register
                let (reg1_index, _) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];

                self.pc = reg1;

                (3, BusActivity::Standard)
            }
            0b10_1010 => {
                // JR Jump relative
                self.jump(bus, instruction, false)
            }
            0b01_1100 => {
                // LDSR Load to system register
                let (reg_id, reg2_index) = extract_reg1_2_index(instruction);

                let reg2 = self.general_purpose_reg[reg2_index];

                match reg_id {
                    0 => self.eipc = reg2,
                    1 => self.eipsw = reg2,
                    2 => self.fepc = reg2,
                    3 => self.fepsw = reg2,
                    // 4 => ecr
                    5 => self.psw.set(reg2),
                    // 6 => pir
                    // 7 => tkcw
                    24 => self.chcw = reg2,
                    25 => self.adtre = reg2,
                    29 => self.unknown_29 = reg2,
                    30 => self.unknown_30 = reg2,
                    31 => self.unknown_31 = reg2,
                    _ => {}
                }

                // TODO: Are flags supposed to be set here?

                (8, BusActivity::Standard)
            }
            0b01_1001 => {
                // RETI Return from trap or interrupt
                if self.psw.nmi_pending {
                    self.pc = self.fepc;
                    self.psw.set(self.fepsw);
                } else {
                    self.pc = self.eipc;
                    self.psw.set(self.eipsw);
                }

                // TODO: Are flags supposed to be set here?

                (10, BusActivity::Standard)
            }
            0b01_1101 => {
                // STSR Store contents of system register
                let (reg_id, reg2_index) = extract_reg1_2_index(instruction);

                let value = match reg_id {
                    0 => self.eipc,
                    1 => self.eipsw,
                    2 => self.fepc,
                    3 => self.fepsw,
                    // 4 => ecr
                    5 => self.psw.get(),
                    // 6 => pir
                    // 7 => tkcw
                    24 => self.chcw,
                    25 => self.adtre,
                    29 => self.unknown_29,
                    30 => self.unknown_30,
                    31 => self.unknown_31,
                    _ => 0,
                };

                self.set_gen_purpose_reg(reg2_index, value);

                (8, BusActivity::Standard)
            }
            0b01_1000 => {
                // TRAP
                todo!("Raise exception and set restore PC");
            }

            0b11_1110 => {
                // Floating point operations
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let second_instruction = self.fetch_instruction_word(bus);
                let sub_opcode = second_instruction >> 10;

                let reg1_int = self.general_purpose_reg[reg1_index];
                let reg1 = f32::from_bits(reg1_int);
                let reg2 = f32::from_bits(self.general_purpose_reg[reg2_index]);

                match sub_opcode {
                    0b00_0100 => {
                        // ADDF.S Add
                        let result = reg2 + reg1;

                        self.set_gen_purpose_reg(reg2_index, result.to_bits());
                        self.psw
                            .update_float_flags(result, true, false, false, true, true, true);

                        // TODO: This is a range of 9-28 cycles
                        (28, BusActivity::Standard)
                    }
                    0b00_0000 => {
                        // CMPF.S Compare
                        let result = reg2 - reg1;

                        self.psw
                            .update_float_flags(result, true, false, false, false, false, false);

                        // TODO: This is a range of 7-10 cycles
                        (10, BusActivity::Standard)
                    }
                    0b00_0011 => {
                        // CVT.SW Convert float to int
                        let result = reg1.round() as i32;

                        self.set_gen_purpose_reg(reg2_index, result as u32);
                        self.psw.update_float_flags(
                            result as f32,
                            true,
                            true,
                            false,
                            false,
                            false,
                            true,
                        );
                        self.psw.update_alu_flags_u32(result as u32, false, None);

                        // TODO: This is a range of 9-14 cycles
                        (14, BusActivity::Standard)
                    }
                    0b00_0010 => {
                        // CVT.WS Convert int to float
                        let result = (reg1_int as i32) as f32;

                        self.set_gen_purpose_reg(reg2_index, result.to_bits());
                        self.psw
                            .update_float_flags(result, false, false, false, false, false, true);

                        // TODO: This is a range of 5-16 cycles
                        (16, BusActivity::Standard)
                    }
                    0b00_0111 => {
                        // DIVF.S Divide
                        let result = reg2 / reg1;

                        self.set_gen_purpose_reg(reg2_index, result.to_bits());
                        self.psw
                            .update_float_flags(result, true, true, true, true, true, true);

                        (44, BusActivity::Standard)
                    }
                    0b00_0110 => {
                        // MULF.S Multiply
                        let result = reg2 * reg1;

                        self.set_gen_purpose_reg(reg2_index, result.to_bits());
                        self.psw
                            .update_float_flags(result, true, false, false, true, true, true);

                        // TODO: This is a range of 8-30 cycles
                        (30, BusActivity::Standard)
                    }
                    0b00_0101 => {
                        // SUBF.S Subtract
                        let result = reg2 - reg1;

                        self.set_gen_purpose_reg(reg2_index, result.to_bits());
                        self.psw
                            .update_float_flags(result, true, false, false, true, true, true);

                        // TODO: This is a range of 12-28 cycles
                        (28, BusActivity::Standard)
                    }
                    0b00_1011 => {
                        // TRNC.SW Truncate float to int
                        let result = reg1.trunc() as i32;

                        self.set_gen_purpose_reg(reg2_index, result as u32);
                        self.psw.update_float_flags(
                            result as f32,
                            true,
                            true,
                            false,
                            false,
                            false,
                            true,
                        );

                        // TODO: This is a range of 9-14 cycles
                        (14, BusActivity::Standard)
                    }
                    _ => panic!("Invalid float instruction {sub_opcode:x}"),
                }
            }

            0b01_1111 => {
                // Bit string operations
                let (sub_opcode, _) = extract_reg1_2_index(instruction);

                match sub_opcode {
                    0b0_1001 => {
                        // ANDBSU AND bit string
                        self.bit_string_process_upwards(bus, |source, dest| *dest = source & *dest);
                    }
                    _ => panic!("Invalid bit string instruction {sub_opcode:x}"),
                }

                // TODO: This should be updated from the table in the V810 manual
                (49, BusActivity::Standard)
            }

            _ => panic!("Invalid opcode {opcode:x}"),
        }
    }

    fn fetch_instruction_word(&mut self, bus: &mut Bus) -> u16 {
        let instruction = bus.get_u16(self.pc);

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
        bus: &mut Bus,
        instruction: u16,
        address_mask: u32,
        value_mask: u16,
        sign_extend_count: u8,
    ) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let disp = self.fetch_instruction_word(bus) as u32;
        let disp = sign_extend(disp, 16);

        let address = self.general_purpose_reg[reg1_index].wrapping_add(disp);
        let address = address & address_mask;

        let mut value = bus.get_u16(address);

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

        self.psw.update_alu_flags_u32(result, overflow, Some(carry));

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

        self.psw.update_alu_flags_u32(result, overflow, Some(carry));

        if let Some(store_reg_index) = store_reg_index {
            self.set_gen_purpose_reg(store_reg_index, result);
        }

        (1, BusActivity::Standard)
    }

    fn sar_inst(&mut self, value: u32, shift: u32, store_reg_index: usize) -> (u32, BusActivity) {
        // Limit to shift by 32
        let shift = shift & 0x1F;

        // Per https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
        // Arithmetic right shift on signed integer types, logical right shift on unsigned integer types
        // So we use a signed type
        let carry_result = if shift > 0 {
            (value as i32) >> (shift - 1)
        } else {
            value as i32
        };

        // One last shift to finish it
        let result = (carry_result >> 1) as u32;

        let carry = value != 0 && carry_result & 1 != 0;

        self.set_gen_purpose_reg(store_reg_index, result as u32);
        self.psw
            .update_alu_flags_u32(result as u32, false, Some(carry));

        (1, BusActivity::Standard)
    }

    fn shr_inst(&mut self, value: u32, shift: u32, store_reg_index: usize) -> (u32, BusActivity) {
        // Limit to shift by 32
        let shift = shift & 0x1F;

        // Per https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
        // Arithmetic right shift on signed integer types, logical right shift on unsigned integer types
        // So we use a signed type
        let carry_result = if shift > 0 {
            value >> (shift - 1)
        } else {
            value
        };

        // One last shift to finish it
        let result = carry_result >> 1;

        let carry = value != 0 && carry_result & 1 != 0;

        self.set_gen_purpose_reg(store_reg_index, result);
        self.psw.update_alu_flags_u32(result, false, Some(carry));

        (1, BusActivity::Standard)
    }

    fn shl_inst(&mut self, value: u32, shift: u32, store_reg_index: usize) -> (u32, BusActivity) {
        // Limit to shift by 32
        let shift = shift & 0x1F;

        let carry_result = if shift > 0 {
            value << (shift - 1)
        } else {
            value
        };

        // One last shift to finish it
        let result = carry_result << 1;

        let carry = value != 0 && carry_result & 1 != 0;

        self.set_gen_purpose_reg(store_reg_index, result);
        self.psw.update_alu_flags_u32(result, false, Some(carry));

        (1, BusActivity::Standard)
    }

    fn conditional_jump(&mut self, disp: u32, condition: bool) -> (u32, BusActivity) {
        if condition {
            // Jumping
            self.pc = (self.pc + disp) & 0xFFFF_FFFE;

            (3, BusActivity::Standard)
        } else {
            // Don't jump
            (1, BusActivity::Standard)
        }
    }

    fn jump(&mut self, bus: &mut Bus, instruction: u16, save_pc: bool) -> (u32, BusActivity) {
        let upper_disp = (instruction & 0x3FF) as u32;
        let disp = self.fetch_instruction_word(bus) as u32;

        let disp = disp | (upper_disp << 16);

        if save_pc {
            // PC has already been incremented by 2 by `fetch_instruction_word`
            self.set_gen_purpose_reg(31, self.pc + 2);
        }
        self.pc = self.pc + disp;

        (3, BusActivity::Standard)
    }

    fn bit_string_process_upwards(&mut self, bus: &mut Bus, func: impl Fn(bool, &mut bool)) {
        let mut dest_offset = self.general_purpose_reg[26] & 0x3F;
        self.set_gen_purpose_reg(26, dest_offset);

        let mut source_offset = self.general_purpose_reg[27] & 0x3F;
        self.set_gen_purpose_reg(27, source_offset);

        let mut length = self.general_purpose_reg[28];

        let mut dest_addr = self.general_purpose_reg[29] & 0xFFFF_FFFC;
        self.set_gen_purpose_reg(29, dest_addr);

        let mut source_addr = self.general_purpose_reg[30] & 0xFFFF_FFFC;
        self.set_gen_purpose_reg(30, source_addr);

        while length > 0 {
            // TODO: This fetches way more often. Unsure how costly this will be
            let source_word = BitArray::<_, Lsb0>::new(bus.get_u32(source_addr));
            let mut dest_word = BitArray::<_, Lsb0>::new(bus.get_u32(dest_addr));

            let source_bit = source_word.get(source_offset as usize).unwrap();
            let mut dest_bit = dest_word.get_mut(dest_offset as usize).unwrap();

            let func = |source: bool, dest: &mut bool| *dest = source & *dest;

            func(*source_bit, &mut dest_bit);

            // Make sure we can access borrowed data
            drop(dest_bit);
            bus.set_u32(dest_addr, dest_word.data);

            if source_offset >= 31 {
                source_offset = 0;
                // Increase by a word
                source_addr += 4;
            } else {
                source_offset += 1;
            }

            if dest_offset >= 31 {
                dest_offset = 0;
                // Increase by a word
                dest_addr += 4;
            } else {
                dest_offset += 1;
            }

            length -= 1;
        }

        // TODO: Do these need to be updated constantly and are interrupts allowed to interrupt this?
        self.set_gen_purpose_reg(26, dest_offset);
        self.set_gen_purpose_reg(27, source_offset);
        self.set_gen_purpose_reg(29, dest_addr);
        self.set_gen_purpose_reg(30, source_addr);
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
