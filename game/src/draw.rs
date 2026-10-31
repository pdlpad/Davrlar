use crate::color::{clamp, hex, mix, noise, parse, scale, WHITE};
use crate::eras::{style, Crown, Helm, Skyline};
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

pub struct Anim {
    pub walk: i32,
    pub atk: f64,
    pub flash: f64,
}

pub struct Frame {
    pub color: Vec<u32>,
    bg: HashMap<usize, Vec<u32>>,
}

const ROOF: [i32; 5] = [34, 54, 34, 30, 56];
const FLAG: [i32; 5] = [52, 56, 48, 42, 66];

pub fn unit_height(u: &Unit) -> i32 {
    let st = style(u.era);
    match (u.role, u.era) {
        (2, 19..=20) => 24,
        (2, 21..) => 38,
        (2, _) if st.mount => 38,
        (2, _) => 30,
        _ if st.helm == crate::eras::Helm::Shako => 30,
        _ => 24,
    }
}

pub fn turret_x(o: usize, slot: usize) -> i32 {
    let lx = 10 + slot as i32 * 14;
    if o == 0 { lx } else { W - 1 - lx }
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
        let h = FLAG[b];
        let st = style(era);
        let mut rr = |lx: i32, ly: i32, ww: i32, hh: i32, c: u32| self.base_rect(o, flash, lx, ly, ww, hh, c);
        let (team, team_d) = (TEAM[o], TEAM_DARK[o]);
        let stone = [st.stone, scale(st.stone, 0.76), scale(st.stone, 0.54)];
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
                rr(0, 34, 54, 34, stone[1]);
                rr(0, 34, 54, 3, stone[0]);
                for i in 0..5 {
                    rr(2 + i * 10, 38, 6, 4, stone[0]);
                }
                rr(20, 18, 14, 16, hex(0x2a2118));
                rr(40, 46, 12, 12, hex(0x6a5e48));
                rr(46, 52, 12, 3, hex(0x2a2d34));
                rr(8, h + 4, 2, 18, hex(0x3a2a18));
                rr(10, h + 4, 14, 9, team);
            }
            3 => {
                rr(0, 30, 54, 30, stone[1]);
                rr(0, 30, 54, 3, stone[0]);
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
                rr(6, 16, 40, 16, stone[2]);
                rr(6, 16, 40, 2, st.accent);
                rr(14, 40, 24, 28, stone[1]);
                rr(14, 40, 3, 28, stone[0]);
                rr(20, 56, 12, 20, stone[1]);
                rr(24, 58, 4, 18, st.accent);
                rr(24, h + 12, 4, 20, hex(0x7a8cc0));
                rr(20, h + 18, 12, 4, team);
                rr(-4, 4, 60, 4, hex(0x12182a));
            }
        }
        let t = ROOF[b];
        match st.crown {
            Crown::Eagle => {
                rr(18, t + 14, 2, 14, st.dark);
                rr(14, t + 14, 10, 3, st.metal);
                rr(17, t + 18, 4, 4, st.accent);
            }
            Crown::Palisade => {
                for i in 0..6 {
                    let up = (i & 1) * 3;
                    rr(4 + i * 8, t + 8 + up, 5, 8 + up, st.dark);
                }
            }
            Crown::Dome => {
                for (i, w) in [6, 12, 16, 20, 22].into_iter().enumerate() {
                    rr(26 - w / 2, t + 15 - 3 * i as i32, w, 3, st.accent);
                }
                rr(25, t + 19, 2, 4, st.metal);
            }
            Crown::Minaret => {
                rr(40, t + 30, 8, 30, stone[0]);
                rr(38, t + 20, 12, 2, st.accent);
                rr(42, t + 34, 4, 4, st.accent);
            }
            Crown::Cross => {
                rr(25, t + 16, 2, 16, st.metal);
                rr(21, t + 12, 10, 2, st.metal);
            }
            Crown::Yurt => {
                for (i, w) in [10, 22, 30, 34].into_iter().enumerate() {
                    rr(26 - w / 2, t + 12 - 3 * i as i32, w, 3, st.cloth);
                }
                rr(25, t + 17, 2, 5, st.accent);
            }
            Crown::Bell => {
                rr(18, t + 20, 16, 20, stone[1]);
                rr(16, t + 22, 20, 2, stone[0]);
                rr(24, t + 16, 4, 8, st.accent);
            }
            Crown::Stack => {
                rr(6, t + 30, 6, 30, st.dark);
                rr(16, t + 24, 5, 24, st.dark);
                rr(5, t + 30, 8, 2, st.accent);
            }
            Crown::Radar => {
                rr(28, t + 14, 2, 14, st.metal);
                rr(20, t + 18, 18, 2, st.metal);
                rr(20, t + 22, 3, 4, st.metal);
                rr(35, t + 22, 3, 4, st.metal);
            }
            Crown::Antenna => {
                rr(12, t + 16, 2, 26, st.metal);
                rr(11, t + 18, 4, 2, st.accent);
                rr(38, t + 10, 2, 22, st.metal);
                rr(37, t + 12, 4, 2, st.accent);
            }
            Crown::Orb => {
                for (i, w) in [6, 12, 14, 12, 6].into_iter().enumerate() {
                    rr(14 - w / 2, t + 22 - 2 * i as i32, w, 2, st.accent);
                }
                rr(13, t + 12, 2, 16, st.metal);
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
        let x = turret_x(o, slot);
        let y = GROUND_Y + 4 - ROOF[data.eras[base_era].bucket] - 2;
        let d = if o == 0 { 1 } else { -1 };
        let st = style(t_era);
        let (metal, hi) = (st.metal_d, if b == 4 { st.accent } else { st.metal });
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
        let p = style(u.era);
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
            if u.era == 19 {
                r!(-11, 9, 2, 4, p.metal_d);
                r!(12, 9, 2, 4, p.metal_d);
                r!(-2, 16, 6, 2, p.metal);
            } else {
                r!(-1, 15, 7, 1, p.accent);
                r!(-6, 16, 1, 5, p.dark);
            }
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
            r!(-3, 25 - bob, 9, 5, p.dark);
            r!(-2, 24 - bob, 7, 2, p.accent);
            r!(6, 17 - bob, 14, 3, p.metal_d);
            r!(20, 17 - bob, 2, 3, if atk > 0.5 { WHITE } else { p.accent });
            if u.era >= 22 {
                r!(-10, 23 - bob, 4, 5, p.metal_d);
                r!(7, 23 - bob, 4, 5, p.metal_d);
            }
            if u.era >= 23 {
                r!(-7, 33 - bob, 2, 4, p.accent);
                r!(-11, 26 - bob, 2, 2, p.accent);
            }
            if u.era == 24 {
                r!(-1, 34 - bob, 3, 5, p.metal);
                r!(-13, 20 - bob, 4, 10, p.cloth);
            }
            r!(-8, 15 - bob, 4, 2, team);
            return;
        }

        let heavy = u.role == 2;
        let (bw, bh) = if heavy { (8, 8) } else { (6, 6) };
        let leg_h = 5;
        let ride = heavy && p.mount;
        let hy = leg_h + bh + if ride { 7 } else { 0 };

        let (l1, l2) = if lift > 1 { (1, 0) } else { (0, 1) };
        if ride {
            let horse = hex(0x6a4a2a);
            r!(-8, 5 - l1, 2, 5 - l1, p.dark);
            r!(-4, 5 - l2, 2, 5 - l2, p.dark);
            r!(4, 5 - l2, 2, 5 - l2, p.dark);
            r!(8, 5 - l1, 2, 5 - l1, p.dark);
            r!(-9, 10, 19, 5, horse);
            r!(-9, 10, 12, 3, team);
            r!(9, 13, 3, 4, horse);
            r!(10, 15, 6, 3, horse);
            r!(-11, 9, 2, 5, p.dark);
        } else {
            r!(-3, leg_h - l1, 2, leg_h - l1, p.dark);
            r!(1, leg_h - l2, 2, leg_h - l2, p.dark);
        }

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
            r!(-1, hy - 2, 3, 2, p.accent);
        }
        if heavy && b < 3 {
            r!(bl, hy, bw, 1, scale(p.metal, 1.2));
            r!(bl, hy - 2, bw, 1, p.accent);
        }
        if b == 2 && u.role != 1 {
            r!(bl + 2, hy - 1, 2, bh - 1, p.accent);
        }

        r!(-2, hy + 5, 5, 5, SKIN);
        match p.helm {
            Helm::Galea => {
                r!(-3, hy + 6, 7, 3, p.metal);
                r!(-2, hy + 9, 5, 2, p.accent);
                r!(-3, hy + 8, 1, 3, p.accent);
            }
            Helm::Ridge => {
                r!(-3, hy + 6, 7, 3, p.metal);
                r!(-1, hy + 7, 3, 1, p.metal_d);
                r!(-3, hy + 3, 2, 3, p.metal_d);
            }
            Helm::Hood => {
                r!(-3, hy + 6, 7, 4, p.cloth);
                r!(-3, hy + 2, 3, 3, p.cloth);
            }
            Helm::Spire => {
                r!(-3, hy + 6, 7, 2, p.metal);
                r!(-1, hy + 8, 3, 2, p.metal);
                r!(0, hy + 10, 1, 2, p.accent);
            }
            Helm::Turban => {
                r!(-3, hy + 8, 7, 3, p.cloth);
                r!(-2, hy + 9, 5, 1, p.accent);
                r!(-1, hy + 10, 3, 1, p.metal);
            }
            Helm::Nasal => {
                r!(-3, hy + 6, 7, 2, p.metal);
                r!(-2, hy + 8, 5, 1, p.metal);
                r!(2, hy + 4, 1, 3, p.metal_d);
            }
            Helm::Spectacle => {
                r!(-3, hy + 6, 7, 3, p.metal);
                r!(1, hy + 3, 3, 1, p.metal_d);
                r!(0, hy + 1, 4, 2, p.accent);
            }
            Helm::Great => {
                r!(-3, hy + 6, 7, 6, p.metal);
                r!(1, hy + 3, 3, 1, p.dark);
                r!(0, hy + 5, 1, 4, p.accent);
            }
            Helm::Kettle => {
                r!(-4, hy + 6, 9, 1, p.metal);
                r!(-2, hy + 8, 5, 2, p.metal);
            }
            Helm::Fur => {
                r!(-3, hy + 7, 7, 3, p.cloth);
                r!(-3, hy + 4, 7, 1, p.dark);
                r!(0, hy + 9, 1, 3, p.metal);
            }
            Helm::Bascinet => {
                r!(-3, hy + 6, 7, 5, p.metal);
                r!(-1, hy + 7, 3, 1, p.metal);
                r!(2, hy + 3, 3, 2, p.metal_d);
            }
            Helm::Morion => {
                r!(-5, hy + 6, 11, 1, p.metal);
                r!(-2, hy + 8, 5, 2, p.metal);
                r!(-1, hy + 9, 3, 1, p.metal_d);
            }
            Helm::Feather => {
                r!(-4, hy + 6, 9, 1, p.dark);
                r!(-2, hy + 8, 5, 2, p.dark);
                r!(-3, hy + 10, 2, 4, p.accent);
            }
            Helm::Tricorn => {
                r!(-5, hy + 7, 11, 2, p.dark);
                r!(-2, hy + 8, 5, 1, p.dark);
                r!(-5, hy + 7, 1, 1, p.accent);
                r!(5, hy + 7, 1, 1, p.accent);
            }
            Helm::Shako => {
                r!(-2, hy + 10, 5, 5, p.dark);
                r!(2, hy + 6, 3, 1, p.dark);
                r!(0, hy + 12, 1, 3, p.accent);
            }
            Helm::Brodie => {
                r!(-4, hy + 6, 9, 1, p.metal);
                r!(-2, hy + 8, 5, 2, p.metal);
                r!(-2, hy + 7, 5, 1, p.metal_d);
            }
            Helm::Modern => {
                r!(-3, hy + 7, 7, 3, p.metal);
                r!(-3, hy + 4, 2, 3, p.metal_d);
                r!(1, hy + 4, 3, 1, p.accent);
            }
            Helm::Robot => {
                r!(-3, hy + 6, 7, 6, p.metal);
                r!(0, hy + 4, 4, 2, p.accent);
                r!(-2, hy + 8, 1, 3, p.metal_d);
            }
            Helm::Visor => {
                r!(-3, hy + 6, 7, 5, p.metal);
                r!(0, hy + 4, 4, 2, p.accent);
            }
            Helm::Dome => {
                r!(-3, hy + 6, 7, 5, p.metal_d);
                r!(-2, hy + 7, 5, 3, p.accent);
                r!(-1, hy + 8, 3, 1, p.accent);
            }
            Helm::Space => {
                r!(-3, hy + 7, 7, 7, p.metal);
                r!(-1, hy + 5, 5, 3, p.accent);
                r!(-3, hy + 9, 1, 3, p.metal_d);
            }
        }

        let reach = (atk * 4.0).round() as i32;
        let (steel, bright, flare) = (hex(0xd3d9e3), hex(0xe8ecf2), hex(0xffe28a));
        if u.role == 0 && p.pole {
            r!(-1 + reach, hy - 1, 24, 1, p.dark);
            r!(23 + reach, hy - 2, 2, 3, steel);
            r!(25 + reach, hy - 1, 2, 1, bright);
        } else if u.role == 0 {
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
                    r!(10, hy - 1, 5, 1, p.accent);
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
                    r!(11, hy + 1, 3, 2, p.accent);
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
                    r!(16, hy + 2, 6, 3, team);
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
    let st = style(era);
    let t = era as f64 / (data.eras.len() - 1) as f64;
    let (sky0, sky1) = (mix(parse(&b.sky[0]), st.sky, 0.75), mix(parse(&b.sky[1]), st.sky, 0.3));
    let top = mix(sky0, hex(0x0b0720), if t > 0.84 { (t - 0.84) * 2.2 } else { 0.0 });
    let bot = sky1;
    let ground = mix(parse(&b.ground[0]), st.ground, 0.8);
    let ground_dark = mix(parse(&b.ground[1]), scale(st.ground, 0.6), 0.8);
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
            let cx = (n01(i as f64, 3.0 + era as f64) * W as f64) as i32;
            let cy = 30 + (n01(i as f64, 5.0 + era as f64) * 70.0) as i32;
            let w = 30 + (i % 3) * 14;
            for dy in -3i32..=3 {
                for dx in -w..=w {
                    if (dx * dx) as f64 / (w * w) as f64 + (dy * dy) as f64 / 12.0 <= 1.0 {
                        put(cx + dx, cy + dy, mix(bot, WHITE, 0.55));
                    }
                }
            }
        }
        let (sx, sy) = (70 + era as i32 * 23, 40 + (n01(era as f64 * 3.0, 11.0) * 60.0) as i32);
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

    let edge = mix(near, st.dark, 0.45);
    let lit = st.accent;
    skyline(&mut put, st.skyline, horizon, edge, lit, far, era);

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

