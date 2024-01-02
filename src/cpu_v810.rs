use std::io::Write;
use std::{fs::File, io::BufWriter};

use bitvec::array::BitArray;
use bitvec::prelude::Lsb0;

use crate::{
    bus::Bus, cpu_internals::ProgramStatusWord, interrupt::InterruptRequest, util::sign_extend,
};

/// Tracks the most recent activity of the bus for the purposes of timing
pub enum BusActivity {
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

    /// Tracks whether the CPU has been halted with HALT
    is_halted: bool,

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

    /// ID 7: Task Control Word
    ///
    /// Specifies the behavior of floating-point instructions.
    tkcw: u32,

    /// ID 24: Cache Control Word
    ///
    /// Configures the instruction cache.
    // chcw: u32,
    cache_enabled: bool,

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

    last_bus_activity: BusActivity,
}

impl CpuV810 {
    pub fn new() -> Self {
        let mut psw = ProgramStatusWord::new();

        psw.set(0x8000);

        CpuV810 {
            pc: 0xFFFF_FFF0,
            is_halted: false,
            general_purpose_reg: [0; 32],
            eipc: 0,
            eipsw: 0,
            fepc: 0,
            fepsw: 0,
            ecr: 0xFFF0,
            psw,
            tkcw: 0,
            cache_enabled: false,
            adtre: 0,
            unknown_29: 0,
            unknown_30: 0,
            unknown_31: 0,

            last_bus_activity: BusActivity::Standard,
        }
    }

    /// TODO: This is debug init to match with Mednafen
    pub fn debug_init(&mut self) {
        self.tkcw = 0xE0;
    }

    pub fn log_instruction(
        &self,
        log_file: Option<&mut BufWriter<File>>,
        cycle_count: usize,
        extra_log_info: Option<String>,
    ) {
        // let mut tuples = vec![
        //     ("PC".to_string(), self.pc),
        //     ("R1".to_string(), self.general_purpose_reg[1]),
        //     ("FP".to_string(), self.general_purpose_reg[2]),
        //     ("SP".to_string(), self.general_purpose_reg[3]),
        //     ("GP".to_string(), self.general_purpose_reg[4]),
        //     ("TP".to_string(), self.general_purpose_reg[5]),
        // ];

        // for i in 6..31 {
        //     tuples.push((format!("R{i}"), self.general_purpose_reg[i]));
        // }

        // let mut after_tuples = vec![
        //     ("LP".to_string(), self.general_purpose_reg[31]),
        //     ("EIPC".to_string(), self.eipc),
        //     ("EIPSW".to_string(), self.eipsw),
        //     ("FEPC".to_string(), self.fepc),
        //     ("FEPSW".to_string(), self.fepsw),
        //     ("ECR".to_string(), self.ecr),
        //     ("PSW".to_string(), self.psw.get()),
        //     ("PIR".to_string(), self.pir),
        //     ("TKCW".to_string(), self.tkcw),
        //     ("CHCW".to_string(), self.chcw),
        //     ("ADTRE".to_string(), self.adtre),
        // ];

        // tuples.append(&mut after_tuples);

        // let mut string = String::new();
        // let mut first = true;

        // for (name, value) in tuples {
        //     if !first {
        //         string += " ";
        //     }

        //     string += &format!("{name}={value:08X}");

        //     first = false;
        // }

        // if let Some(extra_log_info) = extra_log_info {
        //     string += &format!(" {extra_log_info}");
        // }

        // TODO: Mednafen seems to wrap cycle count at the arbitrary? value 0x061200
        // let cycle_count = cycle_count % 0x061200;
        // string += &format!(" TStamp={cycle_count:06X}");

        if let Some(log_file) = log_file {
            writeln!(log_file, "PC={:08X}", self.pc).unwrap();
            // writeln!(log_file, "PC={:08X} R1={:08X} FP={:08X} SP={:08X} GP={:08X} TP={:08X} R6={:08X} R7={:08X} R8={:08X} R9={:08X} R10={:08X} R11={:08X} R12={:08X} R13={:08X} R14={:08X} R15={:08X} R16={:08X} R17={:08X} R18={:08X} R19={:08X} R20={:08X} R21={:08X} R22={:08X} R23={:08X} R24={:08X} R25={:08X} R26={:08X} R27={:08X} R28={:08X} R29={:08X} R30={:08X} LP={:08X} EIPC={:08X} EIPSW={:08X} FEPC={:08X} FEPSW={:08X} ECR={:08X} PSW={:08X} PIR={:08X} TKCW={:08X} CHCW={:08X} ADTRE={:08X}",
            // self.pc, self.general_purpose_reg[1], self.general_purpose_reg[2], self.general_purpose_reg[3], self.general_purpose_reg[4], self.general_purpose_reg[5], self.general_purpose_reg[6], self.general_purpose_reg[7], self.general_purpose_reg[8], self.general_purpose_reg[9], self.general_purpose_reg[10], self.general_purpose_reg[11], self.general_purpose_reg[12], self.general_purpose_reg[13], self.general_purpose_reg[14], self.general_purpose_reg[15], self.general_purpose_reg[16], self.general_purpose_reg[17], self.general_purpose_reg[18], self.general_purpose_reg[19], self.general_purpose_reg[20], self.general_purpose_reg[21], self.general_purpose_reg[22], self.general_purpose_reg[23], self.general_purpose_reg[24], self.general_purpose_reg[25], self.general_purpose_reg[26], self.general_purpose_reg[27], self.general_purpose_reg[28], self.general_purpose_reg[29], self.general_purpose_reg[30], self.general_purpose_reg[31], self.eipc, self.eipsw, self.fepc, self.fepsw, self.ecr, self.psw.get(), self.pir, self.tkcw, self.chcw, self.adtre).unwrap();
        }
    }

