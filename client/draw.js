import { hex, mix, scale, rgb, WHITE, noise, clamp } from "pixelfx/core/index.js";

export const W = 640, H = 360;
export const GROUND_Y = 300;
export const TEAM = [hex("#3d7bff"), hex("#ff4a3d")];
const TEAM_DARK = [hex("#1c3f94"), hex("#8f1f1a")];
const SKIN = hex("#e8b98a");
const INK = hex("#0d0b14");

let F = null;
export const setFrame = (f) => { F = f; };

export function px(x, y, c) {
  x |= 0; y |= 0;
  if (x < 0 || y < 0 || x >= W || y >= H) return;
  F.color[y * W + x] = c;
}
export function rect(x, y, w, h, c) {
  x |= 0; y |= 0;
  for (let j = 0; j < h; j++) {
    const yy = y + j;
    if (yy < 0 || yy >= H) continue;
    for (let i = 0; i < w; i++) {
      const xx = x + i;
      if (xx >= 0 && xx < W) F.color[yy * W + xx] = c;
    }
  }
}

export function blend(x, y, c, a) {
  x |= 0; y |= 0;
  if (x < 0 || y < 0 || x >= W || y >= H) return;
  const i = y * W + x;
  F.color[i] = mix(F.color[i], c, a);
}
function line(x0, y0, x1, y1, c) {
  x0 |= 0; y0 |= 0; x1 |= 0; y1 |= 0;
  const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0), sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1;
  let e = dx + dy;
  for (;;) {
    px(x0, y0, c);
    if (x0 === x1 && y0 === y1) break;
    const e2 = 2 * e;
    if (e2 >= dy) { e += dy; x0 += sx; }
    if (e2 <= dx) { e += dx; y0 += sy; }
  }
}

const bgCache = new Map();
export function backgroundFor(data, era) {
  let bg = bgCache.get(era);
  if (bg) return bg;
  const E = data.eras[era], B = data.buckets[E.bucket];

  const t = era / (data.eras.length - 1);
  const top = mix(hex(B.sky[0]), hex("#0b0720"), t > 0.84 ? (t - 0.84) * 2.2 : 0);
  const bot = hex(B.sky[1]);
  const ground = hex(B.ground[0]), groundDark = hex(B.ground[1]);
  bg = new Uint32Array(W * H);
  const night = E.bucket === 4;
  const horizon = 232;
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      let c;
      if (y < horizon) {
        const k = y / horizon;

        const bayer = ((x & 3) * 4 + (y & 3)) / 16;
        const kk = clamp(Math.floor(k * 14 + bayer) / 14);
        c = mix(top, bot, kk);
      } else c = ground;
      bg[y * W + x] = c;
    }
  }
  const put = (x, y, c) => { if (x >= 0 && x < W && y >= 0 && y < H) bg[y * W + x] = c; };

  if (night) {
    for (let i = 0; i < 90; i++) {
      const x = ((noise(i, 7) * 0.5 + 0.5) * W) | 0, y = ((noise(i, 19) * 0.5 + 0.5) * 150) | 0;
      put(x, y, mix(hex("#bfb8ff"), WHITE, (i % 3) / 3));
    }

    const px0 = 500, py0 = 70;
    for (let y = -26; y <= 26; y++) for (let x = -26; x <= 26; x++) {
      if (x * x + y * y <= 26 * 26) put(px0 + x, py0 + y, mix(hex("#6a4fc8"), hex("#f0a8ff"), clamp((x + y + 40) / 80)));
    }
  } else {
    for (let i = 0; i < 7; i++) {
      const cx = ((noise(i, 3) * 0.5 + 0.5) * W) | 0, cy = 30 + (((noise(i, 5) * 0.5 + 0.5) * 70) | 0);
      const w = 30 + (i % 3) * 14;
      for (let dy = -3; dy <= 3; dy++) for (let dx = -w; dx <= w; dx++) {
        if ((dx * dx) / (w * w) + (dy * dy) / 12 <= 1) put(cx + dx, cy + dy, mix(bot, WHITE, 0.55));
      }
    }

    const sx = 120, sy = 64;
    for (let y = -14; y <= 14; y++) for (let x = -14; x <= 14; x++) if (x * x + y * y <= 196) put(sx + x, sy + y, hex(E.bucket === 3 ? "#f4f0d8" : "#fff2b0"));
  }

  const far = mix(bot, top, 0.35), near = mix(ground, hex(B.sky[0]), 0.4);
  for (let x = 0; x < W; x++) {
    const h1 = 40 + noise(x * 0.012, era * 0.37 + 1) * 34 + noise(x * 0.05, 4) * 8;
    for (let y = horizon - h1 | 0; y < horizon; y++) put(x, y, far);
    const h2 = 18 + noise(x * 0.02, era * 0.21 + 9) * 16 + noise(x * 0.09, 2) * 4;
    for (let y = horizon - h2 | 0; y < horizon; y++) put(x, y, near);
  }

  for (let y = horizon; y < H; y++) {
    const depth = (y - horizon) / (H - horizon);
    for (let x = 0; x < W; x++) {
      const n = noise(x * 0.35, y * 0.9 + era) * 0.5 + 0.5;
      let c = mix(ground, groundDark, depth * 0.8);
      if (n > 0.78) c = scale(c, 1.12);
      else if (n < 0.2) c = scale(c, 0.88);
      put(x, y, c);
    }
  }

  const road = mix(ground, hex("#d8c9a0"), E.bucket === 4 ? 0 : 0.22);
  for (let y = 292; y < 318; y++) for (let x = 0; x < W; x++) {
    const edge = y === 292 || y === 317;
    put(x, y, edge ? scale(road, 0.7) : mix(road, groundDark, 0.15 + ((x + y) % 7 === 0 ? 0.12 : 0)));
  }
  if (E.bucket === 4) for (let x = 0; x < W; x += 16) put(x, 305, hex("#4af0ff"));
  bgCache.set(era, bg);
  return bg;
}

