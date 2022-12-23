use super::{get_bit, Bits, S21Decimal, Sign};

const BITSTR_LEN: usize = 96;

#[derive(PartialEq, Clone)]
pub struct BitStr {
    pub sign: Sign,
    pub scale: i32,
    pub bytes: [u8; BITSTR_LEN],
}

impl BitStr {
    pub const LENGTH: usize = BITSTR_LEN;

    pub fn new(sign: Sign, scale: i32, bytes: &[i32]) -> Self {
        let mut res = Self::from(bytes);
        res.sign = sign;
        res.scale = scale;

        res
    }

    pub fn from_str_radix(str: &str, radix: u32) -> Self {
        let mut result = BitStr::default();
        let point = str.find('.');

        match radix {
            2 => BitStr::from(str),
            10 => {
                let iter = str.chars().peekable();

                let mut size = iter
                    .clone()
                    .filter(|ch| ch.is_ascii_digit() || ch.is_ascii_hexdigit())
                    .count();

                iter.filter(|ch| ch.is_ascii_digit() || ch.is_ascii_hexdigit())
                    .enumerate()
                    .for_each(|(i, ch)| {
                        result = result.clone() + BitStr::from(ch);
                        if i < size - 1 {
                            result.pow(1);
                        }
                    });

                if str.starts_with("-") {
                    result.sign = Sign::Negative;
                    size += 1;
                }

                if let Some(scale) = point {
                    result.scale = (size - scale) as i32;
                }

                result
            }
            _ => unimplemented!(),
        }
    }

    pub fn all_zeroes(&self) -> bool {
        self.bytes.iter().take_while(|&&b| b == 48).count() == Self::LENGTH
    }

    pub fn all_ones(&self) -> bool {
        self.bytes.iter().take_while(|&&b| b == 49).count() == Self::LENGTH
    }

    pub fn shift(&mut self, offset: u32) {
        (0..offset).for_each(|_| {
            for i in 0..Self::LENGTH - 1 {
                self.bytes[i] = self.bytes[i + 1];
            }
            self.bytes[Self::LENGTH - 1] = '0' as u8;
        });
    }

    pub fn invert(&mut self, upto: usize) {
        for k in Self::LENGTH - upto..Self::LENGTH {
            if self.bytes[k] - 48 == 1 {
                self.bytes[k] = '0' as u8;
            } else {
                self.bytes[k] = '1' as u8;
            }
        }
    }

    pub fn pow(&mut self, exp: u32) {
        let mut exponent = 10;
        let mut result = BitStr::default();
        (0..exp - 1).for_each(|_| exponent *= 10);

        for i in 0..32u32 {
            if get_bit(exponent, i as i32) == 1 {
                let mut mask = self.clone();
                mask.shift(i);
                result = result + mask;
            }
        }

        self.bytes = result.bytes;
    }

    pub fn msbi(&self) -> Option<usize> {
        self.bytes.iter().position(|&b| b == 49)
    }

    pub fn cmp_bytes(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }

