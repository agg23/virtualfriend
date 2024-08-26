use bitvec::prelude::*;

pub struct ProgramStatusWord {
    pub zero: bool,
    pub sign: bool,
    pub overflow: bool,
    pub carry: bool,

    /// FPR - Set when the result of a floating-point operation is subjected to rounding and suffers precision degradation.
    pub float_precision: bool,
    /// FUD - Set when the result of a floating-point operation is too small to be represented as a normal floating short value.
    pub float_underflow: bool,
    /// FOV - Set when the result of a floating-point operation is too large to be represented by the floating short data type.
    pub float_overflow: bool,
    /// FZD - Set when the DIVF.S instruction is executed with a divisor of zero.
    pub float_zero_divide: bool,
    /// FIV - Set when an invalid floating-point operation attempted.
    pub float_invalid: bool,
    /// FRO - Set when a floating-point operation is attempted with a reserved operand.
    pub float_reserved: bool,

    pub interrupt_disable: bool,
    pub nmi_pending: bool,
    pub interrupt_level: u8,

    pub address_trap_enable: bool,

    pub exception_pending: bool,
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
        let mut value = bitarr![u32, Lsb0; 0; 32];
        value.set(0, self.zero);
        value.set(1, self.sign);
        value.set(2, self.overflow);
        value.set(3, self.carry);

        value.set(4, self.float_precision);
        value.set(5, self.float_underflow);
        value.set(6, self.float_overflow);
        value.set(7, self.float_zero_divide);
        value.set(8, self.float_invalid);
        value.set(9, self.float_reserved);

        value.set(12, self.interrupt_disable);
        value.set(13, self.address_trap_enable);
        value.set(14, self.exception_pending);
        value.set(15, self.nmi_pending);

        let (_, interrupt_level) = value.split_at_mut(16);
        interrupt_level.store(self.interrupt_level as u16);

        value.load()
    }

    pub fn set(&mut self, value: u32) {
        let array = BitArray::<_, Lsb0>::new([value]);

        self.zero = *array.get(0).unwrap();
        self.sign = *array.get(1).unwrap();
        self.overflow = *array.get(2).unwrap();
        self.carry = *array.get(3).unwrap();

        self.float_precision = *array.get(4).unwrap();
        self.float_underflow = *array.get(5).unwrap();
        self.float_overflow = *array.get(6).unwrap();
        self.float_zero_divide = *array.get(7).unwrap();
        self.float_invalid = *array.get(8).unwrap();
        self.float_reserved = *array.get(9).unwrap();

        self.interrupt_disable = *array.get(12).unwrap();
        self.address_trap_enable = *array.get(13).unwrap();
        self.exception_pending = *array.get(14).unwrap();
        self.nmi_pending = *array.get(15).unwrap();

        self.interrupt_level = ((value >> 16) & 0xF) as u8;
    }

    pub fn update_alu_flags(&mut self, alu_value: u32, overflow: bool, carry: Option<bool>) {
        self.zero = alu_value == 0;
        self.sign = (alu_value & 0x8000_0000) != 0;
        self.overflow = overflow;

        if let Some(carry) = carry {
            self.carry = carry;
        }
    }

    pub fn update_float_flags(
        &mut self,
        value: f32,
        set_fro: bool,
        _set_fiv: bool,
        _set_fzd: bool,
        set_fov: bool,
        set_fud: bool,
        _set_fpr: bool,
    ) {
        // https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#numeric-cast
        if set_fro && value.is_nan() {
            self.float_reserved = true;
        }

        if set_fov && value == f32::INFINITY {
            self.float_overflow = true;
        }

        if set_fud && value == f32::NEG_INFINITY {
            self.float_underflow = true;
        }

        // TODO: I don't know how to detect this in Rust
        // self.float_precision = false;

        self.zero = value == 0.0;
        self.sign = value.is_sign_negative();
        self.overflow = false;
        self.carry = value.is_sign_negative();
    }
}
