use crate::ai::{ai_commands, Level};
use crate::data::Data;
use crate::sim::{hash_state, new_game, step, Cmd};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Replay {
    pub seed: i64,
    pub income_pct: [i64; 2],

    #[serde(default)]
    pub levels: Option<[String; 2]>,
    pub ticks: i64,
    pub winner: i64,
    pub final_hash: u32,

    pub log: Vec<(i64, Vec<Cmd>)>,
}

#[derive(Debug, PartialEq)]
pub struct Outcome {
    pub ticks: i64,
    pub winner: i64,
    pub hash: u32,
}

pub fn replay_by_log(data: &Data, r: &Replay) -> Outcome {
    let mut st = new_game(data, r.seed, r.income_pct);
    let mut li = 0;
    while st.winner < 0 && st.tick < r.ticks {
        let next = st.tick + 1;
        let mut cmds: &[Cmd] = &[];
        if li < r.log.len() && r.log[li].0 == next {
            cmds = &r.log[li].1;
            li += 1;
        }
        step(&mut st, data, cmds);
    }
    Outcome { ticks: st.tick, winner: st.winner, hash: hash_state(&st) }
}

pub fn selfplay(data: &Data, r: &Replay, max_ticks: i64) -> Option<Outcome> {
    let lv = r.levels.as_ref()?;
    let (l0, l1) = (Level::parse(&lv[0])?, Level::parse(&lv[1])?);
    let mut st = new_game(data, r.seed, r.income_pct);
    while st.winner < 0 && st.tick < max_ticks {
        let mut cmds = ai_commands(&st, data, 0, l0);
        cmds.extend(ai_commands(&st, data, 1, l1));
        step(&mut st, data, &cmds);
    }
    Some(Outcome { ticks: st.tick, winner: st.winner, hash: hash_state(&st) })
}
