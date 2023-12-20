struct CPU_V810 {
    pc: u32,

    // 31 general purpose registers, and r0 == 0
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
    general_purpose_reg: [u32; 31],

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
    psw: u32,

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
}
