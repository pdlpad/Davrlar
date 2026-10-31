export const ROLE_MELEE = 0, ROLE_RANGED = 1, ROLE_HEAVY = 2;

function nextRng(s) {
  s ^= s << 13; s >>>= 0;
  s ^= s >>> 17;
  s ^= s << 5; s >>>= 0;
  return s >>> 0;
}
function rngU32(st) {
  st.rng = nextRng(st.rng);
  return st.rng;
}

export function newGame(data, seed = 1, incomePct = [100, 100]) {
  const e0 = data.eras[0];
  const mkPlayer = (o) => ({
    incPct: incomePct[o],
    gold: data.startGold, goldAcc: 0, xp: 0, era: 0,
    baseHp: e0.baseMax, baseMax: e0.baseMax,
    queue: [],
    turrets: Array.from({ length: data.turretSlots }, () => null),
    spCd: 600,
  });
  return {
    tick: 0,
    seed: seed | 0,
    rng: (seed >>> 0) || 1,
    nextId: 1,
    winner: -1,
    players: [mkPlayer(0), mkPlayer(1)],
    units: [],
  };
}

export const unitStats = (data, u) => data.eras[u.era].units[u.role];
const dirOf = (o) => (o === 0 ? 1 : -1);
const abs = (v) => (v < 0 ? -v : v);

function applyCommand(st, data, c, ev) {
  const p = st.players[c.p];
  if (!p) return;
  const E = data.eras[p.era];
  switch (c.t) {
    case "train": {
      if (c.role < 0 || c.role > 2) return;
      const us = E.units[c.role];
      if (p.queue.length >= data.queueMax || p.gold < us.cost) return;
      p.gold -= us.cost;
      p.queue.push({ role: c.role, era: p.era, t: us.train });
      break;
    }
    case "evolve": {
      if (p.era >= data.eras.length - 1 || p.xp < E.xpNeed) return;
      p.xp -= E.xpNeed;
      p.era += 1;
      const N = data.eras[p.era];
      p.baseHp += N.baseMax - p.baseMax;
      p.baseMax = N.baseMax;
      ev.push({ k: "evolve", o: c.p, era: p.era });
      break;
    }
    case "buyTurret": {
      if (c.slot < 0 || c.slot >= data.turretSlots || p.turrets[c.slot] || p.gold < E.turret.cost) return;
      p.gold -= E.turret.cost;
      p.turrets[c.slot] = { era: p.era, cd: 0 };
      ev.push({ k: "turretBuilt", o: c.p, slot: c.slot, era: p.era });
      break;
    }
    case "sellTurret": {
      const t = p.turrets[c.slot];
      if (!t) return;
      p.gold += data.eras[t.era].turret.sell;
      p.turrets[c.slot] = null;
      ev.push({ k: "turretSold", o: c.p, slot: c.slot });
      break;
    }
    case "special": {
      if (p.spCd > 0) return;
      p.spCd = data.specialCd;
      const dmg = E.special.dmg;
      const hit = [];
      for (const u of st.units) {
        if (u.o !== c.p) { u.hp -= dmg; hit.push({ id: u.id, x: u.x }); }
      }
      ev.push({ k: "special", o: c.p, era: p.era, bucket: E.bucket, dmg, hit });
      break;
    }
  }
}

