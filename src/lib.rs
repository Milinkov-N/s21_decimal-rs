use bits::*;
pub use bitstr::*;
pub use decstr::*;

mod bits;
mod bitstr;
mod decstr;
mod ops;

const EXP_MASK: i32 = 0b00000000111111110000000000000000;

#[derive(Debug, PartialEq, Clone)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, PartialEq)]
pub struct S21Decimal {
    bits: [i32; 4],
}

impl S21Decimal {
    pub fn new(integer: i32, scale: i32) -> Self {
        let mut decimal = Self::from(integer);
        decimal.set_scale(scale);

        decimal
    }

    pub fn from_str_radix(str: &str, radix: u32) -> Self {
        BitStr::from_str_radix(str, radix).into()
    }

    pub const fn is_negative(&self) -> bool {
        get_bit(self.bits[3], 31) == 1
    }

    fn set_negative(&mut self) {
        set_bit(&mut self.bits[3], 31);
    }

    const fn sign(&self) -> Sign {
        if get_bit(self.bits[3], 31) == 1 {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }

    fn set_sign(&mut self, sign: Sign) {
        match sign {
            Sign::Positive => unset_bit(&mut self.bits[3], 31),
            Sign::Negative => set_bit(&mut self.bits[3], 31),
        }
    }

    pub const fn scale(&self) -> i32 {
        (self.bits[3] & EXP_MASK) >> 16
    }

    fn set_scale(&mut self, exp: i32) {
        let sign = self.sign();
        self.bits[3] = exp << 16;
        self.set_sign(sign);
    }

    pub fn normalize(&mut self, scale: u32) {
        let mut bstr = BitStr::from(&self.bits[0..3]);

        bstr.pow(scale);
        bstr.scale = scale as i32;
        let normalized: S21Decimal = bstr.into();

        self.set_sign(normalized.sign());
        self.set_scale(normalized.scale());
        self.bits = normalized.bits;
    }

    pub fn is_max(&self) -> bool {
        match (self.sign(), self.bits) {
            (Sign::Positive, [-1, -1, -1, _]) => true,
            _ => false,
        }
    }

    pub fn is_min(&self) -> bool {
        match (self.sign(), self.bits) {
            (Sign::Negative, [-1, -1, -1, _]) => true,
            _ => false,
        }
    }
}

impl Default for S21Decimal {
    fn default() -> Self {
        Self {
            bits: Default::default(),
        }
    }
}

impl From<i32> for S21Decimal {
    fn from(src: i32) -> Self {
        let mut src = src;
        let mut decimal = S21Decimal::default();

        if src.is_negative() {
            decimal.set_negative();
            src = -src;
        }

        decimal.bits[0] = src;

        decimal
    }
}

impl From<BitStr> for S21Decimal {
    fn from(bstr: BitStr) -> Self {
        let mut decimal = S21Decimal::default();
        let mut index = BitStr::LENGTH;

        for i in 0..3 {
            for j in 0..32 {
                index -= 1;
                let bit = bstr.bytes[index] - 48;

                if bit == 1 {
                    set_bit(&mut decimal.bits[i], j);
                }
            }
        }

        decimal.set_sign(bstr.sign);
        decimal.set_scale(bstr.scale);
        decimal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_default() {
        assert_eq!(
            S21Decimal {
                bits: Default::default()
            },
            S21Decimal::default()
        );
    }

    #[test]
    fn decimal_from_int_pos() {
        assert_eq!(S21Decimal { bits: [1, 0, 0, 0] }, S21Decimal::from(1));
    }

    #[test]
    fn decimal_from_int_neg() {
        let decimal = S21Decimal::from(-1);
        assert_eq!(1, decimal.bits[0]);
        assert_eq!(true, decimal.is_negative());
    }

    #[test]
    fn decimal_from_bitstr() {
        let bstr = BitStr::default();

        assert_eq!(S21Decimal::default(), bstr.into());
    }

    #[test]
    fn decimal_into_bstr() {
        let decimal = S21Decimal::default();

        assert_eq!(BitStr::default(), decimal.into());
    }

    #[test]
    fn decimal_from_str_radix_base_10_num_45() {
        let decimal = S21Decimal::from_str_radix("45", 10);

        assert_eq!(S21Decimal::from(45), decimal);
    }
}
