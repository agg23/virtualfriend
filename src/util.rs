pub fn sign_extend(value: u32, size: u8) -> u32 {
    // Per https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
    // Arithmetic right shift on signed integer types, logical right shift on unsigned integer types
    // So we use a signed type
    let value = value as i32;
    let result = (value << (32 - size)) >> (32 - size);

    result as u32
}

pub fn sign_extend_16(value: u16, size: u8) -> i16 {
    // Per https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
    // Arithmetic right shift on signed integer types, logical right shift on unsigned integer types
    // So we use a signed type
    let value = value as i16;
    let result = (value << (16 - size)) >> (16 - size);

    result
}