export function baseGeom(era) {
  return { w: 56, h: 56 + Math.min(era, 24) };
}
export function drawBase(data, o, era, hpFrac, flash) {
  const E = data.eras[era], b = E.bucket, { w, h } = baseGeom(era);
  const left = o === 0;
  const X = (lx) => (left ? lx : W - 1 - lx);
  const rr = (lx, ly, ww, hh, c) => {
    if (flash > 0) c = mix(c, WHITE, flash);
    const x0 = left ? lx : W - lx - ww;
    rect(x0, GROUND_Y + 4 - ly, ww, hh, c);
  };
  const team = TEAM[o], teamD = TEAM_DARK[o];
  const stone = [hex("#cfc6b0"), hex("#9d947e"), hex("#6f6858")];
  const dmg = hpFrac < 0.5;
  if (b === 0) {

    rr(0, 34, 52, 34, stone[0]); rr(0, 34, 52, 3, stone[1]); rr(0, 4, 52, 4, stone[2]);
    for (let i = 0; i < 4; i++) { rr(5 + i * 12, 30, 5, 28, stone[1]); rr(5 + i * 12, 30, 2, 28, stone[0]); }
    rr(-2, 38, 56, 5, hex("#b9602a"));
    rr(22, 14, 10, 14, hex("#2a2118"));
    rr(46, h - 6 + 6, 2, 18, hex("#5a3a1c")); rr(36, h + 6, 10, 8, team);
  } else if (b === 1) {

    rr(8, 44, 40, 40, stone[1]); rr(8, 44, 40, 2, stone[0]);
    rr(0, 54, 14, 54, stone[0]); rr(38, 54, 14, 54, stone[0]);
    for (const tx of [0, 6, 38, 44]) rr(tx, 62, 4, 5, stone[0]);
    for (let i = 0; i < 3; i++) rr(16 + i * 10, 48, 5, 4, stone[0]);
    rr(22, 20, 12, 20, hex("#2a2118")); rr(22, 22, 12, 3, hex("#6a4a2a"));
    rr(24, h + 2, 2, 14, hex("#4a3820")); rr(26, h + 2, 12, 9, team);
    for (const wy of [40, 48]) rr(3, wy, 3, 5, hex("#1e1a28"));
  } else if (b === 2) {

    rr(0, 34, 54, 34, hex("#8a7e66")); rr(0, 34, 54, 3, hex("#b4a88c"));
    for (let i = 0; i < 5; i++) rr(2 + i * 10, 38, 6, 4, hex("#b4a88c"));
    rr(20, 18, 14, 16, hex("#2a2118"));
    rr(40, 46, 12, 12, hex("#6a5e48")); rr(46, 52, 12, 3, hex("#2a2d34"));
    rr(8, h + 4, 2, 18, hex("#3a2a18")); rr(10, h + 4, 14, 9, team);
  } else if (b === 3) {

    rr(0, 30, 54, 30, hex("#7d8478")); rr(0, 30, 54, 3, hex("#a6ad9c"));
    rr(4, 44, 22, 14, hex("#6c7368")); rr(8, 40, 14, 4, hex("#1e2420"));
    rr(30, 44, 22, 14, hex("#6c7368")); rr(34, 40, 14, 4, hex("#1e2420"));
    rr(38, h + 14, 2, 26, hex("#bfc4b4")); rr(32, h + 8, 14, 2, hex("#bfc4b4"));
    rr(8, h + 4, 2, 16, hex("#3a3a30")); rr(10, h + 4, 12, 8, team);
    rr(22, 14, 12, 12, hex("#1a1d18"));
  } else {

    rr(6, 16, 40, 16, hex("#26304a")); rr(6, 16, 40, 2, hex("#4af0ff"));
    rr(14, 40, 24, 28, hex("#324260")); rr(14, 40, 3, 28, hex("#5a74a6"));
    rr(20, 56, 12, 20, hex("#3a4e78")); rr(24, 58, 4, 18, hex("#4af0ff"));
    rr(24, h + 12, 4, 20, hex("#7a8cc0"));
    rr(20, h + 18, 12, 4, team);
    rr(-4, 4, 60, 4, hex("#12182a"));
  }

  if (b >= 3) rr(2, 8, 4, 3, team);
  rr(0, 4, 3, 3, teamD);
  if (dmg) {
    for (let i = 0; i < 7; i++) rr(6 + i * 7, 8 + (i * 9) % 30, 2, 5 + (i % 3), hex("#1a1612"));
  }
  return { top: GROUND_Y + 4 - h, X };
}

