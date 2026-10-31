const LEVELS = {
  easy: { every: 45, slip: 35, queue: 1, turretAfter: 5400, pick: 10 },
  normal: { every: 20, slip: 12, queue: 2, turretAfter: 2700, pick: 65 },
  hard: { every: 12, slip: 0, queue: 2, turretAfter: 2700, pick: 100 },
};
export const AI_LEVELS = Object.keys(LEVELS);

export const AI_INCOME = { easy: 92, normal: 100, hard: 108 };

function h32(a, b) {
  let h = (Math.imul(a | 0, 0x9e3779b1) + Math.imul((b | 0) + 0x7f4a7c15, 0x85ebca6b)) | 0;
  h ^= h >>> 16; h = Math.imul(h, 0x85ebca6b);
  h ^= h >>> 13; h = Math.imul(h, 0xc2b2ae35);
  h ^= h >>> 16;
  return h >>> 0;
}

export function aiCommands(st, data, me, level = "normal") {
  const L = LEVELS[level] ?? LEVELS.normal;
  if (st.winner >= 0) return [];
  if ((st.tick + me * 3) % L.every !== 0) return [];
  const r = h32(st.tick ^ Math.imul(st.seed, 0x27d4eb2f), me + 1);
  if (L.slip && r % 100 < L.slip) return [];

  const p = st.players[me], q = st.players[1 - me];
  const E = data.eras[p.era];
  const cmds = [];

  if (p.era < data.eras.length - 1 && p.xp >= E.xpNeed) return [{ t: "evolve", p: me }];

  const myFront = data.baseFront[me];
  let near = [0, 0, 0], total = 0, nearBase = 0;
  for (const u of st.units) {
    if (u.o === me) continue;
    total++;
    near[u.role]++;
    if (Math.abs(u.x - myFront) < 2400) nearBase++;
  }

  if (p.spCd === 0 && (total >= 4 || (nearBase >= 2 && p.baseHp * 3 < p.baseMax))) cmds.push({ t: "special", p: me });

  if (st.tick >= L.turretAfter) {
    const slot = p.turrets.findIndex((t) => !t);
    if (slot >= 0 && p.gold >= E.turret.cost + E.units[0].cost * 2) cmds.push({ t: "buyTurret", p: me, slot });
    else if (slot < 0) {

      const old = p.turrets.findIndex((t) => t && t.era + 3 <= p.era);
      if (old >= 0 && p.gold >= E.turret.cost - data.eras[p.turrets[old].era].turret.sell) {
        cmds.push({ t: "sellTurret", p: me, slot: old });
        cmds.push({ t: "buyTurret", p: me, slot: old });
      }
    }
  }

  if (p.queue.length < L.queue) {
    let want;
    if (total === 0) want = r % 3 === 0 ? 1 : 0;
    else if ((r >>> 8) % 100 >= L.pick) want = (r >>> 16) % 3;
    else {

      let bestScore = -1 << 30;
      want = 0;
      for (let ro = 0; ro < 3; ro++) {
        let sc = 0;
        for (let e = 0; e < 3; e++) sc += near[e] * (data.counterPct[ro][e] - data.counterPct[e][ro]);
        if (sc > bestScore) { bestScore = sc; want = ro; }
      }
    }
    const dangerous = nearBase >= 3;
    if (E.units[want].cost > p.gold) {

      want = dangerous ? 0 : -1;
    }
    if (want >= 0 && E.units[want].cost <= p.gold) cmds.push({ t: "train", p: me, role: want });
  }
  return cmds;
}
