use s21_decimal::*;

fn main() {
    let l = BitStr::from_str_radix("79_228_162_514_264_337_593_543_950_335", 10);
    println!("{l:#?}");
    println!("l is max {}", S21Decimal::from(l.clone()).is_max());
    let r = BitStr::from_str_radix("-1", 10);
    println!("{r:#?}");
    println!("{:#?}", l + r);
}
