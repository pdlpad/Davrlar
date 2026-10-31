import { writeFileSync } from "node:fs";

const G = 1.19;
const S = 1.27;

const ERAS = [
  ["Ранняя империя", ["Легионер", "Велит-пращник", "Центурион"]],
  ["Золотой век Антонинов", ["Гастат", "Сагиттарий", "Преторианец"]],
  ["Кризис III века", ["Лимитан", "Балеарский пращник", "Клибанарий"]],
  ["Поздняя империя", ["Комитат", "Конный лучник", "Схолария"]],
  ["Падение Запада", ["Федерат", "Готский лучник", "Букеллярий"]],
  ["Византия", ["Скутат", "Токсот", "Катафрактарий"]],
  ["Арабский халифат", ["Воин джунда", "Лучник-бедуин", "Тяжёлый всадник"]],
  ["Каролинги", ["Франкский воин", "Стрелок Карла", "Паладин"]],
  ["Эпоха викингов", ["Бонд", "Метатель дротиков", "Берсерк"]],
  ["Раннее рыцарство", ["Копейщик", "Лучник", "Рыцарь"]],
  ["Крестовые походы", ["Сержант", "Арбалетчик", "Тамплиер"]],
  ["Высокое Средневековье", ["Алебардист", "Генуэзский арбалетчик", "Госпитальер"]],
  ["Монгольская империя", ["Пеший нукер", "Конный лучник", "Кешик"]],
  ["Столетняя война", ["Билльмен", "Лонгбоумен", "Латник"]],
  ["Ренессанс", ["Пикинёр", "Аркебузир", "Кондотьер"]],
  ["Эпоха пороха", ["Ландскнехт", "Мушкетёр", "Кирасир"]],
  ["Тридцатилетняя война", ["Пикинёр", "Мушкетёр", "Рейтар"]],
  ["Эпоха Просвещения", ["Линейный пехотинец", "Фузилёр", "Гренадёр"]],
  ["Индустриальная эра", ["Стрелок", "Егерь", "Кирасир-гвардеец"]],
  ["Мировые войны", ["Пехотинец", "Пулемётчик", "Танк"]],
  ["Информационная эпоха", ["Мотострелок", "Снайпер", "Основной танк"]],
  ["Эра роботов", ["Боевой дрон", "Дрон-стрелок", "Шагоход"]],
  ["Энергетическое оружие", ["Штурмовик в экзоскелете", "Лазерный стрелок", "Тяжёлый мех"]],
  ["Нанотехнологии", ["Нано-боец", "Плазменный стрелок", "Нано-титан"]],
  ["Космическая эра", ["Звёздный пехотинец", "Ионный стрелок", "Дредноут-мех"]],
];

const ROMAN = ["", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII", "XIII", "XIV", "XV", "XVI", "XVII", "XVIII", "XIX", "XX", "XXI", "XXII", "XXIII", "XXIV", "XXV"];

const bucketOf = (e) => (e <= 5 ? 0 : e <= 13 ? 1 : e <= 18 ? 2 : e <= 20 ? 3 : 4);

const BUCKETS = [
  { name: "Античность", turret: "Баллиста", special: "Дождь стрел", sky: ["#6fa8dc", "#cfe6f5"], ground: ["#8a6a3c", "#5a4326"], metal: ["#a9743a", "#d9b26a"] },
  { name: "Средневековье", turret: "Арбалетная башня", special: "Огненный дождь", sky: ["#7a8fb3", "#d6c9b0"], ground: ["#6e6a3e", "#47452a"], metal: ["#8a93a3", "#d3d9e3"] },
  { name: "Эпоха пороха", turret: "Пушка", special: "Артобстрел", sky: ["#8f9aa6", "#d8d3c4"], ground: ["#5d5a45", "#3a392c"], metal: ["#4a4f58", "#9aa0aa"] },
  { name: "Современность", turret: "Пулемётная турель", special: "Авиаудар", sky: ["#6b7c8c", "#b9c3c9"], ground: ["#4f5a45", "#323a2c"], metal: ["#586149", "#a6ad8c"] },
  { name: "Будущее", turret: "Лазерная турель", special: "Орбитальный удар", sky: ["#1d1640", "#5a3a8a"], ground: ["#2a2540", "#16132a"], metal: ["#3ad0e8", "#e8fbff"] },
];

const r = (x) => Math.round(x);

const ROLES = [

  { hp: 110, dmg: 11, range: 130, cd: 24, speed: 7, cost: 40, train: 45, rangeGrow: 0 },
  { hp: 65, dmg: 13, range: 700, cd: 30, speed: 6, cost: 55, train: 60, rangeGrow: 8 },
  { hp: 300, dmg: 26, range: 150, cd: 40, speed: 4, cost: 120, train: 105, rangeGrow: 0 },
];

const eras = ERAS.map(([name, names], e) => {
  const k = Math.pow(G, e);
  const ks = Math.pow(S, e);
  const b = bucketOf(e);
  const units = ROLES.map((ro, i) => {
    const cost = r(ro.cost * k);
    return {
      name: names[i],
      hp: r(ro.hp * ks),
      dmg: r(ro.dmg * ks),
      range: ro.range + ro.rangeGrow * e,
      cd: ro.cd,
      speed: ro.speed,
      cost,
      train: ro.train,
      reward: r(cost * 0.7),
      xp: cost,
    };
  });
  const meleeCost = units[0].cost, rangedCost = units[1].cost, heavyCost = units[2].cost;
  return {
    century: e + 1,
    roman: ROMAN[e + 1],
    name,
    bucket: b,
    units,
    turret: { name: BUCKETS[b].turret, dmg: r(15 * ks), range: 900, cd: 30, cost: r(140 * k), sell: r(140 * k * 0.6) },
    special: { name: BUCKETS[b].special, dmg: r(100 * ks) },
    baseMax: r(2600 * ks),
    xpNeed: r((meleeCost + rangedCost + heavyCost) * 2.4),
    income: r(7 * k * 1000),
  };
});

const data = {
  version: 1,
  tps: 30,
  lane: 6400,
  baseFront: [560, 5840],
  spacing: 140,
  startGold: 120,
  queueMax: 5,
  turretSlots: 3,
  specialCd: 1500,
  dmgVarPct: [92, 108],
  baseDmgPct: 300,

  counterPct: [
    [100, 160, 65],
    [85, 100, 160],
    [160, 65, 100],
  ],

  suddenTick: 25200,
  suddenPermille: 9,
  eras,
  buckets: BUCKETS,
};

writeFileSync(new URL("../data/game.json", import.meta.url), JSON.stringify(data, null, 1));
console.log("data/game.json:", eras.length, "эпох");
for (const e of [0, 6, 12, 18, 24]) {
  const x = eras[e];
  console.log(`${x.roman} ${x.name}: baseMax ${x.baseMax}, xpNeed ${x.xpNeed}, income/s ${x.income / 1000}, melee hp ${x.units[0].hp} dmg ${x.units[0].dmg} cost ${x.units[0].cost}`);
}
