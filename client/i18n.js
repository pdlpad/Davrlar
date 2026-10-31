import names from "../data/names.json";

export const LANGS = ["ru", "en", "uz"];
export const LANG_LABEL = { ru: "RU", en: "EN", uz: "UZ" };

const UI = {
  ru: {
    doc: "Davrlar — 25 столетий войны",
    title: "DAVRLAR",
    play: "В бой",
    watch: "Наблюдать (бот против бота)",
    resume: "Продолжить",
    again: "Ещё раз",
    replay: "Скачать реплей",
    menu: "Меню",
    paused: "ПАУЗА",
    roles: ["Пехота", "Стрелок", "Тяжёлый"],
    levels: { easy: "Лёгкий", normal: "Нормальный", hard: "Сложный" },
    century: "{r} век",
    enemy: "Враг: ",
    siege: "Осада",
    siegeClock: "ОСАДА",
    draw: "Ничья",
    blueWin: "Победа синих",
    redWin: "Победа красных",
    win: "Победа!",
    lose: "Поражение",
    evolve: "Эволюция",
    auto: "Авто",
    autoOn: "Авто: вкл",
    autoOff: "Авто: выкл",
    perSec: "/с",
    sec: "с",
    sell: "продать",
    ready: "готово",
  },
  en: {
    doc: "Davrlar — 25 centuries of war",
    title: "DAVRLAR",
    play: "Play",
    watch: "Watch (bot vs bot)",
    resume: "Resume",
    again: "Again",
    replay: "Download replay",
    menu: "Menu",
    paused: "PAUSED",
    roles: ["Infantry", "Archer", "Heavy"],
    levels: { easy: "Easy", normal: "Normal", hard: "Hard" },
    century: "Century {r}",
    enemy: "Enemy: ",
    siege: "Siege",
    siegeClock: "SIEGE",
    draw: "Draw",
    blueWin: "Blue wins",
    redWin: "Red wins",
    win: "Victory!",
    lose: "Defeat",
    evolve: "Evolve",
    auto: "Auto",
    autoOn: "Auto: on",
    autoOff: "Auto: off",
    perSec: "/s",
    sec: "s",
    sell: "sell",
    ready: "ready",
  },
  uz: {
    doc: "Davrlar — 25 asrlik jang",
    title: "DAVRLAR",
    play: "Jangga",
    watch: "Kuzatish (bot botga qarshi)",
    resume: "Davom etish",
    again: "Yana",
    replay: "Yozuvni yuklash",
    menu: "Menyu",
    paused: "PAUZA",
    roles: ["Piyoda", "O'qchi", "Og'ir"],
    levels: { easy: "Oson", normal: "O'rtacha", hard: "Qiyin" },
    century: "{r} asr",
    enemy: "Dushman: ",
    siege: "Qamal",
    siegeClock: "QAMAL",
    draw: "Durang",
    blueWin: "Ko'klar g'olib",
    redWin: "Qizillar g'olib",
    win: "G'alaba!",
    lose: "Mag'lubiyat",
    evolve: "Evolyutsiya",
    auto: "Avto",
    autoOn: "Avto: yoqilgan",
    autoOff: "Avto: o'chiq",
    perSec: "/s",
    sec: "s",
    sell: "sotish",
    ready: "tayyor",
  },
};

let lang = "ru";

export function detectLang(saved) {
  if (saved && UI[saved]) return saved;
  const n = (typeof navigator !== "undefined" && navigator.language || "").toLowerCase();
  if (n.startsWith("uz")) return "uz";
  if (n.startsWith("ru")) return "ru";
  return "en";
}

export const getLang = () => lang;
export const setLang = (l) => { if (UI[l]) lang = l; };
export const t = (k) => UI[lang][k];
export const century = (roman) => UI[lang].century.replace("{r}", roman);
export const centuryHtml = (roman) => UI[lang].century.replace("{r}", `<b>${roman}</b>`);
export const eraName = (era) => names[lang].eras[era].name;
export const unitName = (era, role) => names[lang].eras[era].units[role];
export const turretName = (bucket) => names[lang].buckets[bucket].turret;
export const specialName = (bucket) => names[lang].buckets[bucket].special;
