import { createFrame, clearFrame, composite } from "pixelfx/core/index.js";
import data from "../data/game.json";
import { newGame, step, hashState } from "../core/sim.js";
import { aiCommands, AI_INCOME } from "../core/ai.js";
import { W, H, GROUND_Y, setFrame, backgroundFor, drawBase, drawTurret, drawUnit, drawHpBar, baseGeom, TEAM } from "./draw.js";
import { FxManager, short } from "./fx.js";
import { LANGS, LANG_LABEL, detectLang, setLang, getLang, t, century, centuryHtml, eraName, unitName, turretName, specialName } from "./i18n.js";

const $ = (s) => document.querySelector(s);
const el = (tag, cls, html) => { const e = document.createElement(tag); if (cls) e.className = cls; if (html != null) e.innerHTML = html; return e; };

const canvas = $("#cv");
canvas.width = W; canvas.height = H;
const ctx = canvas.getContext("2d");
const img = new ImageData(W, H);
const frame = createFrame(W, H);
const fxm = new FxManager();

const LEVELS = ["easy", "normal", "hard"];
const G = {
  mode: "menu",
  st: null, level: "normal", auto: false, demo: true,
  speed: 1, paused: false, acc: 0, pending: [], log: [], seed: 1,
  banner: null,
};
const store = { get: (k, d) => { try { return localStorage.getItem(k) ?? d; } catch { return d; } }, set: (k, v) => { try { localStorage.setItem(k, v); } catch {} } };
G.level = store.get("davrlar.level", "normal");
setLang(detectLang(store.get("davrlar.lang", null)));

function startGame({ demo = false, auto = false, level = G.level } = {}) {
  G.seed = (Math.random() * 0x7fffffff) | 0 || 1;
  G.level = level; G.demo = demo; G.auto = auto || demo;
  const inc = [G.auto ? AI_INCOME[level] : 100, AI_INCOME[level]];
  G.st = newGame(data, G.seed, inc);
  G.inc = inc; G.pending = []; G.log = []; G.acc = 0; G.paused = false;
  G.mode = demo ? "menu" : "play";
  fxm.items = []; fxm.timers = [];
  hideOverlays();
  if (!demo) { G.speed = 1; syncSpeed(); }
  else G.speed = 2;
  $("#menu").classList.toggle("hidden", !demo);
  if (!demo) store.set("davrlar.level", level);
}

function stepOnce() {
  const st = G.st;
  const cmds = G.pending.splice(0);
  if (G.auto) cmds.push(...aiCommands(st, data, 0, G.level));
  cmds.push(...aiCommands(st, data, 1, G.level));
  const eraBefore = [st.players[0].era, st.players[1].era];
  const ev = step(st, data, cmds);
  if (cmds.length && !G.demo) G.log.push([st.tick, cmds]);
  fxm.handle(data, st, ev);
  for (const e of ev) onEvent(e, eraBefore);
  if (st.winner >= 0 && G.mode === "play") finish();
}

function onEvent(e) {
  if (G.demo) return;
  if (e.k === "evolve") {
    const E = data.eras[e.era];
    toast(`${e.o === 0 ? "" : t("enemy")}${century(E.roman)}`, e.o === 0 ? "good" : "bad");
  } else if (e.k === "special") {
    toast(`${e.o === 0 ? "" : t("enemy")}${specialName(e.bucket)}`, e.o === 0 ? "good" : "bad");
  } else if (e.k === "siege" && G.st.tick === data.suddenTick) {
    toast(t("siege"), "bad");
  }
}

function finish() {
  G.mode = "over";
  const st = G.st, w = st.winner;
  const finalHash = hashState(st);
  G.replay = { format: 1, game: "davrlar", seed: G.seed, incomePct: G.inc, level: G.level, autoPlayer0: G.auto, ticks: st.tick, winner: w, finalHash, log: G.log };
  const mins = Math.floor(st.tick / 30 / 60), secs = Math.floor(st.tick / 30) % 60;
  const title = w === 2 ? t("draw") : G.auto ? (w === 0 ? t("blueWin") : t("redWin")) : w === 0 ? t("win") : t("lose");
  $("#over h2").textContent = title;
  $("#over h2").className = w === 0 && !G.auto ? "win" : w === 1 && !G.auto ? "lose" : "";
  const p0 = st.players[0], p1 = st.players[1];
  $("#over .stats").innerHTML = `${mins}:${String(secs).padStart(2, "0")} · ${century(`${data.eras[p0.era].roman} / ${data.eras[p1.era].roman}`)}`;
  setTimeout(() => $("#over").classList.remove("hidden"), 1800);
}

