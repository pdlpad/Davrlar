import { createFx, stockActor, PAL, hex, ramp } from "pixelfx/core/index.js";
import * as hitSpark from "pixelfx/fx/impacts/hit-spark.js";
import * as bluntHit from "pixelfx/fx/impacts/blunt-hit.js";
import * as critHit from "pixelfx/fx/impacts/crit-hit.js";
import * as bulletImpact from "pixelfx/fx/impacts/bullet-impact.js";
import * as groundSlam from "pixelfx/fx/impacts/ground-slam.js";
import * as arrow from "pixelfx/fx/projectiles/arrow.js";
import * as tracer from "pixelfx/fx/projectiles/bullet-tracer.js";
import * as laser from "pixelfx/fx/projectiles/laser-bolt.js";
import * as plasma from "pixelfx/fx/projectiles/plasma-shot.js";
import * as muzzle from "pixelfx/fx/projectiles/muzzle-pistol.js";
import * as poof from "pixelfx/fx/spawn/poof.js";
import * as coinDrop from "pixelfx/fx/spawn/coin-drop.js";
import * as dmgNum from "pixelfx/fx/ui/damage-number.js";
import * as levelUp from "pixelfx/fx/ui/level-up.js";
import * as smallExp from "pixelfx/fx/explosions/small-explosion.js";
import * as bigExp from "pixelfx/fx/explosions/big-explosion.js";
import * as nuke from "pixelfx/fx/explosions/nuke.js";
import * as meteor from "pixelfx/fx/fire/meteor.js";
import * as flamePillar from "pixelfx/fx/fire/flame-pillar.js";
import * as lightning from "pixelfx/fx/lightning/lightning-strike.js";
import * as smite from "pixelfx/fx/holy/smite.js";
import { GROUND_Y, W } from "./draw.js";

let seedCounter = 1;
const sd = () => (seedCounter = (seedCounter * 1103515245 + 12345) >>> 0) % 1000 + 1;

const LASER = [
  ramp(["#06203a", "#0a4a8a", "#2a8aff", "#7ad0ff", "#e8fbff"]),
  ramp(["#3a0a06", "#8a1a0a", "#ff4a2a", "#ffa07a", "#fff0e8"]),
];

export class FxManager {
  constructor() {
    this.items = [];
    this.shake = 0;
    this.flashes = new Map();
    this.baseFlash = [0, 0];
    this.recoil = new Map();
    this.lastNum = new Map();
    this.tickNow = 0;
    this.timers = [];
  }
  later(sec, fn) { this.timers.push({ t: sec, fn }); }
  count() { return this.items.length; }

  add(mod, opts) {
    if (this.items.length > 170) return null;
    const fx = createFx(mod, { seed: sd(), ...opts });
    this.items.push({ kind: "fx", fx });
    return fx;
  }

  projectile(mod, o, x0, y0, x1, y1, dur, arc, opts, onEnd) {
    if (this.items.length > 170) { onEnd?.(); return; }
    const fx = createFx(mod, { seed: sd(), x: x0, y: y0, flip: x1 < x0, ...opts });
    this.items.push({ kind: "proj", fx, x0, y0, x1, y1, t: 0, dur, arc, onEnd });
  }

  update(dt) {
    const due = [];
    this.timers = this.timers.filter((q) => { q.t -= dt; if (q.t <= 0) { due.push(q.fn); return false; } return true; });
    for (const f of due) f();
    for (const [k, v] of this.flashes) { const n = v - dt; if (n <= 0) this.flashes.delete(k); else this.flashes.set(k, n); }
    for (const [k, v] of this.recoil) { const n = v - dt; if (n <= 0) this.recoil.delete(k); else this.recoil.set(k, n); }
    for (let i = 0; i < 2; i++) this.baseFlash[i] = Math.max(0, this.baseFlash[i] - dt);
    this.shake = Math.max(0, this.shake - dt * 14);
    const keep = [];
    for (const it of this.items) {
      if (it.kind === "proj") {
        it.t += dt;
        const k = Math.min(1, it.t / it.dur);
        it.fx.options.x = it.x0 + (it.x1 - it.x0) * k;
        it.fx.options.y = it.y0 + (it.y1 - it.y0) * k - Math.sin(Math.PI * k) * it.arc;
        it.fx.update(dt);
        if (k >= 1) { it.onEnd?.(); continue; }
        keep.push(it);
      } else {
        it.fx.update(dt);
        if (!it.fx.done()) keep.push(it);
      }
    }
    this.items = keep;
  }

