use super::{BitStr, Sign};

use std::cmp::Ordering::*;

const DECSTR_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq)]
pub struct DecStr {
    pub sign: Sign,
    pub scale: i32,
    pub bytes: [u8; DECSTR_LEN],
}

impl DecStr {
    const LENGTH: usize = DECSTR_LEN;

    pub fn add(&self, other: &Self) -> Self {
        let mut result = self.clone();

        Self::_add(&mut result, &other);

        result
    }

    pub fn add_mut(&mut self, other: &Self) {
        Self::_add(self, &other);
    }

    pub fn integral(&self) -> String {
        self.to_string()
            .trim_start_matches('0')
            .split_at(29 - self.scale as usize)
            .0
            .to_owned()
    }

    pub fn fraction(&self) -> String {
        self.to_string()
            .trim_start_matches('0')
            .split_at(29 - self.scale as usize)
            .1
            .to_owned()
    }

    pub fn rescale(&self, scale: i32) -> Self {
        let mut res = self.clone();

        if scale > 0 {
            res.bytes.rotate_left(scale as usize);
            res.scale += scale;
        } else if scale < 0 {
            let mut num_of_elem = -scale;
            res.bytes
                .iter_mut()
                .rev()
                .take_while(|_| {
                    num_of_elem -= 1;
                    num_of_elem >= 0
                })
                .for_each(|c| *c = 48);
            res.bytes.rotate_right(-scale as usize);
            res.scale += scale;
        }

        res
    }

    pub fn banker_round(&self) -> Self {
        if self.scale > 0 {
            return match (self.bytes[Self::LENGTH - self.scale as usize - 1] - 48) % 2 {
                0 => self.cut_fractional(),
                _ => self.cut_fractional().add(&DecStr::from("1")),
            };
        }

        self.clone()
    }

    pub fn len(&self) -> usize {
        self.bytes.iter().skip_while(|c| **c == 48).count()
    }

    fn cut_fractional(&self) -> Self {
        self.clone().rescale(-self.scale)
    }

    fn _add(dest: &mut Self, rhs: &Self) {
        use Sign::*;
        match (&dest.sign, &rhs.sign) {
            (Positive, Positive) | (Negative, Negative) => {
                Self::calc_add(dest, rhs, |l, r, carry| {
                    let mut sum = l + r + *carry as u8;

                    if *carry > 0 {
                        *carry -= 1;
                    }

                    if sum >= 10 {
                        *carry += 1;
                        sum -= 10;
                    }

                    sum + 48
                })
            }
            _ => Self::calc_add(dest, rhs, |l, r, carry| {
                let mut sum = l as i8 - r as i8 - *carry as i8;

                if *carry > 0 {
                    *carry -= 1;
                }

                if sum < 0 {
                    *carry += 1;
                    sum += 10;
                }

                sum as u8 + 48
            }),
        }
    }

    fn calc_add<F>(dest: &mut Self, rhs: &Self, calcs: F)
    where
        F: Fn(u8, u8, &mut i32) -> u8,
    {
        let mut rhs = rhs.clone();
        let mut carry = 0;

        match dest.scale.cmp(&rhs.scale) {
            Less => *dest = dest.rescale(rhs.scale - dest.scale),
            Equal => (),
            Greater => rhs = rhs.rescale(dest.scale - rhs.scale),
        }

        for i in (0..DECSTR_LEN).rev() {
            let left = dest.bytes[i] - 48;
            let right = rhs.bytes[i] - 48;

            dest.bytes[i] = calcs(left, right, &mut carry);
        }
    }
}

impl std::fmt::Display for DecStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .bytes
            .as_slice()
            .iter()
            .map(|b| *b as char)
            .fold(String::new(), |mut acc, c| {
                acc.push(c);
                acc
            })
            .trim_start_matches('0')
            .to_owned();

        let split = string.split_at(string.len() - self.scale as usize);

        write!(
            f,
            "{}{}",
            if split.0.is_empty() { "0" } else { split.0 },
            if split.1.is_empty() {
                String::new()
            } else {
                format!(".{}", split.1)
            }
        )
    }
}

impl Default for DecStr {
    fn default() -> Self {
        Self {
            sign: Sign::Positive,
            scale: 0,
            bytes: [48; DECSTR_LEN],
        }
    }
}