function cmd(c) {
  if (G.mode !== "play" || G.auto || G.paused) return;
  G.pending.push({ ...c, p: 0 });
}
const train = (role) => cmd({ t: "train", role });
const evolve = () => cmd({ t: "evolve" });
const special = () => cmd({ t: "special" });
const slotClick = (slot) => {
  const p = G.st.players[0];
  cmd({ t: p.turrets[slot] ? "sellTurret" : "buyTurret", slot });
};

const top = $("#top"), bottom = $("#bottom");
function buildHud() {

  bottom.innerHTML = "";
  const res = el("div", "panel res", `
    <div class="gold"><span class="coin"></span><b id="gold">0</b><small id="inc"></small></div>
    <div class="xpwrap"><div class="xp"><i id="xpbar"></i><span id="xptext"></span></div>
    <button id="evolve" class="btn evo">${t("evolve")} <kbd>E</kbd></button></div>`);
  const tr = el("div", "panel train");
  for (let r = 0; r < 3; r++) {
    const b = el("button", "btn unit", `<span class="role">${t("roles")[r]}</span><b class="uname"></b><span class="cost"></span><kbd>${r + 1}</kbd><i class="cd"></i>`);
    b.id = "tr" + r;
    b.onclick = () => train(r);
    tr.append(b);
  }
  const queue = el("div", "queue"); queue.id = "queue";
  tr.append(queue);
  const sp = el("div", "panel spec");
  const turrets = el("div", "turrets");
  for (let s = 0; s < data.turretSlots; s++) {
    const b = el("button", "btn turret", ""); b.id = "tu" + s; b.onclick = () => slotClick(s);
    turrets.append(b);
  }
  const sb = el("button", "btn special", `<span class="sname"></span><span class="scd"></span><kbd>Q</kbd><i class="cd"></i>`);
  sb.id = "special"; sb.onclick = special;
  sp.append(turrets, sb);
  const ctl = el("div", "panel ctl", `
    <button class="btn sm" id="pause">❚❚</button>
    <button class="btn sm" id="spd">×1</button>
    <button class="btn sm" id="autob">${t("auto")}</button>
    <button class="btn sm" id="menub">${t("menu")}</button>`);
  bottom.append(res, tr, sp, ctl);
  $("#evolve").onclick = evolve;
  $("#pause").onclick = togglePause;
  $("#spd").onclick = () => { G.speed = G.speed === 1 ? 2 : G.speed === 2 ? 4 : 1; syncSpeed(); };
  $("#autob").classList.toggle("on", G.auto && !G.demo);
  $("#autob").onclick = () => { if (G.mode === "play") { G.auto = !G.auto; $("#autob").classList.toggle("on", G.auto); toast(G.auto ? t("autoOn") : t("autoOff")); } };
  $("#menub").onclick = () => { G.paused = true; openMenu(); };

  top.innerHTML = `
    <div class="side p0"><div class="eraname" id="e0"></div><div class="bar"><i id="hp0"></i><span id="hpt0"></span></div></div>
    <div class="mid"><div class="timeline" id="tl"></div><div class="clock" id="clock"></div></div>
    <div class="side p1"><div class="eraname" id="e1"></div><div class="bar"><i id="hp1"></i><span id="hpt1"></span></div></div>`;
  const tl = $("#tl");
  for (let i = 0; i < data.eras.length; i++) tl.append(el("span", "dot", ""));
}
function syncSpeed() { $("#spd").textContent = "×" + G.speed; }
function togglePause() { if (G.mode === "play") { G.paused = !G.paused; $("#pause").classList.toggle("on", G.paused); $("#paused").classList.toggle("hidden", !G.paused); } }