export function step(st, data, cmds = []) {
  const ev = [];
  if (st.winner >= 0) return ev;
  st.tick += 1;

  for (const c of cmds) applyCommand(st, data, c, ev);

  for (let o = 0; o < 2; o++) {
    const p = st.players[o];

    p.goldAcc += ((((data.eras[p.era].income * p.incPct) / 100) | 0) / data.tps) | 0;
    if (p.goldAcc >= 1000) { p.gold += (p.goldAcc / 1000) | 0; p.goldAcc %= 1000; }
    if (p.spCd > 0) p.spCd -= 1;
  }

  for (let o = 0; o < 2; o++) {
    const p = st.players[o];
    if (!p.queue.length) continue;
    const q = p.queue[0];
    if (q.t > 0) q.t -= 1;
    if (q.t === 0) {
      const sx = data.baseFront[o];
      let blocked = false;
      for (const u of st.units) if (u.o === o && abs(u.x - sx) < data.spacing) { blocked = true; break; }
      if (!blocked) {
        p.queue.shift();
        const us = data.eras[q.era].units[q.role];
        const u = { id: st.nextId++, o, role: q.role, era: q.era, hp: us.hp, x: sx, cd: us.cd };
        st.units.push(u);
        ev.push({ k: "spawn", id: u.id, o, role: q.role, era: q.era, x: u.x });
      }
    }
  }

  const hits = [];
  for (const u of st.units) {
    if (u.cd > 0) u.cd -= 1;
    const us = data.eras[u.era].units[u.role];
    const dir = dirOf(u.o);

    let best = null, bd = 1 << 30;
    for (const e of st.units) {
      if (e.o === u.o) continue;
      const d = abs(e.x - u.x);
      if (d < bd) { bd = d; best = e; }
    }
    const baseX = data.baseFront[1 - u.o];
    const db = abs(baseX - u.x);
    let target = null;
    if (best && bd <= us.range) target = "u";
    else if (db <= us.range) target = "b";
    if (target) {
      if (u.cd === 0) {
        u.cd = us.cd;
        const v = data.dmgVarPct[0] + (rngU32(st) % (data.dmgVarPct[1] - data.dmgVarPct[0] + 1));
        let dmg = ((us.dmg * v) / 100) | 0;
        if (target === "u") dmg = ((dmg * data.counterPct[u.role][best.role]) / 100) | 0;
        hits.push({ tgt: target, id: target === "u" ? best.id : 0, o: u.o, dmg, from: u.x, role: u.role, era: u.era, fid: u.id });
      }
      continue;
    }

    let blocked = false;
    for (const v of st.units) {
      if (v === u || v.o !== u.o) continue;
      const ahead = (v.x - u.x) * dir;
      if (ahead > 0 && ahead < data.spacing) { blocked = true; break; }
    }
    if (!blocked) {
      u.x += dir * us.speed;
      const lim = baseX - dir * 60;
      if ((u.x - lim) * dir > 0) u.x = lim;
    }
  }

  for (let o = 0; o < 2; o++) {
    const p = st.players[o];
    const tx = data.baseFront[o];
    for (let s = 0; s < p.turrets.length; s++) {
      const t = p.turrets[s];
      if (!t) continue;
      if (t.cd > 0) { t.cd -= 1; continue; }
      const T = data.eras[t.era].turret;
      let best = null, bd = 1 << 30;
      for (const e of st.units) {
        if (e.o === o) continue;
        const d = abs(e.x - tx);
        if (d < bd) { bd = d; best = e; }
      }
      if (best && bd <= T.range) {
        t.cd = T.cd;
        const v = data.dmgVarPct[0] + (rngU32(st) % (data.dmgVarPct[1] - data.dmgVarPct[0] + 1));
        hits.push({ tgt: "u", id: best.id, o, dmg: ((T.dmg * v) / 100) | 0, from: tx, role: -1, era: t.era, fid: -(s + 1), slot: s });
      }
    }
  }

  for (const h of hits) {
    if (h.tgt === "u") {
      const e = st.units.find((x) => x.id === h.id);
      if (!e) continue;
      e.hp -= h.dmg;
      ev.push({ k: "hit", tgt: "u", id: e.id, fid: h.fid, x: e.x, dmg: h.dmg, o: h.o, from: h.from, role: h.role, era: h.era, slot: h.slot });
    } else {
      const p = st.players[1 - h.o];
      h.dmg = ((h.dmg * data.baseDmgPct) / 100) | 0;
      p.baseHp -= h.dmg;
      ev.push({ k: "hit", tgt: "b", id: 0, fid: h.fid, x: data.baseFront[1 - h.o], dmg: h.dmg, o: h.o, from: h.from, role: h.role, era: h.era, slot: h.slot });
    }
  }

  const alive = [];
  for (const u of st.units) {
    if (u.hp > 0) { alive.push(u); continue; }
    const us = data.eras[u.era].units[u.role];
    const killer = st.players[1 - u.o];
    killer.gold += us.reward;
    killer.xp += us.xp;
    ev.push({ k: "death", id: u.id, o: u.o, role: u.role, era: u.era, x: u.x });
  }
  st.units = alive;

  if (st.tick >= data.suddenTick && st.tick % data.tps === 0) {
    for (const p of st.players) p.baseHp -= ((p.baseMax * data.suddenPermille) / 1000) | 0;
    ev.push({ k: "siege" });
  }

  const d0 = st.players[0].baseHp <= 0, d1 = st.players[1].baseHp <= 0;
  if (d0 || d1) {
    for (const p of st.players) if (p.baseHp < 0) p.baseHp = 0;
    st.winner = d0 && d1 ? 2 : d0 ? 1 : 0;
    ev.push({ k: "win", winner: st.winner });
  }
  return ev;
}

export function hashState(st) {
  let h = 0x811c9dc5;
  const mix = (v) => {
    v = v | 0;
    for (let i = 0; i < 4; i++) {
      h ^= v & 255;
      h = Math.imul(h, 0x01000193);
      v >>= 8;
    }
  };
  mix(st.tick); mix(st.seed); mix(st.rng); mix(st.nextId); mix(st.winner);
  for (const p of st.players) {
    mix(p.incPct); mix(p.gold); mix(p.goldAcc); mix(p.xp); mix(p.era); mix(p.baseHp); mix(p.baseMax); mix(p.spCd);
    mix(p.queue.length);
    for (const q of p.queue) { mix(q.role); mix(q.era); mix(q.t); }
    for (const t of p.turrets) { if (t) { mix(1); mix(t.era); mix(t.cd); } else mix(0); }
  }
  mix(st.units.length);
  for (const u of st.units) { mix(u.id); mix(u.o); mix(u.role); mix(u.era); mix(u.hp); mix(u.x); mix(u.cd); }
  return h >>> 0;
}
