use crate::draw::{turret_pos, GROUND_Y, W};
use davrlar_core::data::Data;
use davrlar_core::sim::Event;
use serde::Deserialize;
use std::collections::HashMap;

const MAX_ITEMS: usize = 170;
const FW: f64 = W as f64;
const GY: f64 = GROUND_Y as f64;

macro_rules! sheets {
    ($($id:ident $file:literal),* $(,)?) => {
        #[derive(Clone, Copy)]
        pub enum Sheet { $($id),* }
        pub const SHEETS: &[(&str, &[u8])] = &[$(($file, include_bytes!(concat!("../assets/fx/", $file, ".png")))),*];
    };
}

sheets! {
    HitSpark "hit-spark",
    BluntHit "blunt-hit",
    BulletImpact "bullet-impact",
    Arrow "arrow",
    Tracer "tracer",
    LaserBlue "laser-blue",
    LaserRed "laser-red",
    PlasmaBlue "plasma-blue",
    PlasmaRed "plasma-red",
    Muzzle "muzzle",
    PoofDust "poof-dust",
    PoofSmoke "poof-smoke",
    PoofCyber "poof-cyber",
    Coin "coin",
    LevelGold "level-gold",
    LevelFire "level-fire",
    ExplosionS "explosion-s",
    ExplosionM "explosion-m",
    ExplosionL "explosion-l",
    Nuke "nuke",
    Meteor "meteor",
    Lightning "lightning",
}

#[derive(Deserialize, Clone)]
pub struct Info {
    pub w: f32,
    pub h: f32,
    pub cols: usize,
    pub frames: usize,
    pub fps: f64,
    pub ox: f32,
    pub oy: f32,
    #[serde(rename = "loop")]
    pub looped: bool,
}

pub fn load_infos() -> Vec<Info> {
    let m: HashMap<String, Info> = serde_json::from_str(include_str!("../assets/fx/manifest.json")).expect("разбор manifest.json");
    SHEETS.iter().map(|(name, _)| m[*name].clone()).collect()
}

type Then = Box<dyn FnOnce(&mut Fx)>;

struct Proj {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    dur: f64,
    arc: f64,
    on_end: Option<Then>,
}

pub struct Item {
    pub sheet: usize,
    pub x: f64,
    pub y: f64,
    pub t: f64,
    pub flip: bool,
    pub angle: f64,
    proj: Option<Proj>,
}

pub struct Label {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub t: f64,
    pub life: f64,
    pub gold: bool,
}

pub struct Fx {
    infos: Vec<Info>,
    pub items: Vec<Item>,
    pub labels: Vec<Label>,
    pub shake: f64,
    pub fast: bool,
    pub flashes: HashMap<i64, f64>,
    pub base_flash: [f64; 2],
    pub recoil: HashMap<i64, f64>,
    last_num: HashMap<i64, f64>,
    timers: Vec<(f64, Then)>,
    clock: f64,
    rng: u32,
}

pub fn turret_key(o: usize, slot: i64) -> i64 {
    -(o as i64 * 10 + slot + 1)
}

fn poof(b: usize, smoke_default: bool) -> Sheet {
    match (b, smoke_default) {
        (4, _) => Sheet::PoofCyber,
        (3, _) | (_, true) => Sheet::PoofSmoke,
        _ => Sheet::PoofDust,
    }
}

