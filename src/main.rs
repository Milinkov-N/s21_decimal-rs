use s21_decimal::*;

fn main() {
    let ds: DecStr = DecStr::from("79");
    let ds2 = DecStr::from("1.6");
    let sum = ds.add(&ds2);
    // let bstr: BitStr = sum.clone().into();

    println!("{}({})", sum, sum.scale);
    // println!("rounded: {}", sum.banker_round());
    // println!("bstr = {bstr:?}");
}
