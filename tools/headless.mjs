import { readFileSync } from "node:fs";
import { newGame, step, hashState } from "../core/sim.js";
import { aiCommands, AI_INCOME } from "../core/ai.js";

const data = JSON.parse(readFileSync(new URL("../data/game.json", import.meta.url), "utf8"));
const N = Number(process.argv[2] ?? 1);
const L0 = process.argv[3] ?? "normal", L1 = process.argv[4] ?? "normal";
const MAX_TICKS = 30 * 60 * 25;

function play(seed, record = false) {
  const st = newGame(data, seed, [AI_INCOME[L0], AI_INCOME[L1]]);
  const log = [];
  while (st.winner < 0 && st.tick < MAX_TICKS) {
    const cmds = [...aiCommands(st, data, 0, L0), ...aiCommands(st, data, 1, L1)];
    if (record && cmds.length) log.push([st.tick + 1, cmds]);
    step(st, data, cmds);
  }
  return { st, hash: hashState(st), log };
}

{
  const a = play(7, true), b = play(7);
  const st = newGame(data, 7, [AI_INCOME[L0], AI_INCOME[L1]]);
  let li = 0;
  while (st.winner < 0 && st.tick < MAX_TICKS) {
    let cmds = [];
    if (li < a.log.length && a.log[li][0] === st.tick + 1) cmds = a.log[li++][1];
    step(st, data, cmds);
  }
  const ok = a.hash === b.hash && a.hash === hashState(st);
  console.log(`детерминизм: ${ok ? "OK" : "ОШИБКА"}  hash=${a.hash.toString(16)}  тиков=${a.st.tick}  команд в журнале=${a.log.length}`);
  if (!ok) process.exit(1);
}

const res = [];
for (let s = 1; s <= N; s++) {
  const { st } = play(s);
  res.push({ seed: s, winner: st.winner, sec: Math.round(st.tick / 30), era0: st.players[0].era + 1, era1: st.players[1].era + 1 });
}
const w = [0, 0, 0];
for (const r of res) w[r.winner < 0 || r.winner === 2 ? 2 : r.winner]++;
const avg = (f) => (res.reduce((a, r) => a + f(r), 0) / res.length).toFixed(1);
console.log(`партий ${N}: побед стороны0(${L0}) ${w[0]}, стороны1(${L1}) ${w[1]}, ничьих/без исхода ${w[2]}`);
console.log(`средняя длительность ${avg((r) => r.sec)} с (${(avg((r) => r.sec) / 60).toFixed(1)} мин); эпоха к концу: ${avg((r) => r.era0)} / ${avg((r) => r.era1)}`);
const sorted = res.map((r) => r.sec).sort((a, b) => a - b);
console.log(`длительность: мин ${sorted[0]} с, медиана ${sorted[sorted.length >> 1]} с, макс ${sorted[sorted.length - 1]} с`);
if (N <= 5) for (const r of res) console.log(r);
