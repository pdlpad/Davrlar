use crate::data::Data;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "t", rename_all = "camelCase")]
pub enum Cmd {
    Train { p: usize, role: i64 },
    Evolve { p: usize },
    BuyTurret { p: usize, slot: i64 },
    SellTurret { p: usize, slot: i64 },
    Special { p: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Spawn { id: i64, o: usize, role: usize, era: usize, x: i64 },

    Hit { tgt_unit: Option<i64>, fid: i64, x: i64, dmg: i64, o: usize, from: i64, role: i64, era: usize, slot: i64 },
    Death { id: i64, o: usize, role: usize, era: usize, x: i64 },
    Evolve { o: usize, era: usize },
    TurretBuilt { o: usize, slot: usize, era: usize },
    TurretSold { o: usize, slot: usize },
    Special { o: usize, era: usize, bucket: usize, dmg: i64, hit: Vec<(i64, i64)> },
    Siege,
    Win { winner: i64 },
}

#[derive(Debug, Clone)]
pub struct QueueItem {
    pub role: usize,
    pub era: usize,
    pub t: i64,
}
#[derive(Debug, Clone)]
pub struct Turret {
    pub era: usize,
    pub cd: i64,
}
#[derive(Debug, Clone)]
pub struct Player {
    pub inc_pct: i64,
    pub gold: i64,
    pub gold_acc: i64,
    pub xp: i64,
    pub era: usize,
    pub base_hp: i64,
    pub base_max: i64,
    pub queue: Vec<QueueItem>,
    pub turrets: Vec<Option<Turret>>,
    pub sp_cd: i64,
}
#[derive(Debug, Clone)]
pub struct Unit {
    pub id: i64,
    pub o: usize,
    pub role: usize,
    pub era: usize,
    pub hp: i64,
    pub x: i64,
    pub cd: i64,
}
#[derive(Debug, Clone)]
pub struct State {
    pub tick: i64,
    pub seed: i64,
    pub rng: u32,
    pub next_id: i64,

    pub winner: i64,
    pub players: [Player; 2],
    pub units: Vec<Unit>,
}

fn next_rng(mut s: u32) -> u32 {
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    s
}
fn rng_u32(st: &mut State) -> u32 {
    st.rng = next_rng(st.rng);
    st.rng
}

pub fn new_game(data: &Data, seed: i64, income_pct: [i64; 2]) -> State {
    let e0 = &data.eras[0];
    let mk = |o: usize| Player {
        inc_pct: income_pct[o],
        gold: data.start_gold,
        gold_acc: 0,
        xp: 0,
        era: 0,
        base_hp: e0.base_max,
        base_max: e0.base_max,
        queue: Vec::new(),
        turrets: (0..data.turret_slots).map(|_| None).collect(),
        sp_cd: 600,
    };
    let seed32 = seed as i32;
    let r = seed as u32;
    State {
        tick: 0,
        seed: seed32 as i64,
        rng: if r == 0 { 1 } else { r },
        next_id: 1,
        winner: -1,
        players: [mk(0), mk(1)],
        units: Vec::new(),
    }
}

#[inline]
fn dir_of(o: usize) -> i64 {
    if o == 0 { 1 } else { -1 }
}

fn apply_command(st: &mut State, data: &Data, c: &Cmd, ev: &mut Vec<Event>) {
    let pi = match c {
        Cmd::Train { p, .. } | Cmd::Evolve { p } | Cmd::BuyTurret { p, .. } | Cmd::SellTurret { p, .. } | Cmd::Special { p } => *p,
    };
    if pi > 1 {
        return;
    }
    let p_era = st.players[pi].era;
    let e = &data.eras[p_era];
    match c {
        Cmd::Train { role, .. } => {
            if *role < 0 || *role > 2 {
                return;
            }
            let us = &e.units[*role as usize];
            let p = &mut st.players[pi];
            if p.queue.len() >= data.queue_max || p.gold < us.cost {
                return;
            }
            p.gold -= us.cost;
            p.queue.push(QueueItem { role: *role as usize, era: p_era, t: us.train });
        }
        Cmd::Evolve { .. } => {
            let p = &mut st.players[pi];
            if p.era >= data.eras.len() - 1 || p.xp < e.xp_need {
                return;
            }
            p.xp -= e.xp_need;
            p.era += 1;
            let n = &data.eras[p.era];
            p.base_hp += n.base_max - p.base_max;
            p.base_max = n.base_max;
            ev.push(Event::Evolve { o: pi, era: p.era });
        }
        Cmd::BuyTurret { slot, .. } => {
            if *slot < 0 || *slot >= data.turret_slots as i64 {
                return;
            }
            let s = *slot as usize;
            let p = &mut st.players[pi];
            if p.turrets[s].is_some() || p.gold < e.turret.cost {
                return;
            }
            p.gold -= e.turret.cost;
            p.turrets[s] = Some(Turret { era: p_era, cd: 0 });
            ev.push(Event::TurretBuilt { o: pi, slot: s, era: p_era });
        }
        Cmd::SellTurret { slot, .. } => {
            if *slot < 0 || *slot >= data.turret_slots as i64 {
                return;
            }
            let s = *slot as usize;
            let p = &mut st.players[pi];
            if let Some(t) = p.turrets[s].take() {
                p.gold += data.eras[t.era].turret.sell;
                ev.push(Event::TurretSold { o: pi, slot: s });
            }
        }
        Cmd::Special { .. } => {
            if st.players[pi].sp_cd > 0 {
                return;
            }
            st.players[pi].sp_cd = data.special_cd;
            let dmg = e.special.dmg;
            let mut hit = Vec::new();
            for u in st.units.iter_mut() {
                if u.o != pi {
                    u.hp -= dmg;
                    hit.push((u.id, u.x));
                }
            }
            ev.push(Event::Special { o: pi, era: p_era, bucket: e.bucket, dmg, hit });
        }
    }
}

struct Hit {