export function turretPos(o, slot, era) {
  const { h } = baseGeom(era);
  const lx = 10 + slot * 14;
  const x = o === 0 ? lx : W - 1 - lx;
  return { x, y: GROUND_Y + 4 - h - 2 };
}
export function drawTurret(data, o, slot, tEra, baseEra, recoil) {
  const B = data.eras[tEra].bucket;
  const p = turretPos(o, slot, baseEra);
  const d = o === 0 ? 1 : -1;
  const x = p.x, y = p.y;
  const metal = [hex("#a07040"), hex("#7e8aa0"), hex("#4f535c"), hex("#59624a"), hex("#26304a")][B];
  const hi = [hex("#d9b26a"), hex("#d3d9e3"), hex("#9aa0aa"), hex("#a6ad8c"), hex("#4af0ff")][B];
  rect(x - 4, y - 5, 9, 6, metal); rect(x - 4, y - 5, 9, 1, hi);
  rect(x - 1, y - 8, 3, 3, TEAM[o]);
  const r = recoil > 0 ? -2 : 0;
  if (B === 0) { line(x, y - 8, x + d * (8 + r), y - 10, hex("#4a3820")); px(x + d * (8 + r), y - 10, hex("#d9d9d9")); }
  else if (B === 1) { rect(x + d * 2, y - 8, 7 + r, 2, hex("#4a3820")); }
  else if (B === 2) { rect(x + d * 2 - (d < 0 ? 8 : 0), y - 8, 8 + r, 3, hex("#2a2d34")); }
  else if (B === 3) { rect(x + d * 2 - (d < 0 ? 9 : 0), y - 8, 9 + r, 2, hex("#2a2d34")); px(x + d * 11, y - 8, hex("#ffe28a")); }
  else { rect(x + d * 2 - (d < 0 ? 9 : 0), y - 8, 9 + r, 2, hi); px(x + d * 11, y - 8, WHITE); }
}

