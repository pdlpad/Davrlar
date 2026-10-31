use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UnitStats {
    pub name: String,
    pub hp: i64,
    pub dmg: i64,
    pub range: i64,
    pub cd: i64,
    pub speed: i64,
    pub cost: i64,
    pub train: i64,
    pub reward: i64,
    pub xp: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TurretStats {
    pub name: String,
    pub dmg: i64,
    pub range: i64,
    pub cd: i64,
    pub cost: i64,
    pub sell: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpecialStats {
    pub name: String,
    pub dmg: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Era {
    pub century: i64,
    pub roman: String,
    pub name: String,
    pub bucket: usize,
    pub units: Vec<UnitStats>,
    pub turret: TurretStats,
    pub special: SpecialStats,
    pub base_max: i64,
    pub xp_need: i64,
    pub income: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub version: i64,
    pub tps: i64,
    pub lane: i64,
    pub base_front: [i64; 2],
    pub spacing: i64,
    pub start_gold: i64,
    pub queue_max: usize,
    pub turret_slots: usize,
    pub special_cd: i64,
    pub dmg_var_pct: [i64; 2],
    pub base_dmg_pct: i64,
    pub counter_pct: [[i64; 3]; 3],
    pub sudden_tick: i64,
    pub sudden_permille: i64,
    pub eras: Vec<Era>,
}

impl Data {
    pub fn load(json: &str) -> Result<Data, serde_json::Error> {
        serde_json::from_str(json)
    }
}
