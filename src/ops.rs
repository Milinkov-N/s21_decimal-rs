use crate::{BitStr, S21Decimal};

use std::cmp::Ordering::{Greater, Less};

impl std::ops::Add for S21Decimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut left = self;
        let mut right = rhs;

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

    #[test]
    fn decimal_add_zeroes() {
        let l = S21Decimal::from(0);
        let r = S21Decimal::from(0);

        assert_eq!(S21Decimal::from(0), l + r);
    }

    #[test]
    fn decimal_add_ones() {
        let l = S21Decimal { bits: [1, 0, 0, 0] };
        let r = S21Decimal { bits: [1, 0, 0, 0] };

        assert_eq!(S21Decimal::from(2), l + r);
    }

    #[test]
    fn decimal_add_left_neg_scale_3() {
        let l = S21Decimal::new(-5, 3); // -0.005
        let r = S21Decimal::from(5);

        /* -0.005 + 5 = 4.995 */
        assert_eq!(S21Decimal::new(4995, 3), l + r);
    }

    #[test]
    fn decimal_add_right_neg_scale_3() {
        let l = S21Decimal::from(5);
        let r = S21Decimal::new(-5, 3); // -0.005

        /* 5 + -0.005 = 4.995 */
        assert_eq!(S21Decimal::new(4995, 3), l + r);
    }

    #[test]
    fn decimal_add_left_right_neg_scale_3() {
        let l = S21Decimal::new(-5, 3); // -0.005
        let r = S21Decimal::new(-5, 3); // -0.005

        /* -0.005 + -0.005 = 0 */
        assert_eq!(S21Decimal::new(0, 3), l + r);
    }

    #[test]
    fn decimal_add_left_right_pos_scale_5() {
        let l = S21Decimal::new(5, 5); // 0.00005
        let r = S21Decimal::new(5, 5); // 0.00005

        /* 0.00005 + 0.00005 = 0.0001 */
        assert_eq!(S21Decimal::new(10, 5), l + r);
    }

    #[test]
    fn decimal_add_left_right_pos_scale_28() {
        let l = S21Decimal::new(1, 28); // 1e-28
        let r = S21Decimal::new(1, 28); // 1e-28

        /* 1e-28 + 1e-28 = 2e-28 */
        assert_eq!(S21Decimal::new(2, 28), l + r);
    }
}