    /// Step one CPU instruction
    ///
    /// Returns the number of cycles consumed
    pub fn step(&mut self, bus: &mut Bus) -> usize {
        if self.is_halted {
            // Do nothing. 1 cycle consumed
            return 1;
        }

        let instruction = self.fetch_instruction_word(bus);

        let (cycles, bus_activity) = self.perform_instruction(bus, instruction);

        self.last_bus_activity = bus_activity;

        cycles as usize
    }

    /// Performs the necessary operations to jump to an interrupt, if valid
    pub fn request_interrupt(&mut self, request: InterruptRequest) {
        if self.psw.interrupt_disable {
            // Ignore
            return;
        }

        if self.psw.nmi_pending {
            todo!("Fatal exception")
        }

        if self.psw.exception_pending {
            todo!("Duplexed exception")
        }

        let code = request.code();

        // Second nibble is the same as interrupt level
        let interrupt_level = ((code >> 4) & 0xF) as u8;

        if interrupt_level < self.psw.interrupt_level {
            // Level not high enough to perform interrupt. Skip
            return;
        }

        self.perform_exception(code);

        if self.psw.interrupt_level < 15 {
            // Mask interrupts at this level or lower
            // Update interrupt mask level _after_ exception PSW copy
            self.psw.interrupt_level = interrupt_level + 1;
        }
    }

    fn perform_exception(&mut self, code: usize) {
        // Set interrupt code into cause code segment
        self.ecr = code as u32;
        // Backup PSW
        self.eipsw = self.psw.get();
        // Backup PC
        self.eipc = self.pc;
        self.psw.exception_pending = true;
        self.psw.interrupt_disable = true;
        self.psw.address_trap_enable = false;

        self.pc = 0xFFFF_0000 | code as u32;

        // Clear halt
        self.is_halted = false;
    }