impl From<BitStr> for DecStr {
    fn from(bs: BitStr) -> Self {
        let mut res = DecStr::default();
        let mut idx = 0;

        bs.bytes.iter().rev().for_each(|b| {
            if *b == 49 {
                let mut pow_of_two = DecStr::from("1");

                for _ in 0..idx {
                    pow_of_two.add_mut(&pow_of_two.clone());
                }

                res.add_mut(&pow_of_two);
            }

            idx += 1;
        });

        res.sign = bs.sign;
        res.scale = bs.scale;
        res
    }
}

impl From<&str> for DecStr {
    fn from(s: &str) -> Self {
        let signed = s.starts_with('-');
        let point = s.find(".");
        let filtered = s.chars().filter(|c| c.is_ascii_digit()).collect::<String>();

        let mut result: DecStr = filtered.as_bytes().into();

        if signed {
            result.sign = Sign::Negative;
        }

        if let Some(scale) = point {
            result.scale = (s
                .split_at(scale + 1)
                .1
                .chars()
                .filter(|c| c.is_ascii_digit())
                .count()) as i32;
        }

        result
    }
}

impl From<&[u8]> for DecStr {
    fn from(s: &[u8]) -> Self {
        let mut ds = DecStr::default();

        if s.len() == Self::LENGTH {
            s.iter()
                .enumerate()
                .for_each(|(i, byte)| ds.bytes[i] = *byte);
        } else if s.len() < Self::LENGTH {
            let diff = Self::LENGTH - s.len();
            s.iter()
                .enumerate()
                .for_each(|(i, byte)| ds.bytes[i + diff] = *byte);
        } else {
            panic!(
                "Byte slice is too long. Expected {} got {}",
                Self::LENGTH,
                s.len()
            );
        }

        ds
    }
}

#[cfg(test)]
mod tests {
    use crate::{DecStr, Sign};

    macro_rules! from_str_assert {
        (from: $from:literal, expect: {
            sign: $sign:expr,
            scale: $scale:literal,
            bytes: $bytes:literal
        }) => {
            let ds = DecStr::from(stringify!($from));
            let mut expecting = DecStr::from($bytes.as_bytes());
            expecting.sign = $sign;
            expecting.scale = $scale;
            assert_eq!(expecting, ds);
        };
    }

    macro_rules! decstr_add_assert {
        ($lhs:literal + $rhs:literal = $expect:literal) => {
            let l = DecStr::from(stringify!($lhs));
            let r = DecStr::from(stringify!($rhs));
            assert_eq!(DecStr::from(stringify!($expect)), l.add(&r));
        };
    }

    #[test]
    fn decstr_from_str_0() {
        from_str_assert!(
            from: 0,
            expect: {
                sign: Sign::Positive,
                scale: 0,
                bytes: "0"
            }
        );
    }

    #[test]
    fn decstr_from_str_1() {
        from_str_assert!(
            from: 1,
            expect: {
                sign: Sign::Positive,
                scale: 0,
                bytes: "1"
            }
        );
    }

    #[test]
    fn decstr_from_str_neg_1() {
        from_str_assert!(
            from: -1,
            expect: {
                sign: Sign::Negative,
                scale: 0,
                bytes: "1"
            }
        );
    }

    #[test]
    fn decstr_from_str_neg_1_scale_1() {
        from_str_assert!(
            from: -0.1,
            expect: {
                sign: Sign::Negative,
                scale: 1,
                bytes: "1"
            }
        );
    }

    #[test]
    fn decstr_from_str_neg_1_scale_28() {
        from_str_assert!(
            from: -0.0000000000000000000000000001,
            expect: {
                sign: Sign::Negative,
                scale: 28,
                bytes: "1"
            }
        );
    }

    #[test]
    fn decstr_from_str_with_underscores() {
        from_str_assert!(
            from: 123_456_234,
            expect: {
                sign: Sign::Positive,
                scale: 0,
                bytes: "123456234"
            }
        );
    }

    #[test]
    fn decstr_add_left_one_right_one() {
        decstr_add_assert!(1 + 1 = 2);
    }

    #[test]
    fn decstr_add_left_45_scale_1_right_one_scale_2() {
        decstr_add_assert!(4.5 + 0.01 = 4.51);
    }

    #[test]
    fn decstr_add_left_45_right_neg_one() {
        decstr_add_assert!(45 + -1 = 44);
    }
}
