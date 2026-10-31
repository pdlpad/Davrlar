import { deflateSync } from "node:zlib";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const px = (p) => import(new URL(`../vendor/pixelfx/src/${p}`, import.meta.url));
const { createFrame, clearFrame, createFx, composite, stockActor, PAL } = await px("core/index.js");

const LASER_BLUE = ["#06203a", "#0a4a8a", "#2a8aff", "#7ad0ff", "#e8fbff"];
const LASER_RED = ["#3a0a06", "#8a1a0a", "#ff4a2a", "#ffa07a", "#fff0e8"];
const OFF = () => ({ ...stockActor(1), x: -9999, y: -9999 });

const SHEETS = [
  ["hit-spark", "impacts/hit-spark", 30],
  ["blunt-hit", "impacts/blunt-hit", 30],
  ["bullet-impact", "impacts/bullet-impact", 20],
  ["arrow", "projectiles/arrow", 30],
  ["tracer", "projectiles/bullet-tracer", 30],
  ["laser-blue", "projectiles/laser-bolt", 30, { palette: LASER_BLUE }],
  ["laser-red", "projectiles/laser-bolt", 30, { palette: LASER_RED }],
  ["plasma-blue", "projectiles/plasma-shot", 30, { palette: LASER_BLUE }],
  ["plasma-red", "projectiles/plasma-shot", 30, { palette: LASER_RED }],
  ["muzzle", "projectiles/muzzle-pistol", 20],
  ["poof-dust", "spawn/poof", 20, { palette: PAL.dust }],
  ["poof-smoke", "spawn/poof", 20, { palette: PAL.smoke }],
  ["poof-cyber", "spawn/poof", 20, { palette: PAL.cyber }],
  ["coin", "spawn/coin-drop", 15],
  ["level-gold", "ui/level-up", 15, { text: " ", actor: OFF() }],
  ["level-fire", "ui/level-up", 15, { text: " ", actor: OFF(), palette: PAL.fire }],
  ["explosion-s", "explosions/small-explosion", 12, { size: 0.55 }],
  ["explosion-m", "explosions/small-explosion", 12, { size: 0.9 }],
  ["explosion-l", "explosions/big-explosion", 10],
  ["nuke", "explosions/nuke", 8],
  ["meteor", "fire/meteor", 12],
  ["lightning", "lightning/lightning-strike", 15],
];

const CRC = new Int32Array(256).map((_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c;
});
function crc32(buf) {
  let c = -1;
  for (const b of buf) c = CRC[(c ^ b) & 255] ^ (c >>> 8);
  return (c ^ -1) >>> 0;
}
function chunk(type, body) {
  const out = Buffer.alloc(12 + body.length);
  out.writeUInt32BE(body.length, 0);
  out.write(type, 4, "latin1");
  body.copy(out, 8);
  out.writeUInt32BE(crc32(out.subarray(4, 8 + body.length)), 8 + body.length);
  return out;
}
function png(w, h, rgba) {
  const head = Buffer.alloc(13);
  head.writeUInt32BE(w, 0);
  head.writeUInt32BE(h, 4);
  head.set([8, 6, 0, 0, 0], 8);
  const raw = Buffer.alloc((w * 4 + 1) * h);
  for (let y = 0; y < h; y++) rgba.copy(raw, y * (w * 4 + 1) + 1, y * w * 4, (y + 1) * w * 4);
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", head), chunk("IDAT", deflateSync(raw, { level: 9 })), chunk("IEND", Buffer.alloc(0))]);
}

function bake(mod, fps, opts) {
  const meta = mod.meta;
  const [w, h] = meta.size, [ox, oy] = meta.origin;
  const frame = createFrame(w, h, -ox, -oy);
  const fx = createFx(mod, { seed: 7, x: 0, y: 0, ...opts });
  const count = Math.max(1, Math.round(meta.duration * fps));
  const frames = [];
  let x0 = w, y0 = h, x1 = -1, y1 = -1;
  for (let i = 0; i < count; i++) {
    fx.update(i === 0 ? 1e-4 : 1 / fps);
    clearFrame(frame);
    fx.draw(frame);
    const rgba = composite(frame, Buffer.alloc(w * h * 4), null);
    frames.push(rgba);
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
      if (rgba[(y * w + x) * 4 + 3] === 0) continue;
      if (x < x0) x0 = x;
      if (x > x1) x1 = x;
      if (y < y0) y0 = y;
      if (y > y1) y1 = y;
    }
  }
  const cw = x1 - x0 + 1, ch = y1 - y0 + 1;
  const cols = Math.ceil(Math.sqrt(count * ch / cw));
  const rows = Math.ceil(count / cols);
  const sheet = Buffer.alloc(cols * cw * rows * ch * 4);
  frames.forEach((rgba, i) => {
    const sx = (i % cols) * cw, sy = Math.floor(i / cols) * ch;
    for (let y = 0; y < ch; y++) rgba.copy(sheet, ((sy + y) * cols * cw + sx) * 4, ((y0 + y) * w + x0) * 4, ((y0 + y) * w + x0 + cw) * 4);
  });
  return { png: png(cols * cw, rows * ch, sheet), info: { w: cw, h: ch, cols, frames: count, fps, ox: ox - x0, oy: oy - y0, loop: !!meta.loop } };
}

const out = path.join(root, "game/assets/fx");
rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });
const manifest = {};
let total = 0;
for (const [name, file, fps, opts] of SHEETS) {
  const mod = await px(`fx/${file}.js`);
  const r = bake(mod, fps, opts ?? {});
  writeFileSync(path.join(out, `${name}.png`), r.png);
  manifest[name] = r.info;
  total += r.png.length;
  console.log(name.padEnd(14), `${r.info.w}x${r.info.h}`.padEnd(9), `${r.info.frames} кадров`.padEnd(12), (r.png.length / 1024).toFixed(0) + " KB");
}
writeFileSync(path.join(out, "manifest.json"), JSON.stringify(manifest, null, 1) + "\n");
console.log("итого", (total / 1048576).toFixed(1) + " MB");
