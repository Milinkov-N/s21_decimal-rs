use crate::{BitStr, S21Decimal, Sign};

use std::cmp::Ordering::{Greater, Less};

impl std::ops::Add for S21Decimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut left = self;
        let mut right = rhs;

        match (left.is_max(), left.sign(), right.is_max(), right.sign()) {
            (true, Sign::Positive, _, Sign::Positive)
            | (_, Sign::Positive, true, Sign::Positive) => panic!("overflow"),
            _ => (),
        }

        match (left.is_min(), left.sign(), right.is_min(), right.sign()) {
            (true, Sign::Negative, _, Sign::Negative)
            | (_, Sign::Negative, true, Sign::Negative) => panic!("underflow"),
            _ => (),
        }

        match left.scale().cmp(&right.scale()) {
            Less => left.normalize(right.scale() as u32),
            Greater => right.normalize(left.scale() as u32),
            _ => (),
        }

        let lbs = BitStr::new(left.sign(), left.scale(), &left.bits[0..3]);
        let rbs = BitStr::new(right.sign(), right.scale(), &right.bits[0..3]);

        S21Decimal::from(lbs + rbs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! decimal_add_tc {
        ($(#[$attr:meta])? $name:ident { left: $lhs:literal, right: $rhs:literal, expect: $expect:literal }) => {
            #[test]
            $(#[$attr])?
            fn $name() {
                let l = S21Decimal::from_str_radix(stringify!($lhs), 10);
                let r = S21Decimal::from_str_radix(stringify!($rhs), 10);

                assert_eq!(S21Decimal::from_str_radix(stringify!($expect), 10), l + r);
            }
        };

        ($(#[$attr:meta])? $name:ident { left: $lhs:literal, right: $rhs:literal, expect: $expect:expr }) => {
            #[test]
            $(#[$attr])?
            fn $name() {
                let l = S21Decimal::from_str_radix(stringify!($lhs), 10);
                let r = S21Decimal::from_str_radix(stringify!($rhs), 10);

                assert_eq!($expect, l + r);
            }
        };
    }

    #[test]
    fn decimal_add_left_right_pos_scale_28() {
        let l = S21Decimal::new(1, 28); // 1e-28
        let r = S21Decimal::new(1, 28); // 1e-28

        /* 1e-28 + 1e-28 = 2e-28 */
        assert_eq!(S21Decimal::new(2, 28), l + r);
    }

    decimal_add_tc!(decimal_add_zeroes {
        left: 0,
        right: 0,
        expect: 0
    });

    decimal_add_tc!(decimal_add_ones {
        left: 1,
        right: 1,
        expect: 2
    });

    decimal_add_tc!(decimal_add_left_neg_scale_3 {
        left: -0.005,
        right: 5,
        expect: 4.995
    });

    decimal_add_tc!(decimal_add_left_right_neg_scale_3 {
        left: 5,
        right: -0.005,
        expect: 4.995
    });

    decimal_add_tc!(decimal_add_left_right_scale_5 {
        left: 0.00005,
        right: 0.00005,
        expect: 0.00010
    });

    decimal_add_tc!(decimal_add_left_neg_10_right_5 {
        left: -10,
        right: 5,
        expect: -5
    });

    decimal_add_tc!(decimal_add_left_neg_10_right_neg_5 {
        left: -10,
        right: -5,
        expect: -15
    });

    decimal_add_tc!(decimal_add_left_large_right_one {
        left: 8_700_600_500_400_300_200_100,
        right: 1,
        expect: 8_700_600_500_400_300_200_101
    });

    decimal_add_tc!(decimal_add_left_large_neg_right_one {
        left: -8_700_600_500_400_300_200_100,
        right: 1,
        expect: -8_700_600_500_400_300_200_099
    });

    decimal_add_tc!(decimal_add_left_max_right_neg_one {
        left: 79_228_162_514_264_337_593_543_950_335,
        right: -1,
        expect: 79_228_162_514_264_337_593_543_950_334
    });

    decimal_add_tc!(
        #[should_panic = "overflow"]
        decimal_add_left_max_right_one {
            left: 79_228_162_514_264_337_593_543_950_335,
            right: 1,
            expect: 0
        }
    );

    decimal_add_tc!(
        #[should_panic = "overflow"]
        decimal_add_left_max_scale_28_right_one {
            left: 7.9_228_162_514_264_337_593_543_950_335,
            right: 1,
            expect: 0
        }
    );

    decimal_add_tc!(
        #[should_panic = "overflow"]
        decimal_add_left_one_right_max {
            left: 1,
            right: 79_228_162_514_264_337_593_543_950_335,
            expect: 0
        }
    );

    decimal_add_tc!(
        #[should_panic = "overflow"]
        decimal_add_left_one_right_max_scale_28 {
            left: 1,
            right: 7.9_228_162_514_264_337_593_543_950_335,
            expect: 0
        }
    );

    decimal_add_tc!(
        #[should_panic = "underflow"]
        decimal_add_left_min_right_neg_one {
            left: -79_228_162_514_264_337_593_543_950_335,
            right: -1,
            expect: 0
        }
    );

    decimal_add_tc!(
        #[should_panic = "underflow"]
        decimal_add_left_min_scale_28_right_neg_one {
            left: -7.9_228_162_514_264_337_593_543_950_335,
            right: -1,
            expect: 0
        }
    );

    decimal_add_tc!(
        #[should_panic = "underflow"]
        decimal_add_left_neg_one_right_min {
            left: -1,
            right: -79_228_162_514_264_337_593_543_950_335,
            expect: 0
        }
    );

    decimal_add_tc!(
        #[should_panic = "underflow"]
        decimal_add_left_neg_one_right_min_scale_28 {
            left: -1,
            right: -7.9_228_162_514_264_337_593_543_950_335,
            expect: 0
        }
    );
}