impl Fx {
    pub fn new(infos: Vec<Info>) -> Fx {
        Fx {
            infos,
            items: Vec::new(),
            labels: Vec::new(),
            shake: 0.0,
            fast: false,
            flashes: HashMap::new(),
            base_flash: [0.0; 2],
            recoil: HashMap::new(),
            last_num: HashMap::new(),
            timers: Vec::new(),
            clock: 0.0,
            rng: 0x2f6e_2b1d,
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.labels.clear();
        self.timers.clear();
    }

    pub fn random(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng as f64 / 4294967296.0
    }

    pub fn info(&self, sheet: usize) -> &Info {
        &self.infos[sheet]
    }

    pub fn frame_of(&self, it: &Item) -> usize {
        let i = &self.infos[it.sheet];
        let f = (it.t * i.fps) as usize;
        if i.looped {
            f % i.frames
        } else {
            f.min(i.frames - 1)
        }
    }

    fn add(&mut self, sheet: Sheet, x: f64, y: f64, flip: bool) {
        if self.items.len() <= MAX_ITEMS {
            self.items.push(Item { sheet: sheet as usize, x, y, t: 0.0, flip, angle: 0.0, proj: None });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn projectile(&mut self, sheet: Sheet, x0: f64, y0: f64, x1: f64, y1: f64, dur: f64, arc: f64, angle: f64, on_end: Then) {
        if self.items.len() > MAX_ITEMS {
            on_end(self);
            return;
        }
        let proj = Some(Proj { x0, y0, x1, y1, dur, arc, on_end: Some(on_end) });
        self.items.push(Item { sheet: sheet as usize, x: x0, y: y0, t: 0.0, flip: x1 < x0, angle, proj });
    }

    fn label(&mut self, text: String, x: f64, y: f64, life: f64, gold: bool) {
        if self.labels.len() < 40 {
            self.labels.push(Label { text, x, y, t: 0.0, life, gold });
        }
    }

    fn later(&mut self, sec: f64, f: Then) {
        self.timers.push((sec, f));
    }

    pub fn update(&mut self, dt: f64) {
        self.clock += dt;
        let timers = std::mem::take(&mut self.timers);
        for (t, f) in timers {
            if t - dt <= 0.0 {
                f(self);
            } else {
                self.timers.push((t - dt, f));
            }
        }
        self.flashes.retain(|_, v| {
            *v -= dt;
            *v > 0.0
        });
        self.recoil.retain(|_, v| {
            *v -= dt;
            *v > 0.0
        });
        for f in self.base_flash.iter_mut() {
            *f = (*f - dt).max(0.0);
        }
        self.shake = (self.shake - dt * 14.0).max(0.0);
        for l in self.labels.iter_mut() {
            l.t += dt;
        }
        self.labels.retain(|l| l.t < l.life);

        let items = std::mem::take(&mut self.items);
        let mut keep = Vec::with_capacity(items.len());
        for mut it in items {
            it.t += dt;
            if let Some(p) = it.proj.as_mut() {
                let k = (it.t / p.dur).min(1.0);
                it.x = p.x0 + (p.x1 - p.x0) * k;
                it.y = p.y0 + (p.y1 - p.y0) * k - (std::f64::consts::PI * k).sin() * p.arc;
                if k >= 1.0 {
                    if let Some(f) = p.on_end.take() {
                        f(self);
                    }
                    continue;
                }
                keep.push(it);
            } else {
                let i = &self.infos[it.sheet];
                if i.looped || it.t < i.frames as f64 / i.fps {
                    keep.push(it);
                }
            }
        }
        keep.append(&mut self.items);
        self.items = keep;
    }

    pub fn handle(&mut self, data: &Data, ev: &[Event]) {
        for e in ev {
            match e {
                Event::Spawn { .. } if self.fast => {}
                Event::Spawn { era, x, .. } => {
                    let b = data.eras[*era].bucket;
                    self.add(poof(b, false), *x as f64 / 10.0, GY + 2.0, false);
                }
                Event::Hit { .. } => self.on_hit(data, e),
                Event::Death { o, role, era, x, .. } => {
                    let b = data.eras[*era].bucket;
                    let x = *x as f64 / 10.0;
                    if *role == 2 && b >= 3 {
                        self.add(Sheet::ExplosionS, x, GY + 2.0, false);
                    } else {
                        self.add(poof(b, true), x, GY + 2.0, false);
                    }
                    if *o == 1 {
                        self.add(Sheet::Coin, x, GY + 2.0, false);
                    }
                    self.shake = self.shake.max(if *role == 2 { 1.2 } else { 0.0 });
                }
                Event::Evolve { o, era } => {
                    let x = if *o == 0 { 74.0 } else { FW - 74.0 };
                    self.add(if *o == 0 { Sheet::LevelGold } else { Sheet::LevelFire }, x, GY - 30.0, false);
                    self.label(format!("AGE {}", data.eras[*era].roman), x, GY - 104.0, 2.3, true);
                    self.shake = self.shake.max(2.0);
                }
                Event::TurretBuilt { o, slot, era } => {
                    let b = data.eras[*era].bucket;
                    let (x, _) = turret_pos(*o, *slot, 0);
                    self.add(poof(b, false), x as f64, GY - 40.0, false);
                }
                Event::Special { o, era, bucket, dmg, hit } => self.on_special(data, *o, *era, *bucket, *dmg, hit),
                Event::Siege => {
                    self.shake = self.shake.max(2.2);
                    for o in 0..2 {
                        let x = if o == 0 { 34.0 } else { FW - 34.0 };
                        let dx = (self.random() - 0.5) * 30.0;
                        self.add(Sheet::ExplosionS, x + dx, GY - 20.0, false);
                        self.base_flash[o] = 0.1;
                    }
                }
                Event::Win { winner } => self.on_win(*winner),
                Event::TurretSold { .. } => {}
            }
        }
    }

    fn on_hit(&mut self, data: &Data, e: &Event) {
        let Event::Hit { tgt_unit, fid, x, dmg, o, from, role, era, slot } = *e else { return };
        let b = data.eras[era].bucket;
        let tx = x as f64 / 10.0;
        let ty = if tgt_unit.is_none() { GY - 24.0 - ((dmg * 7) % 20) as f64 } else { GY - 12.0 };
        let turret = role == -1;
        let ranged = role == 1 || turret;
        let flip = o == 1;
        match tgt_unit {
            Some(id) => {
                self.flashes.insert(id, 0.12);
            }
            None => self.base_flash[1 - o] = 0.12,
        }
        self.recoil.insert(if turret { turret_key(o, slot) } else { fid }, 0.12);
        if self.fast {
            return;
        }

        let key = tgt_unit.unwrap_or(-(o as i64) - 1000);
        let impact: Then = Box::new(move |f: &mut Fx| {
            if b >= 2 && ranged {
                f.add(Sheet::BulletImpact, tx, ty, flip);
            } else if role == 2 {
                f.add(Sheet::BluntHit, tx, ty + 4.0, flip);
            } else {
                f.add(Sheet::HitSpark, tx, ty, flip);
            }
            f.number(key, tgt_unit.is_none(), dmg, tx, ty);
        });
        if !ranged {
            impact(self);
            return;
        }
        let sx = from as f64 / 10.0 + if o == 0 { 8.0 } else { -8.0 };
        let sy = GY - if role == 2 { 20.0 } else { 16.0 };
        let dist = (tx - sx).abs();
        if b <= 1 {
            self.projectile(Sheet::Arrow, sx, sy, tx, ty, (dist / 420.0).max(0.12), 14.0, 0.0, impact);
        } else if b <= 3 {
            self.add(Sheet::Muzzle, sx, sy, flip);
            self.projectile(Sheet::Tracer, sx, sy, tx, ty, (dist / 900.0).max(0.06), 0.0, 0.0, impact);
        } else {
            let sheet = match (turret, o) {
                (true, 0) => Sheet::PlasmaBlue,
                (true, _) => Sheet::PlasmaRed,
                (false, 0) => Sheet::LaserBlue,
                (false, _) => Sheet::LaserRed,
            };
            self.projectile(sheet, sx, sy, tx, ty, (dist / 700.0).max(0.08), 0.0, 0.0, impact);
        }
    }

    fn number(&mut self, key: i64, base: bool, dmg: i64, x: f64, y: f64) {
        let gap = if base { 0.7 } else { 0.38 };
        if self.clock - self.last_num.get(&key).copied().unwrap_or(-1.0) < gap {
            return;
        }
        self.last_num.insert(key, self.clock);
        if self.last_num.len() > 400 {
            self.last_num.clear();
        }
        self.label(format!("-{}", short(dmg)), x, y - 10.0, 1.05, false);
    }

    fn on_special(&mut self, data: &Data, o: usize, era: usize, b: usize, dmg: i64, hit: &[(i64, i64)]) {
        self.shake = self.shake.max(6.0);
        let mut spots: Vec<f64> = hit.iter().map(|h| h.1 as f64 / 10.0).collect();
        if spots.is_empty() {
            spots.push(if o == 0 { FW * 0.7 } else { FW * 0.3 });
        }
        for h in hit.iter().take(14) {
            self.label(format!("-{}", short(dmg)), h.1 as f64 / 10.0, GY - 22.0, 1.05, false);
        }
        let pick = |n: usize| -> Vec<f64> {
            let mut s = spots.clone();
            s.sort_by(f64::total_cmp);
            let m = n.min(s.len());
            (0..m).map(|i| s[((i as f64 + 0.5) / m as f64 * s.len() as f64) as usize]).collect()
        };
        match b {
            0 => {
                let targets: Vec<f64> = if spots.len() < 6 {
                    spots.iter().copied().chain(spots.iter().map(|x| x - 14.0)).chain(spots.iter().map(|x| x + 14.0)).collect()
                } else {
                    spots.clone()
                };
                for (i, x) in targets.into_iter().take(18).enumerate() {
                    self.later(i as f64 * 0.03, Box::new(move |f: &mut Fx| {
                        let top = 30.0 - (i % 3) as f64 * 14.0;
                        let hit: Then = Box::new(move |f: &mut Fx| f.add(Sheet::HitSpark, x, GY - 8.0, false));
                        f.projectile(Sheet::Arrow, x - 70.0, top, x, GY - 6.0, 0.5, 0.0, 0.95, hit);
                    }));
                }
            }
            1 => self.scatter(pick(3), 0.12, Sheet::Meteor, GY + 4.0),
            2 => self.scatter(pick(5), 0.14, Sheet::ExplosionM, GY + 6.0),
            3 => self.scatter(pick(3), 0.22, Sheet::ExplosionL, GY + 6.0),
            _ if era == data.eras.len() - 1 => {
                self.add(Sheet::Nuke, pick(1)[0], GY + 6.0, false);
                self.shake = 10.0;
            }
            _ => self.scatter(pick(7), 0.09, Sheet::Lightning, GY + 4.0),
        }
    }

    fn scatter(&mut self, xs: Vec<f64>, step: f64, sheet: Sheet, y: f64) {
        for (i, x) in xs.into_iter().enumerate() {
            self.later(i as f64 * step, Box::new(move |f: &mut Fx| f.add(sheet, x, y, false)));
        }
    }

    fn on_win(&mut self, winner: i64) {
        let sides: &[usize] = match winner {
            0 => &[1],
            1 => &[0],
            _ => &[0, 1],
        };
        for &o in sides {
            let x = if o == 0 { 30.0 } else { FW - 30.0 };
            self.add(Sheet::ExplosionL, x, GY + 6.0, false);
            for (i, dx) in [-28.0, 26.0, -8.0, 16.0].into_iter().enumerate() {
                let y = GY - 6.0 - i as f64 * 4.0;
                self.later(0.15 + i as f64 * 0.19, Box::new(move |f: &mut Fx| f.add(Sheet::ExplosionM, x + dx, y, false)));
            }
        }
        self.shake = 12.0;
    }
}

pub fn short(n: i64) -> String {
    let tenth = |v: f64| {
        let s = format!("{v:.1}");
        s.strip_suffix(".0").unwrap_or(&s).to_string()
    };
    if n >= 1_000_000 {
        format!("{}M", tenth(n as f64 / 1e6))
    } else if n >= 10_000 {
        format!("{}K", (n as f64 / 1e3).round() as i64)
    } else if n >= 1_000 {
        format!("{}K", tenth(n as f64 / 1e3))
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use davrlar_core::ai::{ai_commands, Level};
    use davrlar_core::sim::{new_game, step};

    #[test]
    fn short_matches_client() {
        assert_eq!(short(950), "950");
        assert_eq!(short(1500), "1.5K");
        assert_eq!(short(2000), "2K");
        assert_eq!(short(12_400), "12K");
        assert_eq!(short(3_000_000), "3M");
    }

    #[test]
    fn every_sheet_has_manifest_entry() {
        assert_eq!(load_infos().len(), SHEETS.len());
    }

    #[test]
    fn fast_mode_drops_hit_effects_but_keeps_flashes() {
        let data = Data::load(include_str!("../../data/game.json")).unwrap();
        let mut fx = Fx::new(load_infos());
        fx.fast = true;
        let hit = Event::Hit { tgt_unit: Some(4), fid: 2, x: 1000, dmg: 9, o: 0, from: 900, role: 1, era: 0, slot: 0 };
        fx.handle(&data, &[hit, Event::Spawn { id: 1, o: 0, role: 0, era: 0, x: 560 }]);
        assert!(fx.items.is_empty() && fx.labels.is_empty());
        assert!(fx.flashes.contains_key(&4));
        fx.handle(&data, &[Event::Death { id: 1, o: 1, role: 0, era: 0, x: 560 }]);
        assert_eq!(fx.items.len(), 2);
    }

    #[test]
    fn effects_survive_a_whole_game_and_drain() {
        let data = Data::load(include_str!("../../data/game.json")).unwrap();
        let mut fx = Fx::new(load_infos());
        let mut st = new_game(&data, 9, [100, 100]);
        while st.winner < 0 && st.tick < 30 * 60 * 25 {
            let mut c = ai_commands(&st, &data, 0, Level::Hard);
            c.extend(ai_commands(&st, &data, 1, Level::Hard));
            let ev = step(&mut st, &data, &c);
            fx.handle(&data, &ev);
            fx.update(1.0 / 30.0);
            assert!(fx.items.len() <= MAX_ITEMS + 2);
        }
        assert!(st.winner >= 0);
        for _ in 0..30 * 15 {
            fx.update(1.0 / 30.0);
        }
        assert!(fx.items.is_empty() && fx.labels.is_empty());
    }
}
