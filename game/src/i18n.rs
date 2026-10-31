use davrlar_core::ai::Level;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum Lang {
    Ru,
    En,
    Uz,
}

pub const LANGS: [Lang; 3] = [Lang::Ru, Lang::En, Lang::Uz];

pub struct Ui {
    pub play: &'static str,
    pub watch: &'static str,
    pub resume: &'static str,
    pub again: &'static str,
    pub replay: &'static str,
    pub saved: &'static str,
    pub menu: &'static str,
    pub paused: &'static str,
    pub roles: [&'static str; 3],
    pub levels: [&'static str; 3],
    century: &'static str,
    pub enemy: &'static str,
    pub siege: &'static str,
    pub siege_clock: &'static str,
    pub draw: &'static str,
    pub blue_win: &'static str,
    pub red_win: &'static str,
    pub win: &'static str,
    pub lose: &'static str,
    pub evolve: &'static str,
    pub auto: &'static str,
    pub auto_on: &'static str,
    pub auto_off: &'static str,
    pub per_sec: &'static str,
    pub sec: &'static str,
    pub sell: &'static str,
    pub ready: &'static str,
}

const RU: Ui = Ui {
    play: "В бой",
    watch: "Наблюдать (бот против бота)",
    resume: "Продолжить",
    again: "Ещё раз",
    replay: "Сохранить реплей",
    saved: "Реплей сохранён",
    menu: "Меню",
    paused: "ПАУЗА",
    roles: ["Пехота", "Стрелок", "Тяжёлый"],
    levels: ["Лёгкий", "Нормальный", "Сложный"],
    century: "{r} век",
    enemy: "Враг: ",
    siege: "Осада",
    siege_clock: "ОСАДА",
    draw: "Ничья",
    blue_win: "Победа синих",
    red_win: "Победа красных",
    win: "Победа!",
    lose: "Поражение",
    evolve: "Эволюция",
    auto: "Авто",
    auto_on: "Авто: вкл",
    auto_off: "Авто: выкл",
    per_sec: "/с",
    sec: "с",
    sell: "продать",
    ready: "готово",
};

const EN: Ui = Ui {
    play: "Play",
    watch: "Watch (bot vs bot)",
    resume: "Resume",
    again: "Again",
    replay: "Save replay",
    saved: "Replay saved",
    menu: "Menu",
    paused: "PAUSED",
    roles: ["Infantry", "Archer", "Heavy"],
    levels: ["Easy", "Normal", "Hard"],
    century: "Century {r}",
    enemy: "Enemy: ",
    siege: "Siege",
    siege_clock: "SIEGE",
    draw: "Draw",
    blue_win: "Blue wins",
    red_win: "Red wins",
    win: "Victory!",
    lose: "Defeat",
    evolve: "Evolve",
    auto: "Auto",
    auto_on: "Auto: on",
    auto_off: "Auto: off",
    per_sec: "/s",
    sec: "s",
    sell: "sell",
    ready: "ready",
};

const UZ: Ui = Ui {
    play: "Jangga",
    watch: "Kuzatish (bot botga qarshi)",
    resume: "Davom etish",
    again: "Yana",
    replay: "Yozuvni saqlash",
    saved: "Yozuv saqlandi",
    menu: "Menyu",
    paused: "PAUZA",
    roles: ["Piyoda", "O'qchi", "Og'ir"],
    levels: ["Oson", "O'rtacha", "Qiyin"],
    century: "{r} asr",
    enemy: "Dushman: ",
    siege: "Qamal",
    siege_clock: "QAMAL",
    draw: "Durang",
    blue_win: "Ko'klar g'olib",
    red_win: "Qizillar g'olib",
    win: "G'alaba!",
    lose: "Mag'lubiyat",
    evolve: "Evolyutsiya",
    auto: "Avto",
    auto_on: "Avto: yoqilgan",
    auto_off: "Avto: o'chiq",
    per_sec: "/s",
    sec: "s",
    sell: "sotish",
    ready: "tayyor",
};

impl Lang {
    pub fn key(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
            Lang::Uz => "uz",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Lang::Ru => "RU",
            Lang::En => "EN",
            Lang::Uz => "UZ",
        }
    }
    pub fn parse(s: &str) -> Option<Lang> {
        LANGS.into_iter().find(|l| l.key() == s)
    }
    pub fn detect(saved: Option<&str>) -> Lang {
        if let Some(l) = saved.and_then(Lang::parse) {
            return l;
        }
        let n = sys_locale::get_locale().unwrap_or_default().to_lowercase();
        if n.starts_with("uz") {
            Lang::Uz
        } else if n.starts_with("ru") {
            Lang::Ru
        } else {
            Lang::En
        }
    }
    pub fn ui(self) -> &'static Ui {
        match self {
            Lang::Ru => &RU,
            Lang::En => &EN,
            Lang::Uz => &UZ,
        }
    }
    pub fn century(self, roman: &str) -> String {
        self.ui().century.replace("{r}", roman)
    }
}

pub fn level_index(l: Level) -> usize {
    match l {
        Level::Easy => 0,
        Level::Normal => 1,
        Level::Hard => 2,
    }
}

#[derive(Deserialize)]
struct EraNames {
    name: String,
    units: Vec<String>,
}
#[derive(Deserialize)]
struct BucketNames {
    turret: String,
    special: String,
}
#[derive(Deserialize)]
struct LangNames {
    eras: Vec<EraNames>,
    buckets: Vec<BucketNames>,
}

pub struct Names(HashMap<String, LangNames>);

impl Names {
    pub fn load(json: &str) -> Names {
        Names(serde_json::from_str(json).expect("разбор names.json"))
    }
    fn of(&self, l: Lang) -> &LangNames {
        &self.0[l.key()]
    }
    pub fn era(&self, l: Lang, era: usize) -> &str {
        &self.of(l).eras[era].name
    }
    pub fn unit(&self, l: Lang, era: usize, role: usize) -> &str {
        &self.of(l).eras[era].units[role]
    }
    pub fn turret(&self, l: Lang, bucket: usize) -> &str {
        &self.of(l).buckets[bucket].turret
    }
    pub fn special(&self, l: Lang, bucket: usize) -> &str {
        &self.of(l).buckets[bucket].special
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_cover_every_era_and_bucket() {
        let names = Names::load(include_str!("../../data/names.json"));
        for l in LANGS {
            for e in 0..25 {
                assert!(!names.era(l, e).is_empty());
                for r in 0..3 {
                    assert!(!names.unit(l, e, r).is_empty());
                }
            }
            for b in 0..5 {
                assert!(!names.turret(l, b).is_empty() && !names.special(l, b).is_empty());
            }
        }
    }
}
