use crate::data::Data;
use crate::sim::{Cmd, State};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level {
    Easy,
    Normal,
    Hard,
}

impl Level {
    pub fn parse(s: &str) -> Option<Level> {
        match s {
            "easy" => Some(Level::Easy),
            "normal" => Some(Level::Normal),
            "hard" => Some(Level::Hard),
            _ => None,
        }
    }

    pub fn income_pct(self) -> i64 {
        match self {
            Level::Easy => 92,
            Level::Normal => 100,
            Level::Hard => 108,
        }
    }
    fn params(self) -> Params {
        match self {
            Level::Easy => Params { every: 45, slip: 35, queue: 1, turret_after: 5400, pick: 10 },
            Level::Normal => Params { every: 20, slip: 12, queue: 2, turret_after: 2700, pick: 65 },
            Level::Hard => Params { every: 12, slip: 0, queue: 2, turret_after: 2700, pick: 100 },
        }
    }
}

struct Params {
    every: i64,
    slip: u32,
    queue: usize,
    turret_after: i64,
    pick: u32,
}

fn h32(a: u32, b: u32) -> u32 {
    let mut h = a.wrapping_mul(0x9e37_79b1).wrapping_add(b.wrapping_add(0x7f4a_7c15).wrapping_mul(0x85eb_ca6b));
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h
}

pub fn ai_commands(st: &State, data: &Data, me: usize, level: Level) -> Vec<Cmd> {
    let l = level.params();
    if st.winner >= 0 {
        return vec![];
    }
    if (st.tick + me as i64 * 3) % l.every != 0 {
        return vec![];
    }
    let r = h32((st.tick as u32) ^ (st.seed as u32).wrapping_mul(0x27d4_eb2f), me as u32 + 1);
    if l.slip > 0 && r % 100 < l.slip {
        return vec![];
    }

    let p = &st.players[me];
    let e = &data.eras[p.era];
    let mut cmds = Vec::new();

    if p.era < data.eras.len() - 1 && p.xp >= e.xp_need {
        return vec![Cmd::Evolve { p: me }];
    }

    let my_front = data.base_front[me];
    let mut near = [0i64; 3];
    let mut total = 0i64;
    let mut near_base = 0i64;
    for u in &st.units {
        if u.o == me {
            continue;
        }
        total += 1;
        near[u.role] += 1;
        if (u.x - my_front).abs() < 2400 {
            near_base += 1;
        }
    }

    if p.sp_cd == 0 && (total >= 4 || (near_base >= 2 && p.base_hp * 3 < p.base_max)) {
        cmds.push(Cmd::Special { p: me });
    }

    if st.tick >= l.turret_after {
        let slot = p.turrets.iter().position(|t| t.is_none());
        match slot {
            Some(s) => {
                if p.gold >= e.turret.cost + e.units[0].cost * 2 {
                    cmds.push(Cmd::BuyTurret { p: me, slot: s as i64 });
                }
            }
            None => {

                let old = p.turrets.iter().position(|t| matches!(t, Some(t) if t.era + 3 <= p.era));
                if let Some(o) = old {
                    let sell = data.eras[p.turrets[o].as_ref().unwrap().era].turret.sell;
                    if p.gold >= e.turret.cost - sell {
                        cmds.push(Cmd::SellTurret { p: me, slot: o as i64 });
                        cmds.push(Cmd::BuyTurret { p: me, slot: o as i64 });
                    }
                }
            }
        }
    }

    if p.queue.len() < l.queue {
        let mut want: i64;
        if total == 0 {
            want = if r % 3 == 0 { 1 } else { 0 };
        } else if (r >> 8) % 100 >= l.pick {
            want = ((r >> 16) % 3) as i64;
        } else {

            let mut best_score = -1i64 << 30;
            want = 0;
            for ro in 0..3usize {
                let mut sc = 0i64;
                for en in 0..3usize {
                    sc += near[en] * (data.counter_pct[ro][en] - data.counter_pct[en][ro]);
                }
                if sc > best_score {
                    best_score = sc;
                    want = ro as i64;
                }
            }
        }
        let dangerous = near_base >= 3;
        if e.units[want as usize].cost > p.gold {

            want = if dangerous { 0 } else { -1 };
        }
        if want >= 0 && e.units[want as usize].cost <= p.gold {
            cmds.push(Cmd::Train { p: me, role: want });
        }
    }
    cmds
}