const PAL_BUCKET = [
  { metal: hex("#c9a45c"), metalD: hex("#8a6a36"), cloth: hex("#c8b38a"), dark: hex("#4a3820") },
  { metal: hex("#b9c3d3"), metalD: hex("#6f7a8c"), cloth: hex("#8a6a4a"), dark: hex("#2a2d38") },
  { metal: hex("#8a8f99"), metalD: hex("#4f535c"), cloth: hex("#e8e2cf"), dark: hex("#2a2118") },
  { metal: hex("#8f9a74"), metalD: hex("#586149"), cloth: hex("#6b7355"), dark: hex("#232a1e") },
  { metal: hex("#c8e8ff"), metalD: hex("#4a6aa0"), cloth: hex("#2c3558"), dark: hex("#0e1224") },
];

export function drawUnit(data, u, cx, dy, dir, anim) {
  const E = data.eras[u.era], b = E.bucket, P = PAL_BUCKET[b];
  const team = TEAM[u.o], teamD = TEAM_DARK[u.o];
  const fl = anim.flash;
  const C = (c) => (fl > 0 ? mix(c, WHITE, fl) : c);
  const gy = GROUND_Y + dy;
  const X = (lx) => cx + lx * dir;
  const R = (lx, ly, w, h, c) => {
    const x0 = dir > 0 ? cx + lx : cx - lx - w + 1;
    rect(x0, gy - ly, w, h, C(c));
  };
  const atk = anim.atk;
  const step = anim.walk & 7;
  const lift = step < 4 ? step : 7 - step;

  for (let i = -5; i <= 5; i++) blend(cx + i, gy + 1, INK, 0.35);

  if (u.role === 2 && b === 3) {

    R(-11, 4, 24, 4, P.dark);
    for (let i = 0; i < 6; i++) px(X(-9 + i * 4), gy - 2 - ((step + i) & 1), C(P.metalD));
    R(-10, 9, 22, 5, P.metal); R(-10, 9, 22, 1, scale(P.metal, 1.2));
    R(-4, 13, 11, 4, P.metalD);
    const rec = atk > 0.5 ? -2 : 0;
    R(6 + rec, 12, 14, 2, P.dark);
    R(-9, 8, 3, 2, team);
    return;
  }
  if (u.role === 2 && b === 4) {

    const bob = lift > 1 ? 1 : 0;
    R(-7, 9 - bob, 3, 9, P.metalD); R(4, 9 - (1 - bob), 3, 9, P.metalD);
    R(-9, 2, 7, 2, P.dark); R(2, 2, 7, 2, P.dark);
    R(-8, 20 - bob, 17, 10, P.metal); R(-8, 20 - bob, 17, 2, scale(P.metal, 1.15));
    R(-3, 25 - bob, 9, 5, hex("#101830")); R(-2, 24 - bob, 7, 2, hex("#4af0ff"));
    R(6, 17 - bob, 14, 3, P.metalD);
    const g = atk > 0.5 ? WHITE : hex("#4af0ff");
    R(20, 17 - bob, 2, 3, g);
    R(-8, 15 - bob, 4, 2, team);
    return;
  }

  const heavy = u.role === 2;
  const bw = heavy ? 8 : 6, bh = heavy ? 8 : 6;
  const legH = heavy ? 5 : 5;
  const hy = legH + bh;

  const l1 = lift > 1 ? 1 : 0, l2 = lift > 1 ? 0 : 1;
  R(-3, legH - l1, 2, legH - l1, P.dark);
  R(1, legH - l2, 2, legH - l2, P.dark);

  const bodyCol = b === 0 ? (heavy ? P.metal : team) : b === 1 ? (heavy ? P.metal : team) : b === 2 ? (u.role === 1 ? P.cloth : team) : b === 3 ? P.cloth : P.cloth;
  R(-bw / 2 | 0, hy, bw, bh, bodyCol);
  if (b <= 1 && !heavy) R((-bw / 2 | 0) + 1, hy - 2, bw - 2, 2, P.metal);
  if (b >= 3) R(-bw / 2 | 0, hy - 1, bw, 1, team);
  if (b === 4) R(-1, hy - 2, 3, 2, hex("#4af0ff"));
  if (heavy && b < 3) R(-bw / 2 | 0, hy, bw, 1, scale(P.metal, 1.2));

  R(-2, hy + 5, 5, 5, SKIN);
  if (b === 0) { R(-3, hy + 6, 7, 3, P.metal); if (heavy) R(-1, hy + 10, 3, 3, team); }
  else if (b === 1) { R(-3, hy + 6, 7, 6, P.metal); R(1, hy + 3, 2, 1, hex("#1e1a28")); if (heavy) R(-1, hy + 12, 2, 3, team); }
  else if (b === 2) { R(-3, hy + 6, 7, 2, u.role === 1 ? P.dark : P.metal); if (u.role === 1) R(-4, hy + 4, 9, 1, P.dark); }
  else if (b === 3) { R(-3, hy + 6, 7, 3, P.metal); }
  else { R(-3, hy + 6, 7, 4, P.metal); R(0, hy + 4, 4, 2, hex("#4af0ff")); }

  const reach = Math.round(atk * 4);
  if (u.role === 0) {
    if (b === 0) { R(3, hy - 1, 3, 7, team); R(3, hy - 1, 1, 7, P.metalD); R(6 + reach, hy - 2, 7, 1, hex("#e8ecf2")); }
    else if (b === 1) { R(3, hy - 1, 3, 7, team); R(6 + reach, hy - 1, 12, 1, hex("#d3d9e3")); R(5 + reach, hy - 2, 2, 3, P.metalD); }
    else if (b === 2) { R(2 + reach, hy - 1, 15, 1, P.dark); R(17 + reach, hy - 1, 3, 1, hex("#d3d9e3")); }
    else if (b === 3) { R(2 + reach / 2 | 0, hy - 1, 10, 2, P.dark); R(11, hy - 1, 2, 1, P.metalD); }
    else { R(2 + reach / 2 | 0, hy - 1, 10, 2, P.metalD); R(10, hy - 1, 5, 1, hex("#4af0ff")); }
  } else if (u.role === 1) {
    if (b <= 1) {
      for (let i = 0; i < 9; i++) R(5 + (i < 4 ? i : 8 - i) / 2 | 0, hy + 3 - i, 1, 1, P.dark);
      R(5, hy - 1, 1, 9, hex("#ece6d2"));
      if (atk < 0.5) R(6, hy + 3, 5, 1, hex("#9a6a3a"));
    } else if (b === 2) { R(2, hy + 1, 12, 1, P.dark); R(2, hy, 5, 2, hex("#6a4a2a")); if (atk > 0.5) R(14, hy + 2, 3, 3, hex("#ffe28a")); }
    else if (b === 3) { R(2, hy + 1, 11, 2, P.dark); R(11, hy + 2, 2, 1, P.metalD); if (atk > 0.5) R(13, hy + 1, 2, 3, hex("#ffe28a")); }
    else { R(2, hy + 1, 11, 2, P.metalD); R(11, hy + 1, 3, 2, hex("#4af0ff")); if (atk > 0.5) R(14, hy + 1, 2, 2, WHITE); }
  } else {

    if (b === 0) { R(4, hy + 2, 4, 10, team); R(4, hy + 2, 1, 10, P.metalD); R(8 + reach, hy - 1, 14, 1, P.dark); R(22 + reach, hy - 1, 3, 1, hex("#e8ecf2")); }
    else if (b === 1) { R(4, hy + 1, 3, 9, team); R(7 + reach, hy + 2, 3, 1, P.metalD); R(8 + reach, hy - 2, 2, 14, hex("#e8ecf2")); }
    else { R(3, hy, 3, 5, P.metalD); R(6 + reach, hy - 1, 14, 1, hex("#d3d9e3")); R(17, hy + 7, 4, 3, team); }
  }
}

export function drawHpBar(cx, topY, frac, o) {
  const w = 12, x = (cx - w / 2) | 0;
  rect(x - 1, topY - 1, w + 2, 3, INK);
  rect(x, topY, w, 1, hex("#3a1a1a"));
  rect(x, topY, Math.max(1, Math.round(w * frac)), 1, frac > 0.5 ? hex("#59e07a") : frac > 0.25 ? hex("#f4c542") : hex("#ff5a4a"));
}
export { INK, SKIN, rgb };
