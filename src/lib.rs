mod bits;

use bits::*;

const EXP_MASK: i32 = 0b00000000111111110000000000000000;
const BITSTR_LEN: usize = 96;

enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, PartialEq)]
pub struct S21Decimal {
    bits: [i32; 4],
}

impl S21Decimal {
    pub fn is_negative(&self) -> bool {
        get_bit(self.bits[3], 31) == 1
    }

    fn set_negative(&mut self) {
        set_bit(&mut self.bits[3], 31);
    }

    fn sign(&self) -> Sign {
        if get_bit(self.bits[3], 31) == 1 {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }

    pub fn scale(&self) -> i32 {
        (self.bits[3] & EXP_MASK) >> 16
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

        decimal
    }
}

impl std::ops::Add for S21Decimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let sign = self.sign();
        let scale = self.scale();
        let bitstr = BitStr::from(self);

        let rhs_sign = rhs.sign();
        let rhs_scale = rhs.scale();
        let rhs_bitstr = BitStr::from(rhs);

        use std::cmp::Ordering::{Greater, Less};
        match scale.cmp(&rhs_scale) {
            Less => match rhs_sign {
                Sign::Positive => todo!("Exponent normalization"),
                Sign::Negative => todo!("Exponent normalization"),
            },
            Greater => match sign {
                Sign::Positive => todo!("Exponent normalization"),
                Sign::Negative => todo!("Exponent normalization"),
            },
            _ => (),
        }

        todo!()
    }
}

#[derive(Debug, PartialEq, Clone)]
struct BitStr {
    bytes: [u8; BITSTR_LEN],
}

impl BitStr {
    const LENGTH: usize = BITSTR_LEN;
}

impl Default for BitStr {
    fn default() -> Self {
        Self { bytes: [48; 96] }
    }
}

impl From<S21Decimal> for BitStr {
    fn from(decimal: S21Decimal) -> Self {
        decimal.bits[0..3]
            .iter()
            .map(|i| i.bits())
            .reduce(|mut acc, i| {
                acc.push_str(&i);
                acc
            })
            .unwrap()
            .into()
    }
}

impl From<String> for BitStr {
    fn from(s: String) -> Self {
        let mut bitstr = BitStr::default();

        if s.len() == Self::LENGTH {
            s.as_bytes()
                .iter()
                .enumerate()
                .for_each(|(i, byte)| bitstr.bytes[i] = *byte);
        } else if s.len() < Self::LENGTH {
            let diff = Self::LENGTH - s.len();
            s.as_bytes()
                .iter()
                .enumerate()
                .for_each(|(i, byte)| bitstr.bytes[i + diff] = *byte);
        }

        bitstr
    }
}

impl From<&str> for BitStr {
    fn from(s: &str) -> Self {
        let mut bitstr = BitStr::default();

        if s.len() == Self::LENGTH {
            s.as_bytes()
                .iter()
                .enumerate()
                .for_each(|(i, byte)| bitstr.bytes[i] = *byte);
        } else if s.len() < Self::LENGTH {
            let diff = Self::LENGTH - s.len();
            s.as_bytes()
                .iter()
                .enumerate()
                .for_each(|(i, byte)| bitstr.bytes[i + diff] = *byte);
        }

        bitstr
    }
}

impl std::ops::Add for BitStr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = BitStr::default();
        let mut carry = 0;

        self.bytes.iter().enumerate().rev().for_each(|(i, bit)| {
            let bit = bit - 48;
            let rhs_bit = rhs.bytes[i] - 48;

            if bit ^ rhs_bit ^ carry == 1 {
                result.bytes[i] = '1' as u8;
                if bit + rhs_bit + carry < 3 {
                    carry = 0;
                }
            } else if bit == 1 && rhs_bit == 1 {
                carry = 1;
            }
        });

        result
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
    fn add_bitstr() {
        let left = BitStr::from("10");
        let right = BitStr::from("01");
        let expecting = BitStr::from("11");
        assert_eq!(expecting, left + right);
    }
}