let hudCache = {};
function setText(id, v) { if (hudCache[id] !== v) { hudCache[id] = v; $(id).textContent = v; } }
function setHtml(id, v) { if (hudCache[id] !== v) { hudCache[id] = v; $(id).innerHTML = v; } }
function updateHud() {
  const st = G.st; if (!st) return;
  const p = st.players[0], q = st.players[1];
  const E = data.eras[p.era], Q = data.eras[q.era];
  setText("#gold", short(p.gold));
  setText("#inc", `+${short(Math.round(E.income * p.incPct / 100000))}${t("perSec")}`);
  const need = E.xpNeed, last = p.era >= data.eras.length - 1;
  $("#xpbar").style.width = (last ? 100 : Math.min(100, (p.xp * 100) / need)) + "%";
  setText("#xptext", last ? "MAX" : `${short(p.xp)} / ${short(need)}`);
  const canEvo = !last && p.xp >= need;
  $("#evolve").disabled = !canEvo || G.auto; $("#evolve").classList.toggle("ready", canEvo && !G.auto);
  for (let r = 0; r < 3; r++) {
    const u = E.units[r], b = $("#tr" + r);
    setText(`#tr${r} .uname`, unitName(p.era, r));
    setText(`#tr${r} .cost`, short(u.cost));
    b.disabled = G.auto || p.gold < u.cost || p.queue.length >= data.queueMax;
  }

  setHtml("#queue", p.queue.map((it, i) => `<span class="qi r${it.role}" title="${unitName(it.era, it.role)}">${i === 0 ? `<i style="width:${100 - (it.t * 100) / data.eras[it.era].units[it.role].train}%"></i>` : ""}</span>`).join(""));

  for (let s = 0; s < data.turretSlots; s++) {
    const tu = p.turrets[s], b = $("#tu" + s);
    if (tu) { b.innerHTML = `<span class="tn">${turretName(data.eras[tu.era].bucket)}</span><span class="tc sell">${t("sell")} +${short(data.eras[tu.era].turret.sell)}</span>`; b.disabled = G.auto; b.classList.add("built"); }
    else { b.innerHTML = `<span class="tn">+ ${turretName(E.bucket)}</span><span class="tc">${short(E.turret.cost)}</span>`; b.disabled = G.auto || p.gold < E.turret.cost; b.classList.remove("built"); }
  }
  const sb = $("#special");
  setText("#special .sname", specialName(E.bucket));
  const cdLeft = p.spCd;
  setText("#special .scd", cdLeft > 0 ? `${Math.ceil(cdLeft / data.tps)} ${t("sec")}` : t("ready"));
  sb.disabled = cdLeft > 0 || G.auto;
  sb.classList.toggle("ready", cdLeft === 0 && !G.auto);
  $("#special .cd").style.height = (cdLeft > 0 ? (cdLeft * 100) / data.specialCd : 0) + "%";

  setHtml("#e0", `${centuryHtml(E.roman)} · ${eraName(p.era)}`);
  setHtml("#e1", `${centuryHtml(Q.roman)} · ${eraName(q.era)}`);
  $("#hp0").style.width = (p.baseHp * 100) / p.baseMax + "%";
  $("#hp1").style.width = (q.baseHp * 100) / q.baseMax + "%";
  setText("#hpt0", `${short(Math.max(0, p.baseHp))} / ${short(p.baseMax)}`);
  setText("#hpt1", `${short(Math.max(0, q.baseHp))} / ${short(q.baseMax)}`);
  const dots = $("#tl").children;
  for (let i = 0; i < dots.length; i++) dots[i].className = "dot" + (i === p.era ? " a" : "") + (i === q.era ? " b" : "") + (i <= Math.max(p.era, q.era) ? " on" : "");
  const sec = Math.floor(st.tick / data.tps);
  const siege = st.tick >= data.suddenTick;
  setHtml("#clock", `${Math.floor(sec / 60)}:${String(sec % 60).padStart(2, "0")}${siege ? ` <b class="siege">${t("siegeClock")}</b>` : ""}`);
}

function toast(text, cls = "") {
  const t = el("div", "toast " + cls, text);
  $("#toasts").append(t);
  setTimeout(() => t.classList.add("out"), 3200);
  setTimeout(() => t.remove(), 3800);
  while ($("#toasts").children.length > 4) $("#toasts").firstChild.remove();
}

function hideOverlays() { $("#over").classList.add("hidden"); $("#paused").classList.add("hidden"); $("#pause")?.classList.remove("on"); }
function openMenu() { $("#menu").classList.remove("hidden"); }
function applyStatic() {
  document.documentElement.lang = getLang();
  document.title = t("doc");
  document.querySelectorAll("[data-i]").forEach((n) => { n.textContent = t(n.dataset.i); });
  document.querySelectorAll(".lvl").forEach((b) => { b.textContent = t("levels")[b.dataset.k]; });
  document.querySelectorAll(".lng").forEach((b) => b.classList.toggle("on", b.dataset.k === getLang()));
}
function changeLang(l) {
  setLang(l);
  store.set("davrlar.lang", l);
  hudCache = {};
  buildHud();
  applyStatic();
  $("#autob").classList.toggle("on", G.auto && !G.demo);
  $("#pause").classList.toggle("on", G.paused);
}
function buildMenu() {
  const lg = $("#langs");
  for (const l of LANGS) {
    const b = el("button", "btn sm lng", LANG_LABEL[l]); b.dataset.k = l;
    b.onclick = () => changeLang(l);
    lg.append(b);
  }
  const lv = $("#levels");
  for (const k of LEVELS) {
    const b = el("button", "btn lvl", ""); b.dataset.k = k;
    b.onclick = () => { G.level = k; store.set("davrlar.level", k); markLevel(); };
    lv.append(b);
  }
  applyStatic();
  markLevel();
  $("#play").onclick = () => startGame({ demo: false, auto: false });
  $("#watch").onclick = () => startGame({ demo: false, auto: true });
  $("#resume").onclick = () => { $("#menu").classList.add("hidden"); if (G.mode === "play") G.paused = false; };
  $("#again").onclick = () => startGame({ demo: false, auto: G.auto });
  $("#tomenu").onclick = () => { startGame({ demo: true }); };
  $("#dl").onclick = () => {
    const blob = new Blob([JSON.stringify(G.replay)], { type: "application/json" });
    const a = document.createElement("a"); a.href = URL.createObjectURL(blob); a.download = `davrlar-replay-${G.seed}.json`; a.click();
  };
}
function markLevel() { document.querySelectorAll(".lvl").forEach((b) => b.classList.toggle("on", b.dataset.k === G.level)); }