    pub fn add_upto(&self, other: &BitStr, upto: usize) -> Self {
        let mut result = BitStr {
            sign: self.sign.clone(),
            scale: self.scale,
            bytes: [48; 96],
        };
        let mut carry = 0;

        self.bytes
            .iter()
            .enumerate()
            .rev()
            .take_while(|(i, _)| *i >= Self::LENGTH - upto)
            .for_each(|(i, bit)| {
                let bit = bit - 48;
                let rhs_bit = other.bytes[i] - 48;

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

    fn add_negative(&self, rhs: &BitStr) -> Self {
        let mut result = BitStr::default();
        let mut left = self.clone();
        let mut right = rhs.clone();

        match self.cmp_bytes(&right) {
            std::cmp::Ordering::Greater => {
                if let Some(msbi) = self.msbi() {
                    right.invert(BitStr::LENGTH - msbi);
                }

                if self.sign == Sign::Negative {
                    result.sign = Sign::Negative;
                }
            }

            _ => {
                if let Some(msbi) = right.msbi() {
                    left.invert(BitStr::LENGTH - msbi);
                }
            }
        }

        left.sign = Sign::Positive;
        let lbs_add_one = left + BitStr::from("1");
        let sum = right.add_upto(&lbs_add_one, BitStr::LENGTH - right.msbi().unwrap());

        if !sum.all_zeroes() {
            result.scale = self.scale;
        }

        result.bytes = sum.bytes;
        result
    }

    fn add_positive(&self, rhs: &Self) -> Self {
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

        if !result.all_zeroes() {
            result.scale = self.scale;
        }

        result.sign = self.sign.clone();
        result
    }
}

impl Default for BitStr {
    fn default() -> Self {
        Self {
            sign: Sign::Positive,
            scale: 0,
            bytes: [48; 96],
        }
    }
}

impl std::ops::Add for BitStr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use Sign::*;

        match (&self.sign, &rhs.sign) {
            (Positive, Positive) | (Negative, Negative) => self.add_positive(&rhs),
            (_, _) => self.add_negative(&rhs),
        }
    }
}

macro_rules! bitstr_from_decimal {
    ($type:ty) => {
        impl From<$type> for BitStr {
            fn from(decimal: $type) -> Self {
                let mut bstr: BitStr = decimal.bits[0..3]
                    .iter()
                    .rev()
                    .map(|i| i.bits())
                    .reduce(|mut acc, i| {
                        acc.push_str(&i);
                        acc
                    })
                    .unwrap()
                    .into();

                bstr.sign = decimal.sign();
                bstr.scale = decimal.scale();

                bstr
            }
        }
    };
}

bitstr_from_decimal!(S21Decimal);
bitstr_from_decimal!(&S21Decimal);
bitstr_from_decimal!(&mut S21Decimal);

impl From<&[i32]> for BitStr {
    fn from(s: &[i32]) -> Self {
        let s = s
            .iter()
            .fold(Vec::new(), |mut acc, bit| {
                let chars = (0..32).fold(Vec::new(), |mut v, i| {
                    v.push((get_bit(*bit, i) as u8 + 48) as char);
                    v
                });

                let string = chars.iter().rev().collect::<String>();
                acc.push(string);

                acc
            })
            .iter()
            .rev()
            .fold(String::new(), |mut acc, s| {
                acc.push_str(s);

                acc
            });

        BitStr::from(s)
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

impl From<char> for BitStr {
    fn from(ch: char) -> Self {
        match ch.to_ascii_lowercase() {
            '0'..='9' => BitStr::from(format!("{:b}", ch as u8 - 48)),
            'a'..='f' => BitStr::from(format!("{:b}", ch as u8 - 87)),
            _ => panic!("char is not in range of '0'..='9' || 'a'..='f'"),
        }
    }
}

impl From<BitStr> for String {
    fn from(b: BitStr) -> Self {
        b.bytes.iter().fold(String::new(), |mut acc, byte| {
            acc.push(*byte as char);
            acc
        })
    }
}

impl From<&BitStr> for String {
    fn from(b: &BitStr) -> Self {
        b.bytes.iter().fold(String::new(), |mut acc, byte| {
            acc.push(*byte as char);
            acc
        })
    }
}

impl std::fmt::Debug for BitStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitStr")
            .field("sign", &self.sign)
            .field("scale", &self.scale)
            .field("bytes", &String::from(self))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitstr_from_char() {
        let test_cases = ['0', '1', '2', '5', '9', 'a', 'f'];
        let expecting = [
            BitStr::default(),
            BitStr::from("1"),
            BitStr::from("10"),
            BitStr::from("101"),
            BitStr::from("1001"),
            BitStr::from("1010"),
            BitStr::from("1111"),
        ];

        test_cases.iter().enumerate().for_each(|(i, tc)| {
            assert_eq!(expecting[i], BitStr::from(*tc));
        });
    }

    #[test]
    fn bitstr_from_str_radix_10_num_45() {
        let bstr = BitStr::from_str_radix("45", 10);

        assert_eq!(BitStr::from("101101"), bstr);
    }

    #[test]
    fn bitstr_from_str_radix_10_num_45_scale_1() {
        let bstr = BitStr::from_str_radix("-4.5", 10);
        let mut expecting = BitStr::from("101101");
        expecting.sign = Sign::Negative;
        expecting.scale = 1;

        assert_eq!(expecting, bstr);
    }

    #[test]
    fn bitstr_from_str_radix_10_num_5_scale_5() {
        let bstr = BitStr::from_str_radix("0.00005", 10);
        let mut expecting = BitStr::from("101");
        expecting.scale = 5;

        assert_eq!(expecting, bstr);
    }

    #[test]
    fn bitstr_from_str_radix_10_num_max() {
        let bstr = BitStr::from_str_radix("79_228_162_514_264_337_593_543_950_335", 10);

        assert_eq!(
            BitStr::from(format!("{}", "1".repeat(BitStr::LENGTH))),
            bstr
        );
    }

    #[test]
    fn bitstr_all_zeroes_zeroes() {
        assert_eq!(true, BitStr::default().all_zeroes())
    }

    #[test]
    fn bitstr_all_zeroes_one() {
        assert_eq!(
            false,
            BitStr::from(format!("1{}", "0".repeat(95))).all_zeroes()
        )
    }

    #[test]
    fn bitstr_all_ones_ones() {
        let mut bstr = BitStr::default();
        bstr.invert(BitStr::LENGTH);
        assert_eq!(true, bstr.all_ones())
    }

    #[test]
    fn bitstr_all_ones_zero() {
        assert_eq!(
            false,
            BitStr::from(format!("0{}", "1".repeat(95))).all_ones()
        )
    }

    #[test]
    fn bitstr_shift_one() {
        let mut bstr = BitStr::from("1");
        bstr.shift(3);
        assert_eq!(BitStr::from("1000"), bstr);
    }

    #[test]
    fn bitstr_shift_ten() {
        let mut bstr = BitStr::from("1010");
        bstr.shift(10);
        assert_eq!(BitStr::from("10100000000000"), bstr);
    }

    #[test]
    fn bitstr_invert_one() {
        let mut bstr = BitStr::from("1");
        bstr.invert(1);

        assert_eq!(BitStr::from("0"), bstr);
    }

    #[test]
    fn bitstr_invert_one_full_str() {
        let mut bstr = BitStr::from("1");
        bstr.invert(BitStr::LENGTH);

        assert_eq!(
            BitStr::from(format!("{}{}", "1".repeat(BitStr::LENGTH - 1), "0")),
            bstr
        );
    }

    #[test]
    fn bitstr_invert_to_highest_byte() {
        let mut bstr = BitStr::from("101");
        bstr.invert(BitStr::LENGTH - bstr.msbi().unwrap());

        assert_eq!(BitStr::from("10"), bstr);
    }

    #[test]
    fn bitstr_five_pow_one() {
        let mut bstr = BitStr::from("101");
        bstr.pow(1);

        assert_eq!(BitStr::from("110010"), bstr);
    }

    #[test]
    fn bitstr_five_pow_three() {
        let mut bstr = BitStr::from("101");
        bstr.pow(3);

        assert_eq!(BitStr::from("1001110001000"), bstr);
    }

    #[test]
    fn bitstr_add() {
        let left = BitStr::from("10");
        let right = BitStr::from("01");
        let expecting = BitStr::from("11");
        assert_eq!(expecting, left + right);
    }

    #[test]
    fn bitstr_add_upto() {
        let l = BitStr::from("111");
        let r = BitStr::from("1");

        assert_eq!(BitStr::default(), l.add_upto(&r, 3));
    }
}
