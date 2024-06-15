use std::io::{StdoutLock, Write};
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
    // TODO: Add proper exception for float operations
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

    processing_bitstring: bool,
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

            processing_bitstring: false,
        }
    }

    /// TODO: This is debug init to match with Mednafen
    pub fn debug_init(&mut self) {
        // self.tkcw = 0xE0;

        for i in 1..32 {
            self.general_purpose_reg[i] = 0xDEADBEEF;
        }
        self.eipc = 0xDEADBEEE;
        self.eipsw = 0xDB2EF;
        self.fepc = 0xDEADBEEE;
        self.fepsw = 0xDB2EF;
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

        if self.processing_bitstring {
            // Ignore step
            return;
        }

        let chcw = if self.cache_enabled { 2 } else { 0 };

        if let Some(log_file) = log_file {
            // writeln!(log_file, "PC={:08X}", self.pc).unwrap();
            writeln!(log_file, "PC={:08X} R1={:08X} FP={:08X} SP={:08X} GP={:08X} TP={:08X} R6={:08X} R7={:08X} R8={:08X} R9={:08X} R10={:08X} R11={:08X} R12={:08X} R13={:08X} R14={:08X} R15={:08X} R16={:08X} R17={:08X} R18={:08X} R19={:08X} R20={:08X} R21={:08X} R22={:08X} R23={:08X} R24={:08X} R25={:08X} R26={:08X} R27={:08X} R28={:08X} R29={:08X} R30={:08X} LP={:08X} EIPC={:08X} EIPSW={:08X} FEPC={:08X} FEPSW={:08X} ECR={:08X} PSW={:08X} PIR=00005347 TKCW={:08X} CHCW={:08X} ADTRE={:08X}",
            self.pc, self.general_purpose_reg[1], self.general_purpose_reg[2], self.general_purpose_reg[3], self.general_purpose_reg[4], self.general_purpose_reg[5], self.general_purpose_reg[6], self.general_purpose_reg[7], self.general_purpose_reg[8], self.general_purpose_reg[9], self.general_purpose_reg[10], self.general_purpose_reg[11], self.general_purpose_reg[12], self.general_purpose_reg[13], self.general_purpose_reg[14], self.general_purpose_reg[15], self.general_purpose_reg[16], self.general_purpose_reg[17], self.general_purpose_reg[18], self.general_purpose_reg[19], self.general_purpose_reg[20], self.general_purpose_reg[21], self.general_purpose_reg[22], self.general_purpose_reg[23], self.general_purpose_reg[24], self.general_purpose_reg[25], self.general_purpose_reg[26], self.general_purpose_reg[27], self.general_purpose_reg[28], self.general_purpose_reg[29], self.general_purpose_reg[30], self.general_purpose_reg[31], self.eipc, self.eipsw, self.fepc, self.fepsw, self.ecr, self.psw.get(), self.tkcw, chcw, self.adtre).unwrap();
        }
        // std_lock.write_fmt(format_args!("PC={:08X} R1={:08X} FP={:08X} SP={:08X} GP={:08X} TP={:08X} R6={:08X} R7={:08X} R8={:08X} R9={:08X} R10={:08X} R11={:08X} R12={:08X} R13={:08X} R14={:08X} R15={:08X} R16={:08X} R17={:08X} R18={:08X} R19={:08X} R20={:08X} R21={:08X} R22={:08X} R23={:08X} R24={:08X} R25={:08X} R26={:08X} R27={:08X} R28={:08X} R29={:08X} R30={:08X} LP={:08X} EIPC={:08X} EIPSW={:08X} FEPC={:08X} FEPSW={:08X} ECR={:08X} PSW={:08X} PIR=00005347 TKCW={:08X} CHCW={:08X} ADTRE={:08X}\n",
        // self.pc, self.general_purpose_reg[1], self.general_purpose_reg[2], self.general_purpose_reg[3], self.general_purpose_reg[4], self.general_purpose_reg[5], self.general_purpose_reg[6], self.general_purpose_reg[7], self.general_purpose_reg[8], self.general_purpose_reg[9], self.general_purpose_reg[10], self.general_purpose_reg[11], self.general_purpose_reg[12], self.general_purpose_reg[13], self.general_purpose_reg[14], self.general_purpose_reg[15], self.general_purpose_reg[16], self.general_purpose_reg[17], self.general_purpose_reg[18], self.general_purpose_reg[19], self.general_purpose_reg[20], self.general_purpose_reg[21], self.general_purpose_reg[22], self.general_purpose_reg[23], self.general_purpose_reg[24], self.general_purpose_reg[25], self.general_purpose_reg[26], self.general_purpose_reg[27], self.general_purpose_reg[28], self.general_purpose_reg[29], self.general_purpose_reg[30], self.general_purpose_reg[31], self.eipc, self.eipsw, self.fepc, self.fepsw, self.ecr, self.psw.get(), self.tkcw, chcw, self.adtre));
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
        // Interrupts are disabled during a duplexed exception (i.e. it only applies for internal exceptions)
        if self.psw.interrupt_disable || self.psw.exception_pending || self.psw.nmi_pending {
            // Ignore
            return;
        }

        // Second nibble is the same as interrupt level
        let interrupt_level = ((request.code() >> 4) & 0xF) as u8;

        if interrupt_level < self.psw.interrupt_level {
            // Level not high enough to perform interrupt. Skip
            return;
        }

        if self.psw.nmi_pending {
            // Fatal exception. Terminating program and halting
            panic!(
                "Fatal exception. Code: {:04X}, PSW: {:08X}, PC: {:08X}",
                request.code(),
                self.psw.get(),
                self.pc
            )
        }

        self.perform_exception(request.code());

        if self.psw.interrupt_level < 15 {
            // Mask interrupts at this level or lower
            // Update interrupt mask level _after_ exception PSW copy
            self.psw.interrupt_level = interrupt_level + 1;
        }
    }

    fn perform_exception(&mut self, code: usize) {
        if self.psw.exception_pending {
            // Duplexed exception
            // Set duplex code
            self.ecr = ((code as u32 & 0xFFFF) << 16) | (self.ecr & 0xFFFF);
            // Backup PSW and PC
            self.fepsw = self.psw.get();
            self.fepc = self.pc;
            // Prevent further stacked exceptions
            self.psw.nmi_pending = true;

            // We always jump to a particular duplexed exception program
            self.pc = 0xFFFF_FFD0;
        } else {
            // Set interrupt code into cause code segment
            self.ecr = (self.ecr & 0xFFFF_0000) | code as u32;
            // Backup PSW
            self.eipsw = self.psw.get();
            // Backup PC
            self.eipc = self.pc;

            self.pc = 0xFFFF_0000 | code as u32;
        }

        self.psw.exception_pending = true;
        self.psw.interrupt_disable = true;
        self.psw.address_trap_enable = false;

        self.processing_bitstring = false;

        // Clear halt
        self.is_halted = false;
    }

    fn perform_instruction(&mut self, bus: &mut Bus, instruction: u16) -> (u32, BusActivity) {
        let opcode = instruction >> 10;

        match opcode {
            // Register transfer
            0b01_0000 => self.mov(instruction, true),
            0b00_0000 => self.mov(instruction, false),
            0b10_1000 => self.movea(instruction, bus),
            0b10_1111 => self.movhi(instruction, bus),

            // Load and Input
            0b11_1000 => self.load_inst_16(bus, instruction, 0xFFFF_FFFF, 0xFF, 0), // IN.B Input single byte
            0b11_1001 => self.load_inst_16(bus, instruction, 0xFFFF_FFFE, 0xFFFF, 0), // IN.H Input 16 bit word

            0b11_1011 | 0b11_0011 => self.ld_w(instruction, bus),
            0b11_0000 => self.load_inst_16(bus, instruction, 0xFFFF_FFFF, 0xFF, 8), // LD.B Load single byte (sign extend)
            0b11_0001 => self.load_inst_16(bus, instruction, 0xFFFF_FFFE, 0xFFFF, 16), // LD.H Load 16 bit word (sign extend)

            // Store and Output
            0b11_1100 | 0b11_0100 => self.st_b(instruction, bus),
            0b11_1101 | 0b11_0101 => self.st_h(instruction, bus),
            0b11_1111 | 0b11_0111 => self.st_w(instruction, bus),

            // Arithmetic
            0b01_0001 => self.add(instruction, true), // ADD immediate
            0b00_0001 => self.add(instruction, false), // ADD reg
            0b10_1001 => self.add_16_bit(instruction, bus),
            0b01_0011 => self.cmp(instruction, true),
            0b00_0011 => self.cmp(instruction, false),
            0b00_1001 => self.div(instruction, true),
            0b00_1011 => self.div(instruction, false),
            0b00_1000 => self.mul_signed(instruction),
            0b00_1010 => self.mul_unsigned(instruction),
            0b00_0010 => self.sub(instruction),

            // Bitwise
            0b00_1101 => self.and(instruction),
            0b10_1101 => self.andi(instruction, bus),
            0b00_1111 => self.not(instruction),
            0b00_1100 => self.or(instruction),
            0b10_1100 => self.ori(instruction, bus),
            0b01_0111 => self.sar(instruction, true), // SAR Shift arthmetic right by immediate
            0b00_0111 => self.sar(instruction, false), // SAR Shift arthmetic right by register
            0b01_0100 => self.shl(instruction, true), // SHL Shift logical left by immediate
            0b00_0100 => self.shl(instruction, false), // SHL Shift logical left by register
            0b01_0101 => self.shr(instruction, true), // SHR Shift logical right by immediate
            0b00_0101 => self.shr(instruction, false), // SHR Shift logical right by register
            0b00_1110 => self.xor(instruction, false, bus), // XOR register
            0b10_1110 => self.xor(instruction, true, bus), // XOR immediate, zero extend

            // CPU Control
            // 0b10_0 + 4 bits
            // Opcode is 6 bits, whereas this needs 7, so we just grab the relevant opcode
            // ranges, then extract the condition from the instruction inside
            0b10_0000..=0b10_0111 => self.bcond(instruction),
            0b01_1010 => self.halt(),
            0b10_1011 => self.displaced_jump(bus, instruction, true), // JAL Jump and link
            0b00_0110 => self.jmp(instruction),
            0b10_1010 => self.displaced_jump(bus, instruction, false), // JR Jump relative
            0b01_1100 => self.ldsr(instruction), // LDSR Load to system register
            0b01_1001 => self.reti(),            // RETI Return from trap or interrupt
            0b01_1101 => self.stsr(instruction), // STSR Store contents of system register
            0b01_1000 => self.trap(instruction), // TRAP Raise exception

            0b11_1110 => self.float_inst(instruction, bus), // Floating point operations and Nintendo

            0b01_1111 => self.bit_string_inst(instruction, bus), // Bit string operations

            // Miscellaneous
            0b11_1010 => self.caxi(), // CAXI Compare and exchange interlocked
            0b01_0010 => self.setf(instruction),

            // Nintendo
            0b01_0110 => self.cli(), // CLI Clear interrupt disable flag
            0b01_1110 => self.sei(), // SEI Set interrupt disable flag
            _ => {
                #[cfg(feature = "panic")]
                panic!("Invalid opcode {opcode:x}");

                (1, BusActivity::Standard)
            }
        }
    }

    fn fetch_instruction_word(&mut self, bus: &mut Bus) -> u16 {
        // let instruction = bus.get_u16(self.pc);
        // Hack for speed. This will break if there are instructions fetched from outside of ROM
        let instruction = bus.get_rom(self.pc >> 1);

        // Increment PC by 2 bytes
        self.pc = self.pc.wrapping_add(2);

        instruction
    }

    fn set_gen_purpose_reg(&mut self, index: usize, value: u32) {
        if index == 0 {
            // Do not write to r0
            // println!("Attempted write to register 0");
            return;
        }

        self.general_purpose_reg[index] = value;
    }

    // Instructions

    fn mov(&mut self, instruction: u16, use_immediate: bool) -> (u32, BusActivity) {
        let (reg1_index_or_immediate, reg2_index) = extract_reg1_2_index(instruction);

        let value = if use_immediate {
            sign_extend(reg1_index_or_immediate as u32, 5)
        } else {
            self.general_purpose_reg[reg1_index_or_immediate]
        };

        self.set_gen_purpose_reg(reg2_index, value);

        (1, BusActivity::Standard)
    }

    fn movea(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
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

    fn movhi(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
        // MOVHI Add upper immediate
        // Don't modify flags
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];

        let immediate = self.fetch_instruction_word(bus);

        let result = reg1.wrapping_add((immediate as u32) << 16);

        self.set_gen_purpose_reg(reg2_index, result);

        (1, BusActivity::Standard)
    }

    fn ld_w(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let disp = self.fetch_instruction_word(bus) as u32;
        let disp = (disp as i16) as u32;

        let address = self.general_purpose_reg[reg1_index].wrapping_add(disp) & 0xFFFF_FFFC;

        let value = bus.get_u32(address);

        self.set_gen_purpose_reg(reg2_index, value);

        (self.load_inst_cycle_count(), BusActivity::Load)
    }

    fn st_b(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
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

    fn st_h(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
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

    fn st_w(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
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

    fn add(&mut self, instruction: u16, use_immediate: bool) -> (u32, BusActivity) {
        let (reg1_index_or_immediate, reg2_index) = extract_reg1_2_index(instruction);

        let value = if use_immediate {
            sign_extend(reg1_index_or_immediate as u32, 5)
        } else {
            self.general_purpose_reg[reg1_index_or_immediate]
        };

        let reg2 = self.general_purpose_reg[reg2_index];

        self.add_inst(value, reg2, reg2_index)
    }

    fn add_16_bit(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
        // ADD 16 bit immediate
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);
        let immediate = self.fetch_instruction_word(bus);
        let immediate = (immediate as i16) as u32;

        let reg1 = self.general_purpose_reg[reg1_index];

        self.add_inst(reg1, immediate, reg2_index)
    }

    fn cmp(&mut self, instruction: u16, use_immediate: bool) -> (u32, BusActivity) {
        let (reg1_index_or_immediate, reg2_index) = extract_reg1_2_index(instruction);

        let value = if use_immediate {
            sign_extend(reg1_index_or_immediate as u32, 5)
        } else {
            self.general_purpose_reg[reg1_index_or_immediate]
        };

        let reg2 = self.general_purpose_reg[reg2_index];

        self.sub_inst(reg2, value, None)
    }

    fn div(&mut self, instruction: u16, signed: bool) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];
        let reg2 = self.general_purpose_reg[reg2_index];

        let cycles = if signed { 38 } else { 36 };

        if reg1 == 0 {
            println!("Divide by zero");
            self.perform_exception(0xFF80);

            return (cycles, BusActivity::Long);
        }

        let (result, remainder, overflow) = if signed {
            if reg2 == 0x8000_0000 && reg1 == 0xFFFF_FFFF {
                // Special case to set overflow
                (0x8000_0000, 0, true)
            } else {
                let reg1 = reg1 as i32;
                let reg2 = reg2 as i32;

                let remainder = (reg2 % reg1) as u32;
                let result = (reg2 / reg1) as u32;

                (result, remainder, false)
            }
        } else {
            let remainder = reg2 % reg1;
            let result = reg2 / reg1;

            (result, remainder, false)
        };

        // Remainder
        self.general_purpose_reg[30] = remainder;
        self.set_gen_purpose_reg(reg2_index, result);
        self.psw.update_alu_flags(result, overflow, None);

        (cycles, BusActivity::Long)
    }

    fn mul_signed(&mut self, instruction: u16) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index] as i32;
        let reg2 = self.general_purpose_reg[reg2_index] as i32;

        // We don't do an overflowing_mul because we're looking for a 32 bit overflow, not 64 bit, but
        // we still want to calculate the lower 32 bits of the result
        let result = (reg1 as i64) * (reg2 as i64);
        let result = result as u64;

        let result_low = (result & 0xFFFF_FFFF) as u32;

        // If low 32 bits (sign extended) are not the same as the final result, we overflowed 32 bits
        let overflow = result != ((result_low as i32) as u64);

        self.set_gen_purpose_reg(30, (result >> 32) as u32);
        self.set_gen_purpose_reg(reg2_index, result_low);
        // Multiplication only uses lower 32 bits
        self.psw.update_alu_flags(result_low, overflow, None);

        (13, BusActivity::Long)
    }

    fn mul_unsigned(&mut self, instruction: u16) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];
        let reg2 = self.general_purpose_reg[reg2_index];

        // We don't do an overflowing_mul because we're looking for a 32 bit overflow, not 64 bit, but
        // we still want to calculate the lower 32 bits of the result
        let result = (reg1 as u64) * (reg2 as u64);
        let result_low = (result & 0xFFFF_FFFF) as u32;

        // If low 32 bits are not the same as the final result, we overflowed 32 bits
        let overflow = result != (result_low as u64);

        self.set_gen_purpose_reg(30, (result >> 32) as u32);
        self.set_gen_purpose_reg(reg2_index, result_low);
        // Multiplication only uses lower 32 bits
        self.psw.update_alu_flags(result_low, overflow, None);

        (13, BusActivity::Long)
    }

    fn sub(&mut self, instruction: u16) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];
        let reg2 = self.general_purpose_reg[reg2_index];

        self.sub_inst(reg2, reg1, Some(reg2_index))
    }

    fn and(&mut self, instruction: u16) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];
        let reg2 = self.general_purpose_reg[reg2_index];

        let result = reg2 & reg1;

        self.set_gen_purpose_reg(reg2_index, result);
        self.psw.update_alu_flags(result, false, None);

        (1, BusActivity::Standard)
    }

    fn andi(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
        // ANDI immediate, zero extended
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];
        let immediate = self.fetch_instruction_word(bus) as u32;

        let result = reg1 & immediate;

        self.set_gen_purpose_reg(reg2_index, result);
        self.psw.update_alu_flags(result, false, None);
        // There appears to be a docs bug that indicates that sign is always false. Both Mednafen and rustual-boy don't have this
        // self.psw.sign = false;

        (1, BusActivity::Standard)
    }

    fn not(&mut self, instruction: u16) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];

        // Interestingly, Rust uses ! for bitwise NOT
        let result = !reg1;

        self.set_gen_purpose_reg(reg2_index, result);
        self.psw.update_alu_flags(result, false, None);

        (1, BusActivity::Standard)
    }

    fn or(&mut self, instruction: u16) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];
        let reg2 = self.general_purpose_reg[reg2_index];

        let result = reg2 | reg1;

        self.set_gen_purpose_reg(reg2_index, result);
        self.psw.update_alu_flags(result, false, None);

        (1, BusActivity::Standard)
    }

    fn ori(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
        // ORI immediate, zero extend
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];
        let immediate = self.fetch_instruction_word(bus) as u32;

        let result = reg1 | immediate;

        self.set_gen_purpose_reg(reg2_index, result);
        self.psw.update_alu_flags(result, false, None);

        (1, BusActivity::Standard)
    }

    fn sar(&mut self, instruction: u16, use_immediate: bool) -> (u32, BusActivity) {
        // SAR Shift arthmetic right by immediate/register
        let (reg1_index_or_immediate, store_reg_index) = extract_reg1_2_index(instruction);

        let reg2 = self.general_purpose_reg[store_reg_index];
        let shift = if use_immediate {
            reg1_index_or_immediate as u32
        } else {
            self.general_purpose_reg[reg1_index_or_immediate]
        };

        // Limit to shift by 32
        let shift = shift & 0x1F;

        // Per https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
        // Arithmetic right shift on signed integer types, logical right shift on unsigned integer types
        // So we use a signed type
        let (result, carry) = if shift > 0 {
            let carry_result = (reg2 as i32) >> (shift - 1);

            // One last shift to finish it
            let result = (carry_result >> 1) as u32;

            // Carry is the last bit that's shifted out
            let carry = carry_result & 1 != 0;

            (result, carry)
        } else {
            (reg2, false)
        };

        self.set_gen_purpose_reg(store_reg_index, result);
        self.psw.update_alu_flags(result, false, Some(carry));

        (1, BusActivity::Standard)
    }

    fn shr(&mut self, instruction: u16, use_immediate: bool) -> (u32, BusActivity) {
        // SHR Shift logical right by immediate/register
        let (reg1_index_or_immediate, store_reg_index) = extract_reg1_2_index(instruction);

        let reg2 = self.general_purpose_reg[store_reg_index];
        let shift = if use_immediate {
            reg1_index_or_immediate as u32
        } else {
            self.general_purpose_reg[reg1_index_or_immediate]
        };

        // Limit to shift by 32
        let shift = shift & 0x1F;

        // Per https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
        // Arithmetic right shift on signed integer types, logical right shift on unsigned integer types
        // So we use a signed type
        let (result, carry) = if shift > 0 {
            let carry_result = reg2 >> (shift - 1);

            // One last shift to finish it
            let result = carry_result >> 1;

            // Carry is the last bit that's shifted out
            let carry = carry_result & 1 != 0;

            (result, carry)
        } else {
            (reg2, false)
        };

        self.set_gen_purpose_reg(store_reg_index, result);
        self.psw.update_alu_flags(result, false, Some(carry));

        (1, BusActivity::Standard)
    }

    fn shl(&mut self, instruction: u16, use_immediate: bool) -> (u32, BusActivity) {
        // SHL Shift logical left by immediate/register
        let (reg1_index_or_immediate, store_reg_index) = extract_reg1_2_index(instruction);

        let reg2 = self.general_purpose_reg[store_reg_index];
        let shift = if use_immediate {
            reg1_index_or_immediate as u32
        } else {
            self.general_purpose_reg[reg1_index_or_immediate]
        };

        // Limit to shift by 32
        let shift = shift & 0x1F;

        let (result, carry) = if shift > 0 {
            let carry_result = reg2 << (shift - 1);

            // One last shift to finish it
            let result = carry_result << 1;

            // Carry is the last bit that's shifted out
            let carry = reg2 != 0 && carry_result & 0x8000_0000 != 0;

            (result, carry)
        } else {
            (reg2, false)
        };

        self.set_gen_purpose_reg(store_reg_index, result);
        self.psw.update_alu_flags(result, false, Some(carry));

        (1, BusActivity::Standard)
    }

    fn xor(&mut self, instruction: u16, use_immediate: bool, bus: &mut Bus) -> (u32, BusActivity) {
        let (reg1_index, reg2_index) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];
        let value = if use_immediate {
            self.fetch_instruction_word(bus) as u32
        } else {
            self.general_purpose_reg[reg2_index]
        };

        let result = reg1 ^ value;

        self.set_gen_purpose_reg(reg2_index, result);
        self.psw.update_alu_flags(result, false, None);

        (1, BusActivity::Standard)
    }

    fn bcond(&mut self, instruction: u16) -> (u32, BusActivity) {
        let condition = (instruction >> 9) & 0xF;

        let condition = self.indexed_flag(condition);

        if condition {
            // Jumping
            let disp = instruction & 0x1FF;
            let disp = sign_extend(disp as u32, 9);

            // PC was already incremented by 2, so we have to remove that
            self.pc = (self.pc - 2).wrapping_add(disp & 0xFFFF_FFFE);

            (3, BusActivity::Standard)
        } else {
            // Don't jump
            (1, BusActivity::Standard)
        }
    }

    fn halt(&mut self) -> (u32, BusActivity) {
        self.is_halted = true;

        (1, BusActivity::Standard)
    }

    fn jmp(&mut self, instruction: u16) -> (u32, BusActivity) {
        let (reg1_index, _) = extract_reg1_2_index(instruction);

        let reg1 = self.general_purpose_reg[reg1_index];

        self.pc = reg1 & 0xFFFF_FFFE;

        (3, BusActivity::Standard)
    }

    fn ldsr(&mut self, instruction: u16) -> (u32, BusActivity) {
        // LDSR Load to system register
        let (reg_id, reg2_index) = extract_reg1_2_index(instruction);

        let reg2 = self.general_purpose_reg[reg2_index];

        match reg_id {
            0 => self.eipc = reg2 & 0xFFFF_FFFE,
            1 => {
                // Don't write unused or reserved bits
                self.eipsw = reg2 & 0x000F_F3FF
            }
            2 => self.fepc = reg2 & 0xFFFF_FFFE,
            3 => self.fepsw = reg2 & 0x000F_F3FF,
            // 4 => ECR not setable
            5 => self.psw.set(reg2),
            // 6 => pir
            7 => self.tkcw = reg2,
            24 => {
                println!("WARNING: Writing to cache control register");
                // TODO: Finish implementation
                // Cache enabled is stored here because it can be read
                self.cache_enabled = reg2 & 0x2 != 0;
            }
            25 => self.adtre = reg2,
            29 => self.unknown_29 = reg2,
            // 30 not setable
            // 30 => self.unknown_30 = reg2,
            31 => self.unknown_31 = reg2,
            _ => {}
        }

        // TODO: Are flags supposed to be set here?

        // TODO: Mednafen has this set to 1 cycle
        // The V810 manual doesn't list a cycle count
        (8, BusActivity::Standard)
    }

    fn reti(&mut self) -> (u32, BusActivity) {
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

    fn stsr(&mut self, instruction: u16) -> (u32, BusActivity) {
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
                // PIR Processor ID register
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

    fn trap(&mut self, instruction: u16) -> (u32, BusActivity) {
        // TRAP Raise exception and set restore PC
        let (reg1_index, _) = extract_reg1_2_index(instruction);

        self.perform_exception(0xFFA0 + reg1_index);

        (15, BusActivity::Standard)
    }

    fn float_inst(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
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
                self.psw
                    .update_float_flags(result as f32, true, true, false, false, false, true);
                self.psw.update_alu_flags(result as u32, false, None);

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
                if reg1_float == 0.0 {
                    if reg2_float == 0.0 {
                        println!("Divide float zero by zero");

                        self.psw.float_zero_divide = true;
                        self.perform_exception(0xFF70);
                    } else {
                        println!("Divide float by zero");

                        self.psw.float_zero_divide = true;
                        self.perform_exception(0xFF68);
                    }

                    return (44, BusActivity::Standard);
                }

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
                self.psw
                    .update_float_flags(result as f32, true, true, false, false, false, true);

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

            _ => {
                #[cfg(feature = "panic")]
                panic!("Invalid float or Nintendo instruction {sub_opcode:x}");

                (1, BusActivity::Standard)
            }
        }
    }

    fn bit_string_inst(&mut self, instruction: u16, bus: &mut Bus) -> (u32, BusActivity) {
        // Bit string operations
        let (sub_opcode, _) = extract_reg1_2_index(instruction);

        self.processing_bitstring = false;

        if sub_opcode < 4 {
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

    fn caxi(&mut self) -> (u32, BusActivity) {
        // CAXI Compare and exchange interlocked
        println!("CAXI doesn't really do anything on VB");

        (26, BusActivity::Standard)
    }

    fn setf(&mut self, instruction: u16) -> (u32, BusActivity) {
        // SETF Set flag condition
        let (_, reg2_index) = extract_reg1_2_index(instruction);

        let condition = instruction & 0xF;

        let value = if self.indexed_flag(condition) { 1 } else { 0 };

        self.set_gen_purpose_reg(reg2_index, value);

        (1, BusActivity::Standard)
    }

    fn cli(&mut self) -> (u32, BusActivity) {
        // CLI Clear interrupt disable flag
        self.psw.interrupt_disable = false;

        // TODO: Mednafen has this set to 1 cycle
        (12, BusActivity::Standard)
    }

    fn sei(&mut self) -> (u32, BusActivity) {
        // SEI Set interrupt disable flag
        self.psw.interrupt_disable = true;

        // TODO: Mednafen has this set to 1 cycle
        (12, BusActivity::Standard)
    }

    // Utilities

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

    fn displaced_jump(
        &mut self,
        bus: &mut Bus,
        instruction: u16,
        save_pc: bool,
    ) -> (u32, BusActivity) {
        let upper_disp = (instruction & 0x3FF) as u32;
        let disp = self.fetch_instruction_word(bus) as u32;

        let disp = sign_extend((upper_disp << 16) | disp, 26) & 0xFFFF_FFFE;

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

            // Mark that we're still doing a bitstring op
            self.processing_bitstring = true;
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
        // println!("Running bit string. May have errors?");
        // Docs seem to be wrong about masking out 26 bits?
        let mut dest_offset = self.general_purpose_reg[26] & 0x1F;
        self.set_gen_purpose_reg(26, dest_offset);

        let mut source_offset = self.general_purpose_reg[27] & 0x1F;
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
                _ => {
                    #[cfg(feature = "panic")]
                    panic!("Invalid bit string instruction {sub_opcode:x}")
                }
            }

            // Make sure we can access borrowed data
            drop(dest_bit);
            bus.set_u32(dest_addr, dest_word.data);

            length -= 1;

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

                break;
            } else {
                dest_offset += 1;
            }
        }

        if length != 0 {
            // We haven't finished this operation. Rewind PC
            self.pc -= 2;

            // Mark that we're still doing a bitstring op
            self.processing_bitstring = true;
        }

        // TODO: Do these need to be updated constantly and are interrupts allowed to interrupt this?
        self.set_gen_purpose_reg(26, dest_offset);
        self.set_gen_purpose_reg(27, source_offset);
        self.set_gen_purpose_reg(28, length);
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

#[inline(always)]
fn extract_reg1_2_index(instruction: u16) -> (usize, usize) {
    (
        extract_reg1_index(instruction),
        extract_reg2_index(instruction),
    )
}

#[inline(always)]
fn extract_reg2_index(instruction: u16) -> usize {
    ((instruction >> 5) & 0x1F) as usize
}

#[inline(always)]
fn extract_reg1_index(instruction: u16) -> usize {
    (instruction & 0x1F) as usize
}
