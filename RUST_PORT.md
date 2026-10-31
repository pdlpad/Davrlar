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
| `client/draw.js`, `client/fx.js` | рендер, переписывается заново |
| `client/main.js` | цикл и ввод, переписывается заново |

## Правила, чтобы хэши не разошлись

- только целые числа, деление по шагам
- rng: xorshift32, вызывается в одних и тех же местах
- обход юнитов по порядку вектора
- события в хэш не входят
- баланс правится только в `tools/gen-data.mjs`, затем `node tools/gen-data.mjs && node tools/export-replays.mjs`

## Дальше

Рендер: Bevy или macroquad читает `State` и `Event`, ввод превращает в `Cmd`.
Эффекты pixelfx надо переписать (JS-код в `vendor/pixelfx/src/fx`) или запекать в спрайт-листы: `node tools/render.mjs <категория/эффект> --sheet`.
