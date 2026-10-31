import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { newGame, step, hashState } from "../core/sim.js";
import { aiCommands, AI_INCOME } from "../core/ai.js";

const data = JSON.parse(readFileSync(new URL("../data/game.json", import.meta.url), "utf8"));
const dir = new URL("../rust-core/tests/replays/", import.meta.url);
mkdirSync(dir, { recursive: true });

const CONFIGS = [
  [1, "normal", "normal"], [2, "easy", "hard"], [3, "hard", "normal"], [7, "normal", "easy"],
  [11, "hard", "hard"], [42, "easy", "easy"], [101, "normal", "hard"], [2026, "hard", "easy"],
];
const MAX = 30 * 60 * 25;
for (const [seed, L0, L1] of CONFIGS) {
  const inc = [AI_INCOME[L0], AI_INCOME[L1]];
  const st = newGame(data, seed, inc);
  const log = [];
  while (st.winner < 0 && st.tick < MAX) {
    const cmds = [...aiCommands(st, data, 0, L0), ...aiCommands(st, data, 1, L1)];
    if (cmds.length) log.push([st.tick + 1, cmds]);
    step(st, data, cmds);
  }
  const rep = { format: 1, game: "davrlar", seed, incomePct: inc, levels: [L0, L1], ticks: st.tick, winner: st.winner, finalHash: hashState(st), log };
  const name = `s${seed}-${L0}-${L1}.json`;
  writeFileSync(new URL(name, dir), JSON.stringify(rep));
  console.log(name.padEnd(24), `ticks ${st.tick}`.padEnd(12), `winner ${st.winner}`.padEnd(10), `hash ${rep.finalHash.toString(16).padStart(8, "0")}  cmds ${log.reduce((a, l) => a + l[1].length, 0)}`);
}

writeFileSync(new URL("../rust-core/data/game.json", import.meta.url), readFileSync(new URL("../data/game.json", import.meta.url)));
