pub fn sign_extend(mut x: u16, bit_count: u16) -> u16 {
    // Early return if bit_count is 0
    if bit_count == 0 {
        return x;
    }

    if ((x >> (bit_count - 1)) & 1) != 0 {
        x |= (0xFFFF) << bit_count;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0u16, 1), 0u16);
        assert_eq!(sign_extend(13u16, 0), 13u16);
        assert_eq!(sign_extend(13u16, 5), 13u16);
        assert_eq!(sign_extend(13u16, 4), 0b1111_1111_1111_1101u16);
    }

    #[test]
    fn test_end_swap() {
        assert_eq!(0x6969u16.rotate_right(8), 0x6969u16);
        assert_eq!(0x6699u16.rotate_right(8), 0x9966u16);
        assert_eq!(0x6996u16.rotate_right(8), 0x9669u16);
        assert_eq!(
            0b1100_1010_1101_0110u16.rotate_right(8),
            0b1101_0110_1100_1010u16
        );
    }
}