addEventListener("keydown", (e) => {
  if (e.repeat || e.ctrlKey || e.metaKey) return;
  const k = e.key.toLowerCase();
  if (k === "1" || k === "2" || k === "3") train(Number(k) - 1);
  else if (k === "e" || k === "у") evolve();
  else if (k === "q" || k === "й") special();
  else if (k === "z" || k === "я") slotClick(0);
  else if (k === "x" || k === "ч") slotClick(1);
  else if (k === "c" || k === "с") slotClick(2);
  else if (k === " ") { e.preventDefault(); togglePause(); }
  else if (k === "a" || k === "ф") $("#autob").click();
  else if (k === "f") $("#spd").click();
});

function render(dt) {
  const st = G.st;
  setFrame(frame);
  clearFrame(frame);
  const lead = Math.max(st.players[0].era, st.players[1].era);
  frame.color.set(backgroundFor(data, lead));
  fxm.drawBack(frame);

  for (let o = 0; o < 2; o++) {
    const p = st.players[o];
    const fl = fxm.baseFlash[o] > 0 ? 0.5 : 0;
    drawBase(data, o, p.era, p.baseHp / p.baseMax, fl);
    for (let s = 0; s < p.turrets.length; s++) if (p.turrets[s]) drawTurret(data, o, s, p.turrets[s].era, p.era, fxm.recoil.get(`t${o}${s}`) ?? 0);
  }

  const rows = st.units.slice().sort((a, b) => (a.id % 3) - (b.id % 3) || a.x - b.x);
  for (const u of rows) {
    const cx = u.x / 10, dy = (u.id % 3) * 3 - 3, dir = u.o === 0 ? 1 : -1;
    const rec = fxm.recoil.get(`u${u.id}`) ?? 0;
    drawUnit(data, u, cx, dy, dir, { walk: (u.x / 6) | 0, atk: rec > 0 ? Math.min(1, rec / 0.1) : 0, flash: fxm.flashes.has(u.id) ? 0.7 : 0 });
    const us = data.eras[u.era].units[u.role];
    if (u.hp < us.hp) drawHpBar(cx, GROUND_Y + dy - (u.role === 2 ? 30 : 24), Math.max(0, u.hp) / us.hp, u.o);
  }
  fxm.drawFront(frame);
  composite(frame, img.data, null);
  ctx.putImageData(img, 0, 0);
  const sh = fxm.shake;
  $("#stage").style.transform = sh > 0.2 ? `translate(${((Math.random() - 0.5) * sh * 1.6).toFixed(1)}px,${((Math.random() - 0.5) * sh * 1.6).toFixed(1)}px)` : "";
}

let last = performance.now();
function loop(ts) {
  const dt = Math.min(0.1, (ts - last) / 1000);
  last = ts;
  const st = G.st;
  const running = st && (G.mode === "play" || G.demo) && !G.paused;
  if (running && st.winner < 0) {
    G.acc += dt * G.speed;
    let n = 0;
    while (G.acc >= 1 / data.tps && n < 16) { stepOnce(); G.acc -= 1 / data.tps; n++; }
    if (n === 16) G.acc = 0;
  }
  if (st) {

    const live = G.mode === "over" ? 1 : G.paused ? 0 : Math.min(G.speed, 3);
    fxm.update(dt * live);
    render(dt);
    updateHud();
  }

  if (G.demo && st && st.winner >= 0 && !G.demoRestart) { G.demoRestart = setTimeout(() => { G.demoRestart = null; startGame({ demo: true }); }, 2500); }
  requestAnimationFrame(loop);
}

window.__davrlar = { G, data, step, hashState, fxm };

buildHud();
buildMenu();
startGame({ demo: true });
requestAnimationFrame(loop);
