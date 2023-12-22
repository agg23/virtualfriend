use bitvec::prelude::*;

pub struct ProgramStatusWord {
    pub zero: bool,
    pub sign: bool,
    pub overflow: bool,
    pub carry: bool,

    float_precision: bool,
    float_underflow: bool,
    float_overflow: bool,
    float_zero_divide: bool,
    float_invalid: bool,
    float_reserved: bool,

    interrupt_disable: bool,
    pub nmi_pending: bool,
    interrupt_level: u8,

    address_trap_enable: bool,

    exception_pending: bool,
}

impl ProgramStatusWord {
    pub fn new() -> Self {
        ProgramStatusWord {
            zero: false,
            sign: false,
            overflow: false,
            carry: false,
            float_precision: false,
            float_underflow: false,
            float_overflow: false,
            float_zero_divide: false,
            float_invalid: false,
            float_reserved: false,
            interrupt_disable: false,
            nmi_pending: false,
            interrupt_level: 0,
            address_trap_enable: false,
            exception_pending: false,
        }
    }

    pub fn get(&self) -> u32 {
        let mut value = bitarr![u32, Lsb0;];
        value.set(0, self.zero);
        value.set(1, self.sign);
        value.set(2, self.overflow);
        value.set(3, self.carry);

        value.set(4, self.float_precision);
        value.set(5, self.float_underflow);
        value.set(6, self.float_overflow);
        value.set(7, self.float_zero_divide);
        value.set(8, self.float_invalid);

        value.set(9, self.interrupt_disable);
        value.set(10, self.address_trap_enable);
        value.set(11, self.exception_pending);
        value.set(12, self.nmi_pending);

        let (_, interrupt_level) = value.split_at_mut(16);
        interrupt_level.store(self.interrupt_level);

        value.load()
    }

    pub fn set(&mut self, value: u32) {
        todo!("Implement");
    }

    pub fn update_alu_flags_u32(&mut self, alu_value: u32, overflow: bool, carry: Option<bool>) {
        self.zero = alu_value == 0;
        self.sign = (alu_value & 0x8000_0000) != 0;
        self.overflow = overflow;

        if let Some(carry) = carry {
            self.carry = carry;
        }
    }

    pub fn update_alu_flags_u64(&mut self, alu_value: u64, overflow: bool, carry: Option<bool>) {
        self.zero = alu_value == 0;
        self.sign = (alu_value & 0x8000_0000_0000_0000) != 0;
        self.overflow = overflow;

        if let Some(carry) = carry {
            self.carry = carry;
        }
    }
}