  safeDraw(frame, layer) {
    for (const it of this.items) {
      if (it.dead) continue;
      try { it.fx.draw(frame, layer); } catch (err) { it.dead = true; console.error("fx draw failed:", it.fx.meta?.name, err); }
    }
    if (this.items.some((i) => i.dead)) this.items = this.items.filter((i) => !i.dead);
  }
  drawBack(frame) { this.safeDraw(frame, "back"); }
  drawFront(frame) { this.safeDraw(frame, "front"); }

  handle(data, st, ev) {
    for (const e of ev) {
      switch (e.k) {
        case "spawn": this.onSpawn(data, e); break;
        case "hit": this.onHit(data, e); break;
        case "death": this.onDeath(data, e); break;
        case "evolve": this.onEvolve(data, e); break;
        case "turretBuilt": this.onTurretBuilt(data, e); break;
        case "special": this.onSpecial(data, e); break;
        case "siege": this.onSiege(data); break;
        case "win": this.onWin(data, st, e); break;
      }
    }
  }

  onSpawn(data, e) {
    const b = data.eras[e.era].bucket;
    this.add(poof, { x: e.x / 10, y: GROUND_Y + 2, palette: b === 4 ? PAL.cyber : b === 3 ? PAL.smoke : PAL.dust });
  }

  shooter(data, e) {

    return { x: e.from / 10 + (e.o === 0 ? 8 : -8), y: GROUND_Y - (e.role === 2 ? 20 : 16) };
  }

  onHit(data, e) {
    const b = data.eras[e.era].bucket;
    const tx = e.x / 10;
    const ty = e.tgt === "b" ? GROUND_Y - 24 - ((e.dmg * 7) % 20) : GROUND_Y - 12;
    const turret = e.role === -1;
    const ranged = e.role === 1 || turret;
    if (e.tgt === "u") this.flashes.set(e.id, 0.12);
    else this.baseFlash[1 - e.o] = 0.12;
    this.recoil.set(turret ? `t${e.o}${e.slot}` : `u${e.fid}`, 0.12);

    const impact = () => {
      if (b >= 2 && ranged) this.add(bulletImpact, { x: tx, y: ty, flip: e.o === 1 });
      else if (e.role === 2) this.add(bluntHit, { x: tx, y: ty + 4, flip: e.o === 1 });
      else this.add(hitSpark, { x: tx, y: ty, flip: e.o === 1 });
      this.number(data, e, tx, ty);
    };

    if (!ranged) { impact(); return; }
    const s = this.shooter(data, e);
    const dist = Math.abs(tx - s.x);
    if (b <= 1) {
      this.projectile(arrow, e.o, s.x, s.y, tx, ty, Math.max(0.12, dist / 420), 14, {}, impact);
    } else if (b <= 3) {
      this.add(muzzle, { x: s.x, y: s.y, flip: e.o === 1 });
      this.projectile(tracer, e.o, s.x, s.y, tx, ty, Math.max(0.06, dist / 900), 0, {}, impact);
    } else {
      this.projectile(turret ? plasma : laser, e.o, s.x, s.y, tx, ty, Math.max(0.08, dist / 700), 0, { palette: LASER[e.o] }, impact);
    }
  }

  number(data, e, x, y) {

    const key = e.tgt === "b" ? `b${e.o}` : `u${e.id}`;
    const now = performance.now();
    if (now - (this.lastNum.get(key) ?? 0) < (e.tgt === "b" ? 700 : 380)) return;
    this.lastNum.set(key, now);
    if (this.lastNum.size > 400) this.lastNum.clear();
    this.add(dmgNum, { x: x - 6, y: y - 10, text: "-" + short(e.dmg) });
  }

  onDeath(data, e) {
    const b = data.eras[e.era].bucket;
    const x = e.x / 10;
    if (e.role === 2 && b >= 3) this.add(smallExp, { x, y: GROUND_Y + 2, size: 0.55 });
    else this.add(poof, { x, y: GROUND_Y + 2, palette: b === 4 ? PAL.cyber : PAL.smoke });
    if (e.o === 1) this.add(coinDrop, { x, y: GROUND_Y + 2 });
    this.shake = Math.max(this.shake, e.role === 2 ? 1.2 : 0);
  }

