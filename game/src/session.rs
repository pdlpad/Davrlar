use davrlar_core::ai::{ai_commands, Level};
use davrlar_core::data::Data;
use davrlar_core::sim::{hash_state, new_game, step, Cmd, Event, State};
use serde_json::{json, Value};

const MAX_STEPS: i32 = 48;
pub const SPEEDS: [i32; 5] = [1, 2, 4, 8, 16];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Menu,
    Play,
    Over,
}

pub struct Session {
    pub mode: Mode,
    pub st: State,
    pub level: Level,
    pub auto: bool,
    pub demo: bool,
    pub speed: i32,
    pub paused: bool,
    pub seed: i64,
    inc: [i64; 2],
    acc: f64,
    pending: Vec<Cmd>,
    log: Vec<(i64, Vec<Cmd>)>,
}

pub fn level_key(l: Level) -> &'static str {
    match l {
        Level::Easy => "easy",
        Level::Normal => "normal",
        Level::Hard => "hard",
    }
}

fn cmd_json(c: &Cmd) -> Value {
    match c {
        Cmd::Train { p, role } => json!({"t": "train", "p": p, "role": role}),
        Cmd::Evolve { p } => json!({"t": "evolve", "p": p}),
        Cmd::BuyTurret { p, slot } => json!({"t": "buyTurret", "p": p, "slot": slot}),
        Cmd::SellTurret { p, slot } => json!({"t": "sellTurret", "p": p, "slot": slot}),
        Cmd::Special { p } => json!({"t": "special", "p": p}),
    }
}

impl Session {
    pub fn new(data: &Data, seed: i64, level: Level, demo: bool, auto: bool) -> Session {
        let auto = auto || demo;
        let inc = [if auto { level.income_pct() } else { 100 }, level.income_pct()];
        Session {
            mode: if demo { Mode::Menu } else { Mode::Play },
            st: new_game(data, seed, inc),
            level,
            auto,
            demo,
            speed: if demo { 2 } else { 1 },
            paused: false,
            seed,
            inc,
            acc: 0.0,
            pending: Vec::new(),
            log: Vec::new(),
        }
    }

    pub fn command(&mut self, c: Cmd) {
        if self.mode == Mode::Play && !self.auto && !self.paused {
            self.pending.push(c);
        }
    }

    pub fn toggle_slot(&mut self, slot: usize) {
        let c = if self.st.players[0].turrets[slot].is_some() {
            Cmd::SellTurret { p: 0, slot: slot as i64 }
        } else {
            Cmd::BuyTurret { p: 0, slot: slot as i64 }
        };
        self.command(c);
    }

    fn step_once(&mut self, data: &Data) -> Vec<Event> {
        let mut cmds = std::mem::take(&mut self.pending);
        if self.auto {
            cmds.extend(ai_commands(&self.st, data, 0, self.level));
        }
        cmds.extend(ai_commands(&self.st, data, 1, self.level));
        let ev = step(&mut self.st, data, &cmds);
        if !cmds.is_empty() && !self.demo {
            self.log.push((self.st.tick, cmds));
        }
        if self.st.winner >= 0 && self.mode == Mode::Play {
            self.mode = Mode::Over;
        }
        ev
    }

    pub fn advance(&mut self, data: &Data, dt: f64) -> Vec<Event> {
        let mut ev = Vec::new();
        let running = (self.mode == Mode::Play || self.demo) && !self.paused;
        if !running || self.st.winner >= 0 {
            return ev;
        }
        let tick = 1.0 / data.tps as f64;
        self.acc += dt * self.speed as f64;
        let mut n = 0;
        while self.acc >= tick && n < MAX_STEPS {
            ev.extend(self.step_once(data));
            self.acc -= tick;
            n += 1;
        }
        if n == MAX_STEPS {
            self.acc = 0.0;
        }
        ev
    }

    pub fn replay_json(&self) -> String {
        let log: Vec<Value> = self
            .log
            .iter()
            .map(|(t, cmds)| json!([t, cmds.iter().map(cmd_json).collect::<Vec<_>>()]))
            .collect();
        json!({
            "format": 1,
            "game": "davrlar",
            "seed": self.seed,
            "incomePct": self.inc,
            "level": level_key(self.level),
            "autoPlayer0": self.auto,
            "ticks": self.st.tick,
            "winner": self.st.winner,
            "finalHash": hash_state(&self.st),
            "log": log,
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use davrlar_core::replay::{replay_by_log, selfplay, Replay};

    const GAME: &str = include_str!("../../data/game.json");

    fn play(s: &mut Session, data: &Data) {
        for _ in 0..30 * 60 * 25 {
            s.advance(data, 1.0 / 30.0);
            if s.mode == Mode::Over {
                break;
            }
        }
    }

    #[test]
    fn replay_matches_core() {
        let data = Data::load(GAME).unwrap();
        let mut s = Session::new(&data, 77, Level::Normal, false, true);
        play(&mut s, &data);
        assert_eq!(s.mode, Mode::Over);
        let mut r: Replay = serde_json::from_str(&s.replay_json()).unwrap();
        let out = replay_by_log(&data, &r);
        assert_eq!((out.ticks, out.winner, out.hash), (r.ticks, r.winner, r.final_hash));
        r.levels = Some(["normal".into(), "normal".into()]);
        let bots = selfplay(&data, &r, 30 * 60 * 25).unwrap();
        assert_eq!(bots, out);
    }

    #[test]
    fn human_commands_enter_the_log_on_their_tick() {
        let data = Data::load(GAME).unwrap();
        let mut s = Session::new(&data, 5, Level::Easy, false, false);
        s.advance(&data, 1.0 / 30.0 * 3.0);
        s.command(Cmd::Train { p: 0, role: 0 });
        s.toggle_slot(1);
        let tick = s.st.tick + 1;
        s.advance(&data, 1.0 / 30.0);
        assert_eq!(s.log[0].0, tick);
        assert_eq!(s.log[0].1[0], Cmd::Train { p: 0, role: 0 });
        play(&mut s, &data);
        let r: Replay = serde_json::from_str(&s.replay_json()).unwrap();
        let out = replay_by_log(&data, &r);
        assert_eq!((out.ticks, out.winner, out.hash), (r.ticks, r.winner, r.final_hash));
    }

    #[test]
    fn paused_game_does_not_advance_or_accept_commands() {
        let data = Data::load(GAME).unwrap();
        let mut s = Session::new(&data, 5, Level::Easy, false, false);
        s.paused = true;
        s.command(Cmd::Evolve { p: 0 });
        s.advance(&data, 1.0);
        assert_eq!(s.st.tick, 0);
        assert!(s.pending.is_empty());
    }

    #[test]
    fn top_speed_holds_at_sixty_and_twenty_fps() {
        let data = Data::load(GAME).unwrap();
        for fps in [60, 20] {
            let mut s = Session::new(&data, 5, Level::Easy, false, false);
            s.speed = 16;
            for _ in 0..fps {
                s.advance(&data, 1.0 / fps as f64);
            }
            assert!((s.st.tick - 480).abs() <= 1, "{fps} fps: {}", s.st.tick);
        }
    }

    #[test]
    fn speed_is_capped_per_frame() {
        let data = Data::load(GAME).unwrap();
        let mut s = Session::new(&data, 5, Level::Easy, false, false);
        s.speed = 4;
        s.advance(&data, 10.0);
        assert_eq!(s.st.tick, MAX_STEPS as i64);
    }
}
