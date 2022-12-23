use s21_decimal::*;

fn main() {
    let ds: DecStr = DecStr::from("7_9_228_162_514_264_337_593_543_950_335");
    let ds2 = DecStr::from("0.9_228_162_514_264_337_593_543_950_335");
    let sum = ds.add(&ds2);
    // let bstr: BitStr = sum.clone().into();

    println!("{}", sum);
    println!("rounded: {}", sum.banker_round());
    // println!("bstr = {bstr:?}");
}