  onEvolve(data, e) {
    const x = e.o === 0 ? 74 : W - 74;

    const o = { x, y: GROUND_Y - 30, text: "AGE " + data.eras[e.era].roman, actor: { ...stockActor(1), x: -9999, y: -9999 } };
    if (e.o === 1) o.palette = PAL.fire;
    this.add(levelUp, o);
    this.shake = Math.max(this.shake, 2);
  }

  onTurretBuilt(data, e) {
    const b = data.eras[e.era].bucket;
    this.add(poof, { x: 10 + e.slot * 14 + (e.o === 0 ? 0 : W - 2 * (10 + e.slot * 14)), y: GROUND_Y - 40, palette: b === 4 ? PAL.cyber : PAL.dust });
  }

  onSpecial(data, e) {
    const b = e.bucket;
    const hits = e.hit;
    this.shake = Math.max(this.shake, 6);
    const xs = hits.map((h) => h.x / 10);
    const spots = xs.length ? xs : [e.o === 0 ? W * 0.7 : W * 0.3];
    for (const h of hits.slice(0, 14)) this.add(dmgNum, { x: h.x / 10 - 6, y: GROUND_Y - 22, text: "-" + short(e.dmg) });
    const pick = (n) => {
      const out = [];
      const s = [...spots].sort((a, b) => a - b);
      for (let i = 0; i < Math.min(n, s.length); i++) out.push(s[Math.floor(((i + 0.5) / Math.min(n, s.length)) * s.length)]);
      return out;
    };
    if (b === 0) {

      const targets = spots.length < 6 ? [...spots, ...spots.map((x) => x - 14), ...spots.map((x) => x + 14)] : spots;
      targets.slice(0, 18).forEach((x, i) => {
        const delay = i * 0.03;
        this.later(delay, () => this.projectile(arrow, e.o, x - 70, 30 - (i % 3) * 14, x, GROUND_Y - 6, 0.5, 0, { angle: 0.95 }, () => this.add(hitSpark, { x, y: GROUND_Y - 8 })));
      });
    } else if (b === 1) {
      pick(3).forEach((x, i) => this.later(i * 0.12, () => this.add(meteor, { x, y: GROUND_Y + 4 })));
    } else if (b === 2) {
      pick(5).forEach((x, i) => this.later(i * 0.14, () => this.add(smallExp, { x, y: GROUND_Y + 6, size: 0.9 })));
    } else if (b === 3) {
      pick(3).forEach((x, i) => this.later(i * 0.22, () => this.add(bigExp, { x, y: GROUND_Y + 6 })));
    } else if (e.era === data.eras.length - 1) {
      const x = pick(1)[0];
      this.add(nuke, { x, y: GROUND_Y + 6 });
      this.shake = 10;
    } else {
      pick(7).forEach((x, i) => this.later(i * 0.09, () => this.add(lightning, { x, y: GROUND_Y + 4 })));
    }
  }

  onSiege(data) {
    this.shake = Math.max(this.shake, 2.2);
    for (const o of [0, 1]) {
      const x = o === 0 ? 34 : W - 34;
      this.add(smallExp, { x: x + (Math.random() - 0.5) * 30, y: GROUND_Y - 20, size: 0.5 });
      this.baseFlash[o] = 0.1;
    }
  }

  onWin(data, st, e) {
    const loser = e.winner === 2 ? -1 : 1 - e.winner;
    const sides = loser < 0 ? [0, 1] : [loser];
    for (const o of sides) {
      const x = o === 0 ? 30 : W - 30;
      this.add(bigExp, { x, y: GROUND_Y + 6 });
      [-28, 26, -8, 16].forEach((dx, i) => this.later(0.15 + i * 0.19, () => this.add(smallExp, { x: x + dx, y: GROUND_Y - 6 - i * 4, size: 0.8 })));
    }
    this.shake = 12;
  }
}

export function short(n) {
  if (n >= 1e6) return (n / 1e6).toFixed(1).replace(/\.0$/, "") + "M";
  if (n >= 1e4) return Math.round(n / 1e3) + "K";
  if (n >= 1e3) return (n / 1e3).toFixed(1).replace(/\.0$/, "") + "K";
  return String(n);
}