    tgt: Option<i64>,
    o: usize,
    dmg: i64,
    from: i64,
    role: i64,
    era: usize,
    fid: i64,
    slot: i64,
}

pub fn step(st: &mut State, data: &Data, cmds: &[Cmd]) -> Vec<Event> {
    let mut ev = Vec::new();
    if st.winner >= 0 {
        return ev;
    }
    st.tick += 1;

    for c in cmds {
        apply_command(st, data, c, &mut ev);
    }

    for o in 0..2 {
        let p = &mut st.players[o];
        p.gold_acc += (((data.eras[p.era].income * p.inc_pct) / 100) / data.tps) as i64;
        if p.gold_acc >= 1000 {
            p.gold += p.gold_acc / 1000;
            p.gold_acc %= 1000;
        }
        if p.sp_cd > 0 {
            p.sp_cd -= 1;
        }
    }

    for o in 0..2 {
        if st.players[o].queue.is_empty() {
            continue;
        }
        {
            let q = &mut st.players[o].queue[0];
            if q.t > 0 {
                q.t -= 1;
            }
        }
        if st.players[o].queue[0].t == 0 {
            let sx = data.base_front[o];
            let blocked = st.units.iter().any(|u| u.o == o && (u.x - sx).abs() < data.spacing);
            if !blocked {
                let q = st.players[o].queue.remove(0);
                let us = &data.eras[q.era].units[q.role];
                let id = st.next_id;
                st.next_id += 1;
                st.units.push(Unit { id, o, role: q.role, era: q.era, hp: us.hp, x: sx, cd: us.cd });
                ev.push(Event::Spawn { id, o, role: q.role, era: q.era, x: sx });
            }
        }
    }

    let mut hits: Vec<Hit> = Vec::new();
    let n = st.units.len();
    for i in 0..n {
        if st.units[i].cd > 0 {
            st.units[i].cd -= 1;
        }
        let (uo, urole, uera, ux) = (st.units[i].o, st.units[i].role, st.units[i].era, st.units[i].x);
        let us = &data.eras[uera].units[urole];
        let dir = dir_of(uo);

        let mut best: Option<usize> = None;
        let mut bd = 1i64 << 30;
        for (j, e) in st.units.iter().enumerate() {
            if e.o == uo {
                continue;
            }
            let d = (e.x - ux).abs();
            if d < bd {
                bd = d;
                best = Some(j);
            }
        }
        let base_x = data.base_front[1 - uo];
        let db = (base_x - ux).abs();
        let target_unit = best.is_some() && bd <= us.range;
        let target_base = !target_unit && db <= us.range;
        if target_unit || target_base {
            if st.units[i].cd == 0 {
                st.units[i].cd = us.cd;
                let lo = data.dmg_var_pct[0];
                let hi = data.dmg_var_pct[1];
                let v = lo + (rng_u32(st) as i64 % (hi - lo + 1));
                let mut dmg = (us.dmg * v) / 100;
                let mut tgt = None;
                if target_unit {
                    let b = best.unwrap();
                    dmg = (dmg * data.counter_pct[urole][st.units[b].role]) / 100;
                    tgt = Some(st.units[b].id);
                }
                hits.push(Hit { tgt, o: uo, dmg, from: ux, role: urole as i64, era: uera, fid: st.units[i].id, slot: 0 });
            }
            continue;
        }

        let mut blocked = false;
        for (j, v) in st.units.iter().enumerate() {
            if j == i || v.o != uo {
                continue;
            }
            let ahead = (v.x - ux) * dir;
            if ahead > 0 && ahead < data.spacing {
                blocked = true;
                break;
            }
        }
        if !blocked {
            let mut nx = ux + dir * us.speed;
            let lim = base_x - dir * 60;
            if (nx - lim) * dir > 0 {
                nx = lim;
            }
            st.units[i].x = nx;
        }
    }

    for o in 0..2 {
        let tx = data.base_front[o];
        for s in 0..st.players[o].turrets.len() {
            let (tcd, tera) = match &st.players[o].turrets[s] {
                None => continue,
                Some(t) => (t.cd, t.era),
            };
            if tcd > 0 {
                st.players[o].turrets[s].as_mut().unwrap().cd -= 1;
                continue;
            }
            let t = &data.eras[tera].turret;
            let mut best: Option<usize> = None;
            let mut bd = 1i64 << 30;
            for (j, e) in st.units.iter().enumerate() {
                if e.o == o {
                    continue;
                }
                let d = (e.x - tx).abs();
                if d < bd {
                    bd = d;
                    best = Some(j);
                }
            }
            if let Some(b) = best {
                if bd <= t.range {
                    st.players[o].turrets[s].as_mut().unwrap().cd = t.cd;
                    let lo = data.dmg_var_pct[0];
                    let hi = data.dmg_var_pct[1];
                    let v = lo + (rng_u32(st) as i64 % (hi - lo + 1));
                    hits.push(Hit {
                        tgt: Some(st.units[b].id),
                        o,
                        dmg: (t.dmg * v) / 100,
                        from: tx,
                        role: -1,
                        era: tera,
                        fid: -(s as i64 + 1),
                        slot: s as i64,
                    });
                }
            }
        }
    }

    for h in hits.iter_mut() {
        match h.tgt {
            Some(id) => {
                if let Some(e) = st.units.iter_mut().find(|u| u.id == id) {
                    e.hp -= h.dmg;
                    ev.push(Event::Hit { tgt_unit: Some(e.id), fid: h.fid, x: e.x, dmg: h.dmg, o: h.o, from: h.from, role: h.role, era: h.era, slot: h.slot });
                }
            }
            None => {
                h.dmg = (h.dmg * data.base_dmg_pct) / 100;
                st.players[1 - h.o].base_hp -= h.dmg;
                ev.push(Event::Hit { tgt_unit: None, fid: h.fid, x: data.base_front[1 - h.o], dmg: h.dmg, o: h.o, from: h.from, role: h.role, era: h.era, slot: h.slot });
            }
        }
    }

    let mut alive = Vec::with_capacity(st.units.len());
    let units = std::mem::take(&mut st.units);
    for u in units {
        if u.hp > 0 {
            alive.push(u);
            continue;
        }
        let us = &data.eras[u.era].units[u.role];
        let killer = &mut st.players[1 - u.o];
        killer.gold += us.reward;
        killer.xp += us.xp;
        ev.push(Event::Death { id: u.id, o: u.o, role: u.role, era: u.era, x: u.x });
    }
    st.units = alive;

    if st.tick >= data.sudden_tick && st.tick % data.tps == 0 {
        for p in st.players.iter_mut() {
            p.base_hp -= (p.base_max * data.sudden_permille) / 1000;
        }
        ev.push(Event::Siege);
    }

    let d0 = st.players[0].base_hp <= 0;
    let d1 = st.players[1].base_hp <= 0;
    if d0 || d1 {
        for p in st.players.iter_mut() {
            if p.base_hp < 0 {
                p.base_hp = 0;
            }
        }
        st.winner = if d0 && d1 { 2 } else if d0 { 1 } else { 0 };
        ev.push(Event::Win { winner: st.winner });
    }
    ev
}

pub fn hash_state(st: &State) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    let mut mix = |v: i64| {
        let mut v = v as i32;
        for _ in 0..4 {
            h ^= (v & 255) as u32;
            h = h.wrapping_mul(0x0100_0193);
            v >>= 8;
        }
    };
    mix(st.tick);
    mix(st.seed);
    mix(st.rng as i32 as i64);
    mix(st.next_id);
    mix(st.winner);
    for p in &st.players {
        mix(p.inc_pct);
        mix(p.gold);
        mix(p.gold_acc);
        mix(p.xp);
        mix(p.era as i64);
        mix(p.base_hp);
        mix(p.base_max);
        mix(p.sp_cd);
        mix(p.queue.len() as i64);
        for q in &p.queue {
            mix(q.role as i64);
            mix(q.era as i64);
            mix(q.t);
        }
        for t in &p.turrets {
            match t {
                Some(t) => {
                    mix(1);
                    mix(t.era as i64);
                    mix(t.cd);
                }
                None => mix(0),
            }
        }
    }
    mix(st.units.len() as i64);
    for u in &st.units {
        mix(u.id);
        mix(u.o as i64);
        mix(u.role as i64);
        mix(u.era as i64);
        mix(u.hp);
        mix(u.x);
        mix(u.cd);
    }
    h
}
