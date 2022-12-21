pub trait Bits {
    fn bits(&self) -> String;
}

impl Bits for i32 {
    fn bits(&self) -> String {
        (0..32)
            .into_iter()
            .map(|i| char::from(get_bit(*self, i) as u8 + 48))
            .rev()
            .collect::<String>()
    }
}

pub(crate) const fn get_bit(value: i32, bit: i32) -> i32 {
    (value >> bit) & 1
}

pub(crate) fn set_bit(value: &mut i32, bit: i32) {
    *value |= 1 << bit;
}

pub(crate) fn unset_bit(value: &mut i32, bit: i32) {
    *value &= !(1 << bit);
}