fn skyline(put: &mut impl FnMut(i32, i32, u32), kind: Skyline, base: i32, c: u32, lit: u32, far: u32, era: usize) {
    let jitter = |i: i32, n: i32| ((noise(i as f64, 40.0 + era as f64) * 0.5 + 0.5) * n as f64) as i32;
    let mut block = |x: i32, y: i32, w: i32, h: i32, col: u32| {
        for j in y..y + h {
            for i in x..x + w {
                put(i, j, col);
            }
        }
    };
    let n = 7;
    for i in 0..n {
        let x = 24 + i * 92 + jitter(i, 40);
        let s = 8 + jitter(i + 50, 8);
        match kind {
            Skyline::Temples => {
                block(x, base - 26, 44, 4, c);
                for k in 0..5 {
                    block(x + 2 + k * 9, base - 22, 4, 22, c);
                }
                for k in 0..8 {
                    block(x + k * 3, base - 27 - k.min(7 - k), 44 - k * 6, 1, c);
                }
            }
            Skyline::Aqueduct => {
                block(x - 46, base - 30, 92, 6, c);
                for k in 0..4 {
                    block(x - 44 + k * 23, base - 24, 6, 24, c);
                }
            }
            Skyline::Ruins => {
                for k in 0..4 {
                    block(x + k * 10, base - 10 - jitter(i * 5 + k, 24), 5, 40, c);
                }
                block(x - 4, base - 6, 44, 6, c);
            }
            Skyline::Pines => {
                for k in 0..3 {
                    let px = x + k * 18;
                    let h = 24 + jitter(i * 3 + k, 16);
                    for j in 0..h {
                        block(px - j / 3, base - h + j, j / 3 * 2 + 1, 1, c);
                    }
                }
            }
            Skyline::Domes => {
                block(x, base - 18, 40, 18, c);
                for j in 0..14 {
                    let w = 28 - (j * j) / 8;
                    block(x + 20 - w / 2, base - 18 - j, w, 1, c);
                }
                block(x + 19, base - 38, 2, 6, lit);
            }
            Skyline::Minarets => {
                block(x, base - 16, 34, 16, c);
                for j in 0..10 {
                    let w = 20 - (j * j) / 5;
                    block(x + 17 - w / 2, base - 16 - j, w, 1, c);
                }
                block(x + 42, base - 44, 6, 44, c);
                block(x + 40, base - 34, 10, 2, c);
                block(x + 44, base - 50, 2, 6, lit);
            }
            Skyline::Castles => {
                block(x, base - 18, 56, 18, c);
                block(x - 6, base - 30, 14, 30, c);
                block(x + 48, base - 30, 14, 30, c);
                for k in 0..3 {
                    block(x - 6 + k * 5, base - 34, 3, 4, c);
                    block(x + 48 + k * 5, base - 34, 3, 4, c);
                }
            }
            Skyline::Cathedrals => {
                block(x, base - 26, 40, 26, c);
                for sx in [x + 2, x + 30] {
                    block(sx, base - 44, 8, 18, c);
                    for j in 0..14 {
                        block(sx + j / 3, base - 58 + j, 8 - (j / 3) * 2, 1, c);
                    }
                }
                block(x + 18, base - 18, 5, 8, lit);
            }
            Skyline::Yurts => {
                for j in 0..10 {
                    let w = 30 - (j * j) / 4;
                    block(x + 15 - w.max(2) / 2, base - j, w.max(2), 1, c);
                }
                block(x + 40, base - 26, 1, 26, c);
                block(x + 41, base - 26, 9, 5, lit);
            }
            Skyline::Windmills => {
                block(x + 6, base - 26, 12, 26, c);
                for k in 0..14 {
                    block(x + 12 - k, base - 28 - k, 2, 2, c);
                    block(x + 12 + k, base - 28 - k, 2, 2, c);
                    block(x + 12 - k, base - 28 + k, 2, 2, c);
                    block(x + 12 + k, base - 28 + k, 2, 2, c);
                }
            }
            Skyline::Town => {
                for k in 0..4 {
                    let hx = x + k * 18;
                    let h = 14 + jitter(i * 7 + k, 12);
                    block(hx, base - h, 16, h, c);
                    for j in 0..8 {
                        block(hx + j, base - h - 8 + j, 16 - j * 2, 1, c);
                    }
                    block(hx + 6, base - h + 4, 3, 3, lit);
                }
            }
            Skyline::Factories => {
                block(x, base - 20, 60, 20, c);
                for k in 0..3 {
                    block(x + 6 + k * 18, base - 46 - k * 3, 7, 46, c);
                    for j in 0..12 {
                        block(x + 8 + k * 18 + j, base - 52 - k * 3 - j * 2, 5, 3, far);
                    }
                }
            }
            Skyline::City => {
                for k in 0..4 {
                    let h = 34 + jitter(i * 9 + k, 40);
                    block(x + k * 20, base - h, 18, h, c);
                    for w in 0..(h / 9) {
                        block(x + k * 20 + 3, base - h + 4 + w * 9, 3, 3, lit);
                        block(x + k * 20 + 11, base - h + 4 + w * 9, 3, 3, lit);
                    }
                }
            }
            Skyline::Towers => {
                for k in 0..3 {
                    let h = 70 + jitter(i * 11 + k, 60);
                    block(x + k * 26, base - h, 20, h, c);
                    block(x + k * 26 + 9, base - h - 16, 2, 16, c);
                    block(x + k * 26 + 8, base - h - 17, 4, 2, lit);
                    block(x + k * 26 + 3, base - h + 8, 14, 2, lit);
                }
            }
            Skyline::Neon => {
                for k in 0..3 {
                    let h = 80 + jitter(i * 13 + k, 70);
                    block(x + k * 28, base - h, 22, h, c);
                    block(x + k * 28, base - h, 2, h, lit);
                    for w in 0..(h / 16) {
                        block(x + k * 28 + 5, base - h + 6 + w * 16, 14, 2, lit);
                    }
                }
            }
            Skyline::Spires => {
                for k in 0..3 {
                    let h = 90 + jitter(i * 17 + k, 70);
                    for j in 0..h {
                        let w = 2 + (j * 18) / h;
                        block(x + k * 30 - w / 2, base - h + j, w, 1, c);
                    }
                    block(x + k * 30, base - h - 6, 1, 6, lit);
                }
            }
            Skyline::Orbit => {
                block(x, base - 10 - s, 50, 10 + s, c);
                block(x + 22, base - 40 - s, 6, 30, c);
                block(x + 10, base - 40 - s, 30, 2, lit);
            }
        }
    }
    if kind == Skyline::Orbit {
        for a in 0..360 {
            let r = (a as f64).to_radians();
            block(500 + (64.0 * r.cos()) as i32, 70 + (16.0 * r.sin()) as i32, 2, 1, lit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use davrlar_core::sim::Unit;

    const GAME: &str = include_str!("../../data/game.json");

    fn scene(data: &Data, buckets: &[Bucket], f: &mut Frame, era: usize) {
        f.background(data, buckets, era);
        f.base(data, 0, era, 1.0, 0.0);
        f.base(data, 1, era, 1.0, 0.0);
        f.turret(data, 0, 0, era, era, 0.0);
        for role in 0..3 {
            let u = Unit { id: 1, o: 0, role, era, hp: 1, x: 0, cd: 0 };
            f.unit(data, &u, 200 + role as i32 * 110, 0, 1, &Anim { walk: 3, atk: 0.0, flash: 0.0 });
        }
    }

    #[test]
    fn neighbouring_eras_look_different() {
        let data = Data::load(GAME).unwrap();
        let buckets = load_buckets(GAME);
        let mut f = Frame::new();
        scene(&data, &buckets, &mut f, 0);
        let mut prev = f.color.clone();
        for era in 1..25 {
            scene(&data, &buckets, &mut f, era);
            let same = f.color.iter().zip(&prev).filter(|(a, b)| a == b).count();
            assert!(same * 2 < f.color.len(), "эпоха {era}: совпало {same}");
            prev = f.color.clone();
        }
    }

    #[test]
    #[ignore]
    fn export_contact_sheets() {
        let dir = std::env::var("SHEET_DIR").unwrap();
        let data = Data::load(GAME).unwrap();
        let buckets = load_buckets(GAME);
        let mut f = Frame::new();
        let mut out = Vec::new();
        for era in 0..25 {
            scene(&data, &buckets, &mut f, era);
            out.extend(f.color.iter().flat_map(|c| c.to_le_bytes()));
        }
        std::fs::write(format!("{dir}/scenes.rgba"), &out).unwrap();
        f.color.fill(hex(0x8a8f80));
        for era in 0..25usize {
            for role in 0..3 {
                let (cx, gy) = (22 + (era % 5) as i32 * 126 + role as i32 * 42, 62 + (era / 5) as i32 * 72);
                for o in 0..2 {
                    let u = Unit { id: 1, o, role, era, hp: 1, x: 0, cd: 0 };
                    f.unit(&data, &u, cx + o as i32 * 14 - 8, gy - GROUND_Y, 1, &Anim { walk: 2, atk: 0.0, flash: 0.0 });
                }
            }
        }
        let bytes: Vec<u8> = f.color.iter().flat_map(|c| c.to_le_bytes()).collect();
        std::fs::write(format!("{dir}/units.rgba"), &bytes).unwrap();
    }
}
