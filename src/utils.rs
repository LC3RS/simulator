pub fn sign_extend(mut x: u16, bit_count: u16) -> u16 {
    if ((x >> (bit_count - 1)) & 1) != 0 {
        x |= (0xFFFF) << bit_count;
    }
    x
}
