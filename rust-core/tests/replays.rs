use davrlar_core::data::Data;
use davrlar_core::replay::{replay_by_log, selfplay, Replay};
use std::fs;

fn load() -> (Data, Vec<(String, Replay)>) {
    let data = Data::load(&fs::read_to_string("data/game.json").unwrap()).unwrap();
    let mut out = Vec::new();
    for e in fs::read_dir("tests/replays").unwrap() {
        let p = e.unwrap().path();
        if p.extension().map_or(false, |x| x == "json") {
            let r: Replay = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
            out.push((p.file_name().unwrap().to_string_lossy().into_owned(), r));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(!out.is_empty(), "нет реплеев: запустите node tools/export-replays.mjs");
    (data, out)
}

#[test]
fn replay_by_command_log_matches_js() {
    let (data, reps) = load();
    for (name, r) in &reps {
        let o = replay_by_log(&data, r);
        assert_eq!((o.ticks, o.winner, o.hash), (r.ticks, r.winner, r.final_hash), "{name}: расхождение при воспроизведении по журналу");
    }
}

#[test]
fn rust_bots_play_identically_to_js_bots() {
    let (data, reps) = load();
    for (name, r) in &reps {
        let o = selfplay(&data, r, 30 * 60 * 25).expect("у реплея нет levels");
        assert_eq!((o.ticks, o.winner, o.hash), (r.ticks, r.winner, r.final_hash), "{name}: бот/симуляция разошлись с JS");
    }
}
