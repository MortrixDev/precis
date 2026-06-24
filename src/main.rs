use precis::real::Real;

fn main() {
    println!("5 + 7 = {}", Real::from(5) + Real::from(7));
    println!("288 - 88 = {}", Real::from(288) - Real::from(88));
    println!("{}", Real::from(std::f64::consts::PI));
}