    fn perform_instruction(&mut self, bus: &mut Bus, instruction: u16) -> (u32, BusActivity) {
        let opcode = instruction >> 10;

        if self.pc == 0xFFFCB5C8 + 2 {
            println!("{:08X} {:02X}", self.pc, opcode);
        }

        match opcode {
            // Register transfer
            0b01_0000 => {
                // MOV Immediate
                let (immediate, reg2_index) = extract_reg1_2_index(instruction);

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

                let reg1 = self.general_purpose_reg[reg1_index];

                let immediate = self.fetch_instruction_word(bus);
                let immediate = (immediate as i16) as u32;

                let result = reg1.wrapping_add(immediate);

                self.set_gen_purpose_reg(reg2_index, result);

                (1, BusActivity::Standard)
            }
            0b10_1111 => {
                // MOVHI Add upper immediate
                // Don't modify flags
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let reg1 = self.general_purpose_reg[reg1_index];

                let immediate = self.fetch_instruction_word(bus);

                let result = reg1.wrapping_add((immediate as u32) << 16);

                self.set_gen_purpose_reg(reg2_index, result);

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
                let disp = (disp as i16) as u32;

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp) & 0xFFFF_FFFC;

                let value = bus.get_u32(address);

                self.set_gen_purpose_reg(reg2_index, value);

                (self.load_inst_cycle_count(), BusActivity::Load)
            }
            0b11_0000 => {
                // LD.B Load single byte (sign extend)
                self.load_inst_16(bus, instruction, 0xFFFF_FFFF, 0xFF, 8)
            }
            0b11_0001 => {
                // LD.H Load 16 bit word (sign extend)
                self.load_inst_16(bus, instruction, 0xFFFF_FFFE, 0xFFFF, 16)
            }

            // Store and Output
            0b11_1100 | 0b11_0100 => {
                // OUT.B/ST.B Store byte
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let disp = self.fetch_instruction_word(bus) as u32;
                let disp = (disp as i16) as u32;

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
                let disp = (disp as i16) as u32;

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp) & 0xFFFF_FFFE;

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
                let disp = (disp as i16) as u32;

                let address = self.general_purpose_reg[reg1_index].wrapping_add(disp) & 0xFFFF_FFFC;

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

                self.add_inst(reg2, immediate, reg2_index)
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
                let immediate = self.fetch_instruction_word(bus);
                let immediate = (immediate as i16) as u32;

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

                let result = reg1 & immediate;

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
                let (immediate, reg2_index) = extract_reg1_2_index(instruction);

                let reg2 = self.general_purpose_reg[reg2_index];

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
                let (immediate, reg2_index) = extract_reg1_2_index(instruction);

                let reg2 = self.general_purpose_reg[reg2_index];

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
                let (immediate, reg2_index) = extract_reg1_2_index(instruction);

                let reg2 = self.general_purpose_reg[reg2_index];

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
            // Opcode is 6 bits, whereas this needs 7, so we just grab the relevant opcode
            // ranges, then extract the condition from the instruction inside
            0b10_0000..=0b10_0111 => {
                // BCOND
                let condition = (instruction >> 9) & 0xF;
                let disp = instruction & 0x1FF;
                let disp = sign_extend(disp as u32, 9);

                let condition = self.indexed_flag(condition);

                self.conditional_jump(disp, condition)
            }
            0b01_1010 => {
                // HALT
                self.is_halted = true;

                (1, BusActivity::Standard)
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
                    0 => self.eipc = reg2 & 0xFFFF_FFFE,
                    1 => {
                        // Don't write unused or reserved bits
                        self.eipsw = reg2 & 0x000FF3FF
                    }
                    2 => self.fepc = reg2 & 0xFFFF_FFFE,
                    3 => self.fepsw = reg2 & 0x000FF3FF,
                    // 4 => ECR not setable
                    5 => self.psw.set(reg2),
                    // 6 => pir
                    // 7 => tkcw
                    24 => {
                        println!("WARNING: Writing to cache control register");
                        // TODO: Finish implementation
                        // Cache enabled is stored here because it can be read
                        self.cache_enabled = reg2 & 0x2 != 0;
                    }
                    25 => self.adtre = reg2,
                    29 => self.unknown_29 = reg2,
                    30 => self.unknown_30 = reg2,
                    31 => self.unknown_31 = reg2,
                    _ => {}
                }

                // TODO: Are flags supposed to be set here?

                // TODO: Mednafen has this set to 1 cycle
                // The V810 manual doesn't list a cycle count
                // (8, BusActivity::Standard)
                (1, BusActivity::Standard)
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
                    4 => self.ecr,
                    5 => self.psw.get(),
                    6 => {
                        // Processor ID register
                        // Static value
                        0x5346
                    }
                    // 7 => tkcw
                    24 => {
                        // CHCW
                        if self.cache_enabled {
                            2
                        } else {
                            0
                        }
                    }
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
                // Floating point operations and Nintendo
                let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

                let second_instruction = self.fetch_instruction_word(bus);
                let sub_opcode = second_instruction >> 10;

                let reg1_int = self.general_purpose_reg[reg1_index];
                let reg2_int = self.general_purpose_reg[reg2_index];
                let reg1_float = f32::from_bits(reg1_int);
                let reg2_float = f32::from_bits(reg2_int);

                match sub_opcode {
                    // Float
                    0b00_0100 => {
                        // ADDF.S Add
                        let result = reg2_float + reg1_float;

                        self.set_gen_purpose_reg(reg2_index, result.to_bits());
                        self.psw
                            .update_float_flags(result, true, false, false, true, true, true);

                        // TODO: This is a range of 9-28 cycles
                        (28, BusActivity::Standard)
                    }
                    0b00_0000 => {
                        // CMPF.S Compare
                        let result = reg2_float - reg1_float;

                        self.psw
                            .update_float_flags(result, true, false, false, false, false, false);

                        // TODO: This is a range of 7-10 cycles
                        (10, BusActivity::Standard)
                    }
                    0b00_0011 => {
                        // CVT.SW Convert float to int
                        let result = reg1_float.round() as i32;

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
                        let result = reg2_float / reg1_float;

                        self.set_gen_purpose_reg(reg2_index, result.to_bits());
                        self.psw
                            .update_float_flags(result, true, true, true, true, true, true);

                        (44, BusActivity::Standard)
                    }
                    0b00_0110 => {
                        // MULF.S Multiply
                        let result = reg2_float * reg1_float;

                        self.set_gen_purpose_reg(reg2_index, result.to_bits());
                        self.psw
                            .update_float_flags(result, true, false, false, true, true, true);

                        // TODO: This is a range of 8-30 cycles
                        (30, BusActivity::Standard)
                    }
                    0b00_0101 => {
                        // SUBF.S Subtract
                        let result = reg2_float - reg1_float;

                        self.set_gen_purpose_reg(reg2_index, result.to_bits());
                        self.psw
                            .update_float_flags(result, true, false, false, true, true, true);

                        // TODO: This is a range of 12-28 cycles
                        (28, BusActivity::Standard)
                    }
                    0b00_1011 => {
                        // TRNC.SW Truncate float to int
                        let result = reg1_float.trunc() as i32;

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

                    // Nintendo
                    0b00_1100 => {
                        // MPYHW Multiply halfword
                        let reg1_int = ((reg1_int << 15) as i32) >> 15;
                        let result = (reg2_int as i32) * reg1_int;

                        self.set_gen_purpose_reg(reg2_index, result as u32);

                        (9, BusActivity::Standard)
                    }
                    0b00_1010 => {
                        // REV Reverse bits in word
                        let result = reg1_int.reverse_bits();

                        self.set_gen_purpose_reg(reg2_index, result);

                        (22, BusActivity::Standard)
                    }
                    0b00_1000 => {
                        // XB Exchange byte
                        // Swaps the bottom two bytes
                        let upper = reg2_int & 0xFFFF_0000;
                        let lower_high = (reg2_int << 8) & 0xFF00;
                        let lower_low = (reg2_int >> 8) & 0xFF;

                        let result = upper | lower_high | lower_low;

                        self.set_gen_purpose_reg(reg2_index, result);

                        (6, BusActivity::Standard)
                    }
                    0b00_1001 => {
                        // XH Exchange halfword
                        // Swap upper and lower halfwords
                        let result = (reg2_int >> 16) | (reg2_int << 16);

                        self.set_gen_purpose_reg(reg2_index, result);

                        (1, BusActivity::Standard)
                    }

                    _ => panic!("Invalid float or Nintendo instruction {sub_opcode:x}"),
                }
            }

            0b01_1111 => {
                // Bit string operations
                let (sub_opcode, _) = extract_reg1_2_index(instruction);

                if sub_opcode < 5 {
                    // Search operation
                    let (upwards_direction, match_1) = match sub_opcode {
                        0b0_0001 => {
                            // Search for 0, downward
                            (false, false)
                        }
                        0b0_0000 => {
                            // Search for 0, upward
                            (true, false)
                        }
                        0b0_0011 => {
                            // Search for 1, downward
                            (false, true)
                        }
                        0b0_0010 => {
                            // Search for 1, upward
                            (true, true)
                        }
                        _ => unreachable!(),
                    };

                    self.bit_string_search(bus, upwards_direction, match_1)
                } else {
                    self.bit_string_process_upwards(bus, sub_opcode);
                }

                // TODO: This should be updated from the table in the V810 manual
                // No one seems to emulate this correctly
                (49, BusActivity::Standard)
            }

            // Miscellaneous
            0b11_1010 => {
                // CAXI Compare and exchange interlocked
                unimplemented!("CAXI doesn't really do anything on VB")
            }
            0b01_0010 => {
                // SETF Set flag condition
                let (_, reg2_index) = extract_reg1_2_index(instruction);

                let condition = instruction & 0xF;

                let value = if self.indexed_flag(condition) { 1 } else { 0 };

                self.set_gen_purpose_reg(reg2_index, value);

                (1, BusActivity::Standard)
            }

            // Nintendo
            0b01_0110 => {
                // CLI Clear interrupt disable flag
                self.psw.interrupt_disable = false;

                // TODO: Mednafen has this set to 1 cycle
                (12, BusActivity::Standard)
            }
            0b01_1110 => {
                // SEI Set interrupt disable flag
                self.psw.interrupt_disable = true;

                // TODO: Mednafen has this set to 1 cycle
                (12, BusActivity::Standard)
            }
            _ => panic!("Invalid opcode {opcode:x}"),
        }
    }

    fn fetch_instruction_word(&mut self, bus: &mut Bus) -> u16 {
        let instruction = bus.get_u16(self.pc);

        // Increment PC by 2 bytes
        self.pc = self.pc.wrapping_add(2);

        instruction
    }

    fn set_gen_purpose_reg(&mut self, index: usize, value: u32) {
        if index == 0 {
            // Do not write to r0
            println!("Attempted write to register 0");
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

        let disp = self.fetch_instruction_word(bus);
        let disp = (disp as i16) as u32;

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
        let (result, carry) = if shift > 0 {
            let carry_result = (value as i32) >> (shift - 1);

            // One last shift to finish it
            let result = (carry_result >> 1) as u32;

            // Carry is the last bit that's shifted out
            let carry = carry_result & 1 != 0;

            (result, carry)
        } else {
            (value, false)
        };

        self.set_gen_purpose_reg(store_reg_index, result);
        self.psw.update_alu_flags_u32(result, false, Some(carry));

        (1, BusActivity::Standard)
    }

    fn shr_inst(&mut self, value: u32, shift: u32, store_reg_index: usize) -> (u32, BusActivity) {
        // Limit to shift by 32
        let shift = shift & 0x1F;

        // Per https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
        // Arithmetic right shift on signed integer types, logical right shift on unsigned integer types
        // So we use a signed type
        let (result, carry) = if shift > 0 {
            let carry_result = value >> (shift - 1);

            // One last shift to finish it
            let result = carry_result >> 1;

            // Carry is the last bit that's shifted out
            let carry = carry_result & 1 != 0;

            (result, carry)
        } else {
            (value, false)
        };

        self.set_gen_purpose_reg(store_reg_index, result);
        self.psw.update_alu_flags_u32(result, false, Some(carry));

        (1, BusActivity::Standard)
    }

    fn shl_inst(&mut self, value: u32, shift: u32, store_reg_index: usize) -> (u32, BusActivity) {
        // Limit to shift by 32
        let shift = shift & 0x1F;

        let (result, carry) = if shift > 0 {
            let carry_result = value << (shift - 1);

            // One last shift to finish it
            let result = carry_result << 1;

            // Carry is the last bit that's shifted out
            let carry = value != 0 && carry_result & 0x8000_0000 != 0;

            (result, carry)
        } else {
            (value, false)
        };

        self.set_gen_purpose_reg(store_reg_index, result);
        self.psw.update_alu_flags_u32(result, false, Some(carry));

        (1, BusActivity::Standard)
    }

    fn indexed_flag(&self, flag: u16) -> bool {
        match flag {
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
        }
    }

    fn conditional_jump(&mut self, disp: u32, condition: bool) -> (u32, BusActivity) {
        if condition {
            // Jumping
            // PC was already incremented by 2, so we have to remove that
            self.pc = (self.pc - 2).wrapping_add(disp & 0xFFFF_FFFE);

            (3, BusActivity::Standard)
        } else {
            // Don't jump
            (1, BusActivity::Standard)
        }
    }

    fn jump(&mut self, bus: &mut Bus, instruction: u16, save_pc: bool) -> (u32, BusActivity) {
        let upper_disp = (instruction & 0x3FF) as u32;
        let disp = self.fetch_instruction_word(bus) as u32;

        let disp = sign_extend((upper_disp << 16) | disp, 26);

        if save_pc {
            // PC has already been incremented by 4 by 2x `fetch_instruction_word`
            self.set_gen_purpose_reg(31, self.pc);
        }

        // PC has already been incremented by 4 by 2x `fetch_instruction_word`
        self.pc = (self.pc - 4).wrapping_add(disp);

        (3, BusActivity::Standard)
    }

    fn bit_string_search(&mut self, bus: &mut Bus, upwards_direction: bool, match_1: bool) {
        // Clear mask bits in registers
        let source_offset = self.general_purpose_reg[27] & 0x3F;
        self.set_gen_purpose_reg(27, source_offset);

        let mut source_addr = self.general_purpose_reg[30] & 0xFFFF_FFFC;
        self.set_gen_purpose_reg(30, source_addr);

        let mut length = self.general_purpose_reg[28];

        let mut source_word = bus.get_u32(source_addr);

        // The position of the current bit we're examining
        let mut word_offset = if upwards_direction { 0 } else { 31 };
        let mut examined_bit_count = 0;

        let mut found = false;

        while length > 0 {
            let source_bit = if upwards_direction {
                source_word & 1 != 0
            } else {
                source_word & 0x8000_0000 != 0
            };

            if source_bit == match_1 {
                // We've found our bit
                found = true;
            }

            examined_bit_count += 1;

            // Haven't found it, continue processing
            if upwards_direction {
                if word_offset == 31 {
                    // Finished word
                    // Reset value for reg setting below
                    word_offset = 0;

                    source_addr += 4;
                    break;
                } else {
                    source_word = source_word << 1;

                    word_offset += 1;
                }
            } else {
                if word_offset == 0 {
                    // Finished word
                    word_offset = 31;

                    source_addr -= 4;
                    break;
                } else {
                    source_word = source_word >> 1;

                    word_offset -= 1;
                }
            }

            if found {
                break;
            }

            length -= 1;
        }

        if !found && length != 0 {
            // There's still more string to process
            self.pc -= 2;
        } else if found {
            // examined_bit_count counted the matched bit, remove it
            examined_bit_count -= 1;
        }

        self.set_gen_purpose_reg(27, source_offset);
        self.set_gen_purpose_reg(28, word_offset);
        self.set_gen_purpose_reg(29, self.general_purpose_reg[29] + examined_bit_count);
        self.set_gen_purpose_reg(30, source_addr);
    }

    fn bit_string_process_upwards(&mut self, bus: &mut Bus, sub_opcode: usize) {
        println!("Running bit string. May have errors?");
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

            let source_bit: &bool = &source_word.get(source_offset as usize).unwrap();
            let mut dest_bit = dest_word.get_mut(dest_offset as usize).unwrap();

            match sub_opcode {
                0b0_1001 => {
                    // ANDBSU AND bit string
                    *dest_bit = *dest_bit & *source_bit;
                }
                0b0_1101 => {
                    // ANDNBSU AND NOT bit string
                    *dest_bit = *dest_bit & !*source_bit;
                }
                0b0_1011 => {
                    // MOVBSU Move bit string
                    *dest_bit = *source_bit;
                }
                0b0_1111 => {
                    // NOTBSU NOT bit string
                    *dest_bit = !*source_bit;
                }
                0b0_1000 => {
                    // ORBSU OR bit string
                    *dest_bit = *dest_bit | *source_bit;
                }
                0b0_1100 => {
                    // ORNBSU OR NOT bit string
                    *dest_bit = *dest_bit | !*source_bit;
                }
                0b0_1010 => {
                    // XORBSU XOR bit string
                    *dest_bit = *dest_bit ^ *source_bit;
                }
                0b0_1110 => {
                    // XORNBSU XOR NOT bit string
                    *dest_bit = *dest_bit ^ !*source_bit;
                }
                _ => panic!("Invalid bit string instruction {sub_opcode:x}"),
            }

            // Make sure we can access borrowed data
            drop(dest_bit);
            bus.set_u32(dest_addr, dest_word.data);

            if source_offset >= 31 {
                source_offset = 0;
                // Increase by a word
                source_addr += 4;

                break;
            } else {
                source_offset += 1;
            }

            if dest_offset >= 31 {
                dest_offset = 0;
                // Increase by a word
                dest_addr += 4;

                break;
            } else {
                dest_offset += 1;
            }

            length -= 1;
        }

        if length != 0 {
            // We haven't finished this operation. Rewind PC
            self.pc -= 2;
        }

        // TODO: Do these need to be updated constantly and are interrupts allowed to interrupt this?
        self.set_gen_purpose_reg(26, dest_offset);
        self.set_gen_purpose_reg(27, source_offset);
        self.set_gen_purpose_reg(28, 0);
        self.set_gen_purpose_reg(29, dest_addr);
        self.set_gen_purpose_reg(30, source_addr);
    }

    fn load_inst_cycle_count(&self) -> u32 {
        // NOTE: For some reason changing these values from Mednafen
        // causes Wario Land warning text to not animate
        // TODO: Mednafen for some reason has this as 1, 3
        match self.last_bus_activity {
            BusActivity::Long => 1,
            BusActivity::Load => 2,
            // TODO: Does store warm up the memory pipeline?
            _ => 3,
        }

        // match self.last_bus_activity {
        //     BusActivity::Long => 1,
        //     BusActivity::Load => 4,
        //     // TODO: Does store warm up the memory pipeline?
        //     _ => 5,
        // }
    }

    fn store_inst_cycle_count(&self) -> u32 {
        // NOTE: For some reason changing these values from Mednafen
        // causes Wario Land warning text to not animate
        // TODO: Mednafen for some reason has this as 1, 2
        match self.last_bus_activity {
            BusActivity::StoreInitial | BusActivity::StoreAfter => 2,
            _ => 1,
        }

        // match self.last_bus_activity {
        //     BusActivity::StoreAfter => 4,
        //     // First and second stores are 1 cycle
        //     BusActivity::StoreInitial | _ => 1,
        // }
    }

    fn incrementing_store_bus_activity(&self) -> BusActivity {
        match self.last_bus_activity {
            BusActivity::StoreInitial => BusActivity::StoreAfter,
            BusActivity::StoreAfter => BusActivity::StoreAfter,
            _ => BusActivity::StoreInitial,
        }
    }
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
