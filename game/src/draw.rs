use crate::color::{clamp, hex, mix, noise, parse, scale, WHITE};
use davrlar_core::data::Data;
use davrlar_core::sim::Unit;
use serde::Deserialize;
use std::collections::HashMap;

pub const W: i32 = 640;
pub const H: i32 = 360;
pub const GROUND_Y: i32 = 300;
const TEAM: [u32; 2] = [hex(0x3d7bff), hex(0xff4a3d)];
const TEAM_DARK: [u32; 2] = [hex(0x1c3f94), hex(0x8f1f1a)];
const SKIN: u32 = hex(0xe8b98a);
const INK: u32 = hex(0x0d0b14);
const CYAN: u32 = hex(0x4af0ff);

#[derive(Deserialize)]
pub struct Bucket {
    sky: [String; 2],
    ground: [String; 2],
}
#[derive(Deserialize)]
struct Buckets {
    buckets: Vec<Bucket>,
}
pub fn load_buckets(json: &str) -> Vec<Bucket> {
    serde_json::from_str::<Buckets>(json).expect("разбор game.json").buckets
}

struct Pal {
    metal: u32,
    metal_d: u32,
    cloth: u32,
    dark: u32,
}
const PAL_BUCKET: [Pal; 5] = [
    Pal { metal: hex(0xc9a45c), metal_d: hex(0x8a6a36), cloth: hex(0xc8b38a), dark: hex(0x4a3820) },
    Pal { metal: hex(0xb9c3d3), metal_d: hex(0x6f7a8c), cloth: hex(0x8a6a4a), dark: hex(0x2a2d38) },
    Pal { metal: hex(0x8a8f99), metal_d: hex(0x4f535c), cloth: hex(0xe8e2cf), dark: hex(0x2a2118) },
    Pal { metal: hex(0x8f9a74), metal_d: hex(0x586149), cloth: hex(0x6b7355), dark: hex(0x232a1e) },
    Pal { metal: hex(0xc8e8ff), metal_d: hex(0x4a6aa0), cloth: hex(0x2c3558), dark: hex(0x0e1224) },
];

pub struct Anim {
    pub walk: i32,
    pub atk: f64,
    pub flash: f64,
}

pub struct Frame {
    pub color: Vec<u32>,
    bg: HashMap<usize, Vec<u32>>,
}

pub fn base_height(era: usize) -> i32 {
    56 + era.min(24) as i32
}
pub fn turret_pos(o: usize, slot: usize, era: usize) -> (i32, i32) {
    let lx = 10 + slot as i32 * 14;
    (if o == 0 { lx } else { W - 1 - lx }, GROUND_Y + 4 - base_height(era) - 2)
}

