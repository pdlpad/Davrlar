use davrlar_core::data::Data;
use davrlar_core::replay::{replay_by_log, selfplay, Replay};
use std::{env, fs, process, time::Instant};

fn usage() -> ! {
    eprintln!("использование:\n  davrlar-core verify <game.json> <replay.json>...   # воспроизвести по журналу и сыграть ботами\n  davrlar-core bench  <game.json> [партий]            # скорость симуляции");
    process::exit(2)
}

fn main() {
    let a: Vec<String> = env::args().collect();
    if a.len() < 3 {
        usage();
    }
    let data = Data::load(&fs::read_to_string(&a[2]).expect("game.json")).expect("разбор game.json");
    match a[1].as_str() {
        "verify" => {
            let mut bad = 0;
            for path in &a[3..] {
                let r: Replay = serde_json::from_str(&fs::read_to_string(path).expect("replay")).expect("разбор реплея");
                let want = (r.ticks, r.winner, r.final_hash);
                let by_log = replay_by_log(&data, &r);
                let ok1 = (by_log.ticks, by_log.winner, by_log.hash) == want;
                let sp = selfplay(&data, &r, 30 * 60 * 25);
                let ok2 = sp.as_ref().map(|o| (o.ticks, o.winner, o.hash) == want);
                println!(
                    "{:<26} по журналу: {:<4} сам ботами: {:<4} ticks={} winner={} hash={:08x}",
                    path.rsplit('/').next().unwrap_or(path),
                    if ok1 { "OK" } else { "FAIL" },
                    match ok2 { Some(true) => "OK", Some(false) => "FAIL", None => "—" },
                    r.ticks, r.winner, r.final_hash
                );
                if !ok1 || ok2 == Some(false) {
                    bad += 1;
                    eprintln!("   ожидалось {:?}, по журналу {:?}, ботами {:?}", want, (by_log.ticks, by_log.winner, by_log.hash), sp.map(|o| (o.ticks, o.winner, o.hash)));
                }
            }
            if bad > 0 {
                eprintln!("расхождений: {bad}");
                process::exit(1);
            }
            println!("все реплеи совпали с JS побитово");
        }
        "bench" => {
            use davrlar_core::{ai::*, sim::*};
            let n: i64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(200);
            let t0 = Instant::now();
            let (mut ticks, mut w) = (0i64, [0; 3]);
            for seed in 1..=n {
                let mut st = new_game(&data, seed, [100, 100]);
                while st.winner < 0 && st.tick < 30 * 60 * 25 {
                    let mut c = ai_commands(&st, &data, 0, Level::Normal);
                    c.extend(ai_commands(&st, &data, 1, Level::Normal));
                    step(&mut st, &data, &c);
                }
                ticks += st.tick;
                w[if st.winner < 0 { 2 } else { st.winner as usize }] += 1;
            }
            let s = t0.elapsed().as_secs_f64();
            println!("{n} партий, {ticks} тиков за {s:.2} с = {:.0} тиков/с ({:.0}× реального времени); победы {:?}", ticks as f64 / s, ticks as f64 / s / 30.0, w);
        }
        _ => usage(),
    }
}
