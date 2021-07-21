pub fn f64_to_i32(x: f64) -> i32 {
    x.round().rem_euclid(2f64.powi(32)) as u32 as i32
}