impl Frame {
    pub fn new() -> Frame {
        Frame { color: vec![0; (W * H) as usize], bg: HashMap::new() }
    }
    fn px(&mut self, x: i32, y: i32, c: u32) {
        if x >= 0 && y >= 0 && x < W && y < H {
            self.color[(y * W + x) as usize] = c;
        }
    }
    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        for j in y.max(0)..(y + h).min(H) {
            for i in x.max(0)..(x + w).min(W) {
                self.color[(j * W + i) as usize] = c;
            }
        }
    }
    fn blend(&mut self, x: i32, y: i32, c: u32, a: f64) {
        if x >= 0 && y >= 0 && x < W && y < H {
            let i = (y * W + x) as usize;
            self.color[i] = mix(self.color[i], c, a);
        }
    }
    fn line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, c: u32) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let mut e = dx + dy;
        loop {
            self.px(x0, y0, c);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * e;
            if e2 >= dy {
                e += dy;
                x0 += sx;
            }
            if e2 <= dx {
                e += dx;
                y0 += sy;
            }
        }
    }

    pub fn background(&mut self, data: &Data, buckets: &[Bucket], era: usize) {
        let bg = self.bg.entry(era).or_insert_with(|| paint_background(data, buckets, era));
        self.color.copy_from_slice(bg);
    }

    fn base_rect(&mut self, o: usize, flash: f64, lx: i32, ly: i32, ww: i32, hh: i32, c: u32) {
        let c = if flash > 0.0 { mix(c, WHITE, flash) } else { c };
        let x0 = if o == 0 { lx } else { W - lx - ww };
        self.rect(x0, GROUND_Y + 4 - ly, ww, hh, c);
    }

    pub fn base(&mut self, data: &Data, o: usize, era: usize, hp_frac: f64, flash: f64) {
        let b = data.eras[era].bucket;
        let h = base_height(era);
        let mut rr = |lx: i32, ly: i32, ww: i32, hh: i32, c: u32| self.base_rect(o, flash, lx, ly, ww, hh, c);
        let (team, team_d) = (TEAM[o], TEAM_DARK[o]);
        let stone = [hex(0xcfc6b0), hex(0x9d947e), hex(0x6f6858)];
        match b {
            0 => {
                rr(0, 34, 52, 34, stone[0]);
                rr(0, 34, 52, 3, stone[1]);
                rr(0, 4, 52, 4, stone[2]);
                for i in 0..4 {
                    rr(5 + i * 12, 30, 5, 28, stone[1]);
                    rr(5 + i * 12, 30, 2, 28, stone[0]);
                }
                rr(-2, 38, 56, 5, hex(0xb9602a));
                rr(22, 14, 10, 14, hex(0x2a2118));
                rr(46, h, 2, 18, hex(0x5a3a1c));
                rr(36, h + 6, 10, 8, team);
            }
            1 => {
                rr(8, 44, 40, 40, stone[1]);
                rr(8, 44, 40, 2, stone[0]);
                rr(0, 54, 14, 54, stone[0]);
                rr(38, 54, 14, 54, stone[0]);
                for tx in [0, 6, 38, 44] {
                    rr(tx, 62, 4, 5, stone[0]);
                }
                for i in 0..3 {
                    rr(16 + i * 10, 48, 5, 4, stone[0]);
                }
                rr(22, 20, 12, 20, hex(0x2a2118));
                rr(22, 22, 12, 3, hex(0x6a4a2a));
                rr(24, h + 2, 2, 14, hex(0x4a3820));
                rr(26, h + 2, 12, 9, team);
                for wy in [40, 48] {
                    rr(3, wy, 3, 5, hex(0x1e1a28));
                }
            }
            2 => {
                rr(0, 34, 54, 34, hex(0x8a7e66));
                rr(0, 34, 54, 3, hex(0xb4a88c));
                for i in 0..5 {
                    rr(2 + i * 10, 38, 6, 4, hex(0xb4a88c));
                }
                rr(20, 18, 14, 16, hex(0x2a2118));
                rr(40, 46, 12, 12, hex(0x6a5e48));
                rr(46, 52, 12, 3, hex(0x2a2d34));
                rr(8, h + 4, 2, 18, hex(0x3a2a18));
                rr(10, h + 4, 14, 9, team);
            }
            3 => {
                rr(0, 30, 54, 30, hex(0x7d8478));
                rr(0, 30, 54, 3, hex(0xa6ad9c));
                rr(4, 44, 22, 14, hex(0x6c7368));
                rr(8, 40, 14, 4, hex(0x1e2420));
                rr(30, 44, 22, 14, hex(0x6c7368));
                rr(34, 40, 14, 4, hex(0x1e2420));
                rr(38, h + 14, 2, 26, hex(0xbfc4b4));
                rr(32, h + 8, 14, 2, hex(0xbfc4b4));
                rr(8, h + 4, 2, 16, hex(0x3a3a30));
                rr(10, h + 4, 12, 8, team);
                rr(22, 14, 12, 12, hex(0x1a1d18));
            }
            _ => {
                rr(6, 16, 40, 16, hex(0x26304a));
                rr(6, 16, 40, 2, CYAN);
                rr(14, 40, 24, 28, hex(0x324260));
                rr(14, 40, 3, 28, hex(0x5a74a6));
                rr(20, 56, 12, 20, hex(0x3a4e78));
                rr(24, 58, 4, 18, CYAN);
                rr(24, h + 12, 4, 20, hex(0x7a8cc0));
                rr(20, h + 18, 12, 4, team);
                rr(-4, 4, 60, 4, hex(0x12182a));
            }
        }
        if b >= 3 {
            rr(2, 8, 4, 3, team);
        }
        rr(0, 4, 3, 3, team_d);
        if hp_frac < 0.5 {
            for i in 0..7 {
                rr(6 + i * 7, 8 + (i * 9) % 30, 2, 5 + (i % 3), hex(0x1a1612));
            }
        }
    }

    pub fn turret(&mut self, data: &Data, o: usize, slot: usize, t_era: usize, base_era: usize, recoil: f64) {
        let b = data.eras[t_era].bucket;
        let (x, y) = turret_pos(o, slot, base_era);
        let d = if o == 0 { 1 } else { -1 };
        let metal = [hex(0xa07040), hex(0x7e8aa0), hex(0x4f535c), hex(0x59624a), hex(0x26304a)][b];
        let hi = [hex(0xd9b26a), hex(0xd3d9e3), hex(0x9aa0aa), hex(0xa6ad8c), CYAN][b];
        self.rect(x - 4, y - 5, 9, 6, metal);
        self.rect(x - 4, y - 5, 9, 1, hi);
        self.rect(x - 1, y - 8, 3, 3, TEAM[o]);
        let r = if recoil > 0.0 { -2 } else { 0 };
        let back = |n: i32| if d < 0 { n } else { 0 };
        match b {
            0 => {
                self.line(x, y - 8, x + d * (8 + r), y - 10, hex(0x4a3820));
                self.px(x + d * (8 + r), y - 10, hex(0xd9d9d9));
            }
            1 => self.rect(x + d * 2, y - 8, 7 + r, 2, hex(0x4a3820)),
            2 => self.rect(x + d * 2 - back(8), y - 8, 8 + r, 3, hex(0x2a2d34)),
            3 => {
                self.rect(x + d * 2 - back(9), y - 8, 9 + r, 2, hex(0x2a2d34));
                self.px(x + d * 11, y - 8, hex(0xffe28a));
            }
            _ => {
                self.rect(x + d * 2 - back(9), y - 8, 9 + r, 2, hi);
                self.px(x + d * 11, y - 8, WHITE);
            }
        }
    }

    pub fn unit(&mut self, data: &Data, u: &Unit, cx: i32, dy: i32, dir: i32, anim: &Anim) {
        let b = data.eras[u.era].bucket;
        let p = &PAL_BUCKET[b];
        let team = TEAM[u.o];
        let fl = anim.flash;
        let gy = GROUND_Y + dy;
        let atk = anim.atk;
        let step = anim.walk & 7;
        let lift = if step < 4 { step } else { 7 - step };
        let tint = |c: u32| if fl > 0.0 { mix(c, WHITE, fl) } else { c };

        macro_rules! r {
            ($lx:expr, $ly:expr, $w:expr, $h:expr, $c:expr) => {{
                let (lx, w): (i32, i32) = ($lx, $w);
                let x0 = if dir > 0 { cx + lx } else { cx - lx - w + 1 };
                self.rect(x0, gy - ($ly), w, $h, tint($c));
            }};
        }

        for i in -5..=5 {
            self.blend(cx + i, gy + 1, INK, 0.35);
        }

        if u.role == 2 && b == 3 {
            r!(-11, 4, 24, 4, p.dark);
            for i in 0..6 {
                self.px(cx + (-9 + i * 4) * dir, gy - 2 - ((step + i) & 1), tint(p.metal_d));
            }
            r!(-10, 9, 22, 5, p.metal);
            r!(-10, 9, 22, 1, scale(p.metal, 1.2));
            r!(-4, 13, 11, 4, p.metal_d);
            let rec = if atk > 0.5 { -2 } else { 0 };
            r!(6 + rec, 12, 14, 2, p.dark);
            r!(-9, 8, 3, 2, team);
            return;
        }
        if u.role == 2 && b == 4 {
            let bob = if lift > 1 { 1 } else { 0 };
            r!(-7, 9 - bob, 3, 9, p.metal_d);
            r!(4, 9 - (1 - bob), 3, 9, p.metal_d);
            r!(-9, 2, 7, 2, p.dark);
            r!(2, 2, 7, 2, p.dark);
            r!(-8, 20 - bob, 17, 10, p.metal);
            r!(-8, 20 - bob, 17, 2, scale(p.metal, 1.15));
            r!(-3, 25 - bob, 9, 5, hex(0x101830));
            r!(-2, 24 - bob, 7, 2, CYAN);
            r!(6, 17 - bob, 14, 3, p.metal_d);
            r!(20, 17 - bob, 2, 3, if atk > 0.5 { WHITE } else { CYAN });
            r!(-8, 15 - bob, 4, 2, team);
            return;
        }

        let heavy = u.role == 2;
        let (bw, bh) = if heavy { (8, 8) } else { (6, 6) };
        let leg_h = 5;
        let hy = leg_h + bh;
        let (l1, l2) = if lift > 1 { (1, 0) } else { (0, 1) };
        r!(-3, leg_h - l1, 2, leg_h - l1, p.dark);
        r!(1, leg_h - l2, 2, leg_h - l2, p.dark);

        let body = match b {
            0 | 1 => if heavy { p.metal } else { team },
            2 => if u.role == 1 { p.cloth } else { team },
            _ => p.cloth,
        };
        let bl = -bw / 2;
        r!(bl, hy, bw, bh, body);
        if b <= 1 && !heavy {
            r!(bl + 1, hy - 2, bw - 2, 2, p.metal);
        }
        if b >= 3 {
            r!(bl, hy - 1, bw, 1, team);
        }
        if b == 4 {
            r!(-1, hy - 2, 3, 2, CYAN);
        }
        if heavy && b < 3 {
            r!(bl, hy, bw, 1, scale(p.metal, 1.2));
        }

        r!(-2, hy + 5, 5, 5, SKIN);
        match b {
            0 => {
                r!(-3, hy + 6, 7, 3, p.metal);
                if heavy {
                    r!(-1, hy + 10, 3, 3, team);
                }
            }
            1 => {
                r!(-3, hy + 6, 7, 6, p.metal);
                r!(1, hy + 3, 2, 1, hex(0x1e1a28));
                if heavy {
                    r!(-1, hy + 12, 2, 3, team);
                }
            }
            2 => {
                r!(-3, hy + 6, 7, 2, if u.role == 1 { p.dark } else { p.metal });
                if u.role == 1 {
                    r!(-4, hy + 4, 9, 1, p.dark);
                }
            }
            3 => r!(-3, hy + 6, 7, 3, p.metal),
            _ => {
                r!(-3, hy + 6, 7, 4, p.metal);
                r!(0, hy + 4, 4, 2, CYAN);
            }
        }

        let reach = (atk * 4.0).round() as i32;
        let (steel, bright, flare) = (hex(0xd3d9e3), hex(0xe8ecf2), hex(0xffe28a));
        if u.role == 0 {
            match b {
                0 => {
                    r!(3, hy - 1, 3, 7, team);
                    r!(3, hy - 1, 1, 7, p.metal_d);
                    r!(6 + reach, hy - 2, 7, 1, bright);
                }
                1 => {
                    r!(3, hy - 1, 3, 7, team);
                    r!(6 + reach, hy - 1, 12, 1, steel);
                    r!(5 + reach, hy - 2, 2, 3, p.metal_d);
                }
                2 => {
                    r!(2 + reach, hy - 1, 15, 1, p.dark);
                    r!(17 + reach, hy - 1, 3, 1, steel);
                }
                3 => {
                    r!(2 + reach / 2, hy - 1, 10, 2, p.dark);
                    r!(11, hy - 1, 2, 1, p.metal_d);
                }
                _ => {
                    r!(2 + reach / 2, hy - 1, 10, 2, p.metal_d);
                    r!(10, hy - 1, 5, 1, CYAN);
                }
            }
        } else if u.role == 1 {
            match b {
                0 | 1 => {
                    for i in 0..9 {
                        r!(5 + (if i < 4 { i } else { 8 - i }) / 2, hy + 3 - i, 1, 1, p.dark);
                    }
                    r!(5, hy - 1, 1, 9, hex(0xece6d2));
                    if atk < 0.5 {
                        r!(6, hy + 3, 5, 1, hex(0x9a6a3a));
                    }
                }
                2 => {
                    r!(2, hy + 1, 12, 1, p.dark);
                    r!(2, hy, 5, 2, hex(0x6a4a2a));
                    if atk > 0.5 {
                        r!(14, hy + 2, 3, 3, flare);
                    }
                }
                3 => {
                    r!(2, hy + 1, 11, 2, p.dark);
                    r!(11, hy + 2, 2, 1, p.metal_d);
                    if atk > 0.5 {
                        r!(13, hy + 1, 2, 3, flare);
                    }
                }
                _ => {
                    r!(2, hy + 1, 11, 2, p.metal_d);
                    r!(11, hy + 1, 3, 2, CYAN);
                    if atk > 0.5 {
                        r!(14, hy + 1, 2, 2, WHITE);
                    }
                }
            }
        } else {
            match b {
                0 => {
                    r!(4, hy + 2, 4, 10, team);
                    r!(4, hy + 2, 1, 10, p.metal_d);
                    r!(8 + reach, hy - 1, 14, 1, p.dark);
                    r!(22 + reach, hy - 1, 3, 1, bright);
                }
                1 => {
                    r!(4, hy + 1, 3, 9, team);
                    r!(7 + reach, hy + 2, 3, 1, p.metal_d);
                    r!(8 + reach, hy - 2, 2, 14, bright);
                }
                _ => {
                    r!(3, hy, 3, 5, p.metal_d);
                    r!(6 + reach, hy - 1, 14, 1, steel);
                    r!(17, hy + 7, 4, 3, team);
                }
            }
        }
    }

    pub fn hp_bar(&mut self, cx: i32, top_y: i32, frac: f64) {
        let (w, x) = (12, cx - 6);
        self.rect(x - 1, top_y - 1, w + 2, 3, INK);
        self.rect(x, top_y, w, 1, hex(0x3a1a1a));
        let c = if frac > 0.5 {
            hex(0x59e07a)
        } else if frac > 0.25 {
            hex(0xf4c542)
        } else {
            hex(0xff5a4a)
        };
        self.rect(x, top_y, ((w as f64 * frac).round() as i32).max(1), 1, c);
    }
}

