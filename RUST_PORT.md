# Перенос на Rust

Ядро (`core/sim.js`, `core/ai.js`) уже портировано в `rust-core/` и совпадает с JS побитово.

Проверка:

    node tools/export-replays.mjs
    cd rust-core && cargo test --release
    ./target/release/davrlar-core verify data/game.json tests/replays/*.json

Оба теста: реплей по журналу команд и партия Rust-ботов дают тот же итоговый хэш состояния, что и JS.

## Что где

| JS | Rust |
|---|---|
| `data/game.json` (из `tools/gen-data.mjs`) | `src/data.rs` |
| `core/sim.js` | `src/sim.rs` |
| `core/ai.js` | `src/ai.rs` |
| `client/draw.js`, `client/fx.js` | `game/src/draw.rs`, `game/src/fx.rs` |
| `client/main.js` | `game/src/main.rs`, `game/src/session.rs`, `game/src/hud.rs` |
| `client/i18n.js` | `game/src/i18n.rs` |

## Правила, чтобы хэши не разошлись

- только целые числа, деление по шагам
- rng: xorshift32, вызывается в одних и тех же местах
- обход юнитов по порядку вектора
- события в хэш не входят
- баланс правится только в `tools/gen-data.mjs`, затем `node tools/gen-data.mjs && node tools/export-replays.mjs`

## Эффекты

`node tools/bake-fx.mjs` запекает эффекты pixelfx в `game/assets/fx`: PNG-листы и `manifest.json`. Новый лист добавляется в `SHEETS` скрипта и в `sheets!` в `game/src/fx.rs`.
