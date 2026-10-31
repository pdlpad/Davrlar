pub const fn rgb(r: u32, g: u32, b: u32) -> u32 {
    (255 << 24) | ((b & 255) << 16) | ((g & 255) << 8) | (r & 255)
}
pub const fn hex(v: u32) -> u32 {
    rgb((v >> 16) & 255, (v >> 8) & 255, v & 255)
}
pub fn parse(s: &str) -> u32 {
    hex(u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap_or(0))
}
const fn red(c: u32) -> u32 {
    c & 255
}
const fn green(c: u32) -> u32 {
    (c >> 8) & 255
}
const fn blue(c: u32) -> u32 {
    (c >> 16) & 255
}

pub const WHITE: u32 = hex(0xffffff);

pub fn clamp(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}
pub fn mix(a: u32, b: u32, t: f64) -> u32 {
    let t = clamp(t);
    let m = |x: u32, y: u32| (x as f64 + (y as f64 - x as f64) * t) as u32;
    rgb(m(red(a), red(b)), m(green(a), green(b)), m(blue(a), blue(b)))
}
pub fn scale(c: u32, k: f64) -> u32 {
    let m = |x: u32| (x as f64 * k).min(255.0) as u32;
    rgb(m(red(c)), m(green(c)), m(blue(c)))
}

fn hash(ns: &[i32]) -> u32 {
    let mut h: u32 = 0x9e37_79b9;
    for n in ns {
        h = (h ^ *n as u32).wrapping_mul(0x85eb_ca6b);
        h ^= h >> 13;
        h = h.wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
    }
    h
}
fn rand(a: i32, b: i32) -> f64 {
    hash(&[a, b]) as f64 / 4294967296.0
}
pub fn noise(x: f64, seed: f64) -> f64 {
    let i = x.floor();
    let f = x - i;
    let u = f * f * (3.0 - 2.0 * f);
    let (i, s) = (i as i32, seed as i32);
    let (a, b) = (rand(i, s), rand(i + 1, s));
    (a + (b - a) * u) * 2.0 - 1.0
}