fn paint_background(data: &Data, buckets: &[Bucket], era: usize) -> Vec<u32> {
    let e = &data.eras[era];
    let b = &buckets[e.bucket];
    let t = era as f64 / (data.eras.len() - 1) as f64;
    let (sky0, sky1) = (parse(&b.sky[0]), parse(&b.sky[1]));
    let top = mix(sky0, hex(0x0b0720), if t > 0.84 { (t - 0.84) * 2.2 } else { 0.0 });
    let bot = sky1;
    let (ground, ground_dark) = (parse(&b.ground[0]), parse(&b.ground[1]));
    let mut bg = vec![0u32; (W * H) as usize];
    let night = e.bucket == 4;
    let horizon = 232;
    for y in 0..H {
        for x in 0..W {
            bg[(y * W + x) as usize] = if y < horizon {
                let k = y as f64 / horizon as f64;
                let bayer = (((x & 3) * 4 + (y & 3)) as f64) / 16.0;
                mix(top, bot, clamp(((k * 14.0 + bayer).floor()) / 14.0))
            } else {
                ground
            };
        }
    }
    let mut put = |x: i32, y: i32, c: u32| {
        if x >= 0 && x < W && y >= 0 && y < H {
            bg[(y * W + x) as usize] = c;
        }
    };
    let n01 = |x: f64, s: f64| noise(x, s) * 0.5 + 0.5;

    if night {
        for i in 0..90 {
            let x = (n01(i as f64, 7.0) * W as f64) as i32;
            let y = (n01(i as f64, 19.0) * 150.0) as i32;
            put(x, y, mix(hex(0xbfb8ff), WHITE, (i % 3) as f64 / 3.0));
        }
        let (px0, py0) = (500, 70);
        for y in -26i32..=26 {
            for x in -26i32..=26 {
                if x * x + y * y <= 26 * 26 {
                    put(px0 + x, py0 + y, mix(hex(0x6a4fc8), hex(0xf0a8ff), clamp((x + y + 40) as f64 / 80.0)));
                }
            }
        }
    } else {
        for i in 0..7 {
            let cx = (n01(i as f64, 3.0) * W as f64) as i32;
            let cy = 30 + (n01(i as f64, 5.0) * 70.0) as i32;
            let w = 30 + (i % 3) * 14;
            for dy in -3i32..=3 {
                for dx in -w..=w {
                    if (dx * dx) as f64 / (w * w) as f64 + (dy * dy) as f64 / 12.0 <= 1.0 {
                        put(cx + dx, cy + dy, mix(bot, WHITE, 0.55));
                    }
                }
            }
        }
        let (sx, sy) = (120, 64);
        let sun = if e.bucket == 3 { hex(0xf4f0d8) } else { hex(0xfff2b0) };
        for y in -14i32..=14 {
            for x in -14i32..=14 {
                if x * x + y * y <= 196 {
                    put(sx + x, sy + y, sun);
                }
            }
        }
    }

    let (far, near) = (mix(bot, top, 0.35), mix(ground, sky0, 0.4));
    let er = era as f64;
    for x in 0..W {
        let fx = x as f64;
        let h1 = 40.0 + noise(fx * 0.012, er * 0.37 + 1.0) * 34.0 + noise(fx * 0.05, 4.0) * 8.0;
        for y in (horizon as f64 - h1) as i32..horizon {
            put(x, y, far);
        }
        let h2 = 18.0 + noise(fx * 0.02, er * 0.21 + 9.0) * 16.0 + noise(fx * 0.09, 2.0) * 4.0;
        for y in (horizon as f64 - h2) as i32..horizon {
            put(x, y, near);
        }
    }

    for y in horizon..H {
        let depth = (y - horizon) as f64 / (H - horizon) as f64;
        for x in 0..W {
            let n = noise(x as f64 * 0.35, y as f64 * 0.9 + er) * 0.5 + 0.5;
            let mut c = mix(ground, ground_dark, depth * 0.8);
            if n > 0.78 {
                c = scale(c, 1.12);
            } else if n < 0.2 {
                c = scale(c, 0.88);
            }
            put(x, y, c);
        }
    }

    let road = mix(ground, hex(0xd8c9a0), if e.bucket == 4 { 0.0 } else { 0.22 });
    for y in 292..318 {
        for x in 0..W {
            let c = if y == 292 || y == 317 {
                scale(road, 0.7)
            } else {
                mix(road, ground_dark, 0.15 + if (x + y) % 7 == 0 { 0.12 } else { 0.0 })
            };
            put(x, y, c);
        }
    }
    if e.bucket == 4 {
        for x in (0..W).step_by(16) {
            put(x, 305, CYAN);
        }
    }
    bg
}
