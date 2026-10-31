use crate::fx::{short, Label};
use crate::i18n::{level_index, LANGS};
use crate::session::Mode;
use crate::App;
use davrlar_core::ai::Level;
use davrlar_core::sim::Cmd;
use macroquad::prelude::*;

const VW: f32 = 1280.0;
const VH: f32 = 910.0;
const STAGE_Y: f32 = 64.0;
const BOTTOM_Y: f32 = 788.0;
const BOTTOM_H: f32 = 122.0;
const WORLD_W: f32 = 640.0;
const WORLD_H: f32 = 360.0;
const MIN_VISIBLE_H: f32 = 200.0;

const fn rgba(v: u32, a: f32) -> Color {
    Color::new(((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0, a)
}
const fn col(v: u32) -> Color {
    rgba(v, 1.0)
}
pub const BG: Color = col(0x0d0b14);
const PANEL: Color = col(0x171427);
const PANEL2: Color = col(0x1f1a36);
const LINE: Color = col(0x3a3160);
const GOLD: Color = col(0xffcf4a);
const TXT: Color = col(0xece8ff);
const DIM: Color = col(0x8f88b8);
const BLUE: Color = col(0x3d7bff);
const RED: Color = col(0xff4a3d);
const GREEN: Color = col(0x59e07a);
const DARK: Color = col(0x0b0916);
const HOVER: Color = col(0x2a2350);
const ACCENT: Color = col(0x8c7bff);
const ORANGE: Color = col(0xff9a3d);
const PURPLE: Color = col(0x7a4be8);
const QUEUE: [Color; 3] = [col(0x2f6bd0), col(0x2fa85a), col(0xb8861a)];

pub struct Hud {
    font: Font,
    pub s: f32,
    pub vw: f32,
    pub vh: f32,
    pub dest: Rect,
    pub src: Rect,
    ox: f32,
    oy: f32,
    mouse: (f32, f32),
    click: bool,
}

#[derive(Clone, Copy)]
struct Box2 {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}
const fn bx(x: f32, y: f32, w: f32, h: f32) -> Box2 {
    Box2 { x, y, w, h }
}

fn dim(c: Color, on: bool) -> Color {
    if on {
        c
    } else {
        Color::new(c.r, c.g, c.b, c.a * 0.45)
    }
}

impl Hud {
    pub fn new(font: Font) -> Hud {
        Hud { font, s: 1.0, vw: VW, vh: VH, dest: Rect::default(), src: Rect::default(), ox: 0.0, oy: 0.0, mouse: (0.0, 0.0), click: false }
    }

    pub fn begin(&mut self) {
        let (sw, sh) = (screen_width(), screen_height());
        self.s = (sw / VW).min(sh / VH);
        self.vw = sw / self.s;
        self.vh = sh / self.s;
        self.mouse = mouse_position();
        self.click = is_mouse_button_pressed(MouseButton::Left);
        self.fit_stage();
    }

    fn fit_stage(&mut self) {
        let (area_w, area_h) = (self.vw * self.s, self.area_h() * self.s);
        let top = STAGE_Y * self.s;
        let by_width = area_w / WORLD_W;
        let visible = (area_h / by_width).clamp(MIN_VISIBLE_H, WORLD_H);
        let k = by_width.min(area_h / visible);
        let (w, h) = (WORLD_W * k, visible * k);
        self.dest = Rect::new((area_w - w) / 2.0, top + area_h - h, w, h);
        self.src = Rect::new(0.0, WORLD_H - visible, WORLD_W, visible);
    }

    fn area_h(&self) -> f32 {
        self.vh - STAGE_Y - BOTTOM_H - 4.0
    }

    pub fn area(&self) -> Rect {
        Rect::new(0.0, STAGE_Y * self.s, self.vw * self.s, self.area_h() * self.s)
    }

    fn rect(&self, b: Box2, c: Color) {
        draw_rectangle(self.ox + b.x * self.s, self.oy + b.y * self.s, b.w * self.s, b.h * self.s, c);
    }
    fn panel(&self, b: Box2, fill: Color, border: Color) {
        self.rect(b, border);
        self.rect(bx(b.x + 2.0, b.y + 2.0, b.w - 4.0, b.h - 4.0), fill);
    }
    fn rows(&self, b: Box2, r: f32, mut f: impl FnMut(f32, f32, f32, f32)) {
        let (x, y) = (self.ox + b.x * self.s, self.oy + b.y * self.s);
        let (w, h, r) = (b.w * self.s, (b.h * self.s).ceil(), r * self.s);
        for i in 0..h as i32 {
            let row = i as f32 + 0.5;
            let d = (r - row.min(h - row)).max(0.0);
            let inset = r - (r * r - d * d).max(0.0).sqrt();
            f(x + inset, y + i as f32, w - inset * 2.0, row / h);
        }
    }
    fn gloss(&self, b: Box2, r: f32, top: Color, bottom: Color) {
        self.rows(b, r, |x, y, w, k| {
            let c = Color::new(top.r + (bottom.r - top.r) * k, top.g + (bottom.g - top.g) * k, top.b + (bottom.b - top.b) * k, top.a + (bottom.a - top.a) * k);
            draw_rectangle(x, y, w, 1.0, c);
        });
    }
    fn stroke(&self, b: Box2, r: f32, c: Color) {
        let s = self.s;
        let (x, y, w, h, r) = (self.ox + b.x * s, self.oy + b.y * s, b.w * s, b.h * s, r * s);
        draw_line(x + r, y, x + w - r, y, 1.0, c);
        draw_line(x + r, y + h, x + w - r, y + h, 1.0, c);
        draw_line(x, y + r, x, y + h - r, 1.0, c);
        draw_line(x + w, y + r, x + w, y + h - r, 1.0, c);
        for (cx, cy, from) in [(x + w - r, y + r, 270.0), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, 90.0), (x + r, y + r, 180.0)] {
            draw_arc(cx, cy, 24, r, from, 1.0, 90.0, c);
        }
    }
    fn glass(&self, b: Box2, r: f32, blur: &Texture2D) {
        let (sx, sy) = (blur.width() / WORLD_W, blur.height() / WORLD_H);
        let (kx, ky) = (self.src.w / self.dest.w, self.src.h / self.dest.h);
        self.rows(b, r, |x, y, w, _| {
            let (u, v) = (self.src.x + (x - self.dest.x) * kx, self.src.y + (y - self.dest.y) * ky);
            let src = Rect::new(u * sx, v * sy, w * kx * sx, ky * sy);
            draw_texture_ex(blur, x, y, WHITE, DrawTextureParams { source: Some(src), dest_size: Some(vec2(w, 1.0)), ..Default::default() });
        });
        self.gloss(b, r, rgba(0x20204a, 0.42), rgba(0x0b0916, 0.58));
        self.gloss(bx(b.x, b.y, b.w, b.h * 0.42), r, rgba(0xffffff, 0.16), rgba(0xffffff, 0.0));
        self.stroke(b, r, rgba(0xffffff, 0.38));
    }
    pub fn image(&self, t: &Texture2D, cx: f32, cy: f32, w: f32, alpha: f32) {
        let h = w * t.height() / t.width();
        let (x, y) = (self.ox + (cx - w / 2.0) * self.s, self.oy + (cy - h / 2.0) * self.s);
        let params = DrawTextureParams { dest_size: Some(vec2(w * self.s, h * self.s)), ..Default::default() };
        draw_texture_ex(t, x, y, Color::new(1.0, 1.0, 1.0, alpha), params);
    }
    fn width(&self, t: &str, size: f32) -> f32 {
        let px = (size * self.s).round().max(6.0) as u16;
        measure_text(t, Some(&self.font), px, 1.0).width / self.s
    }
    fn text(&self, t: &str, x: f32, y: f32, size: f32, c: Color) {
        let px = (size * self.s).round().max(6.0) as u16;
        let p = TextParams { font: Some(&self.font), font_size: px, color: c, ..Default::default() };
        draw_text_ex(t, (self.ox + x * self.s).round(), (self.oy + y * self.s).round(), p);
    }
    fn fit(&self, t: &str, size: f32, max_w: f32) -> f32 {
        let w = self.width(t, size);
        if w > max_w {
            (size * max_w / w).floor()
        } else {
            size
        }
    }
    fn fitted(&self, t: &str, x: f32, y: f32, size: f32, max_w: f32, c: Color) {
        self.text(t, x, y, self.fit(t, size, max_w), c);
    }
    fn center(&self, t: &str, cx: f32, y: f32, size: f32, c: Color) {
        self.text(t, cx - self.width(t, size) / 2.0, y, size, c);
    }
    fn right(&self, t: &str, rx: f32, y: f32, size: f32, c: Color) {
        self.text(t, rx - self.width(t, size), y, size, c);
    }
    fn hover(&self, b: Box2) -> bool {
        let (x, y) = ((self.mouse.0 - self.ox) / self.s, (self.mouse.1 - self.oy) / self.s);
        x >= b.x && y >= b.y && x < b.x + b.w && y < b.y + b.h
    }
    fn button(&self, b: Box2, on: bool, border: Option<Color>) -> bool {
        let hover = on && self.hover(b);
        let fill = if hover { HOVER } else { PANEL2 };
        let edge = border.unwrap_or(if hover { ACCENT } else { LINE });
        self.panel(b, dim(fill, on), dim(edge, on));
        hover && self.click
    }
    fn kbd(&self, b: Box2, t: &str) {
        let k = bx(b.x + b.w - 19.0, b.y + 5.0, 15.0, 15.0);
        self.rect(k, LINE);
        self.rect(bx(k.x + 1.0, k.y + 1.0, k.w - 2.0, k.h - 2.0), col(0x0e0b1c));
        self.center(t, k.x + k.w / 2.0, k.y + 12.0, 11.0, DIM);
    }
    fn bar(&self, b: Box2, frac: f32, c: Color, from_right: bool, label: &str) {
        self.rect(b, LINE);
        let inner = bx(b.x + 1.0, b.y + 1.0, b.w - 2.0, b.h - 2.0);
        self.rect(inner, DARK);
        let w = inner.w * frac.clamp(0.0, 1.0);
        self.rect(bx(if from_right { inner.x + inner.w - w } else { inner.x }, inner.y, w, inner.h), c);
        self.center(label, b.x + b.w / 2.0, b.y + b.h - 3.5, 11.0, TXT);
    }
}

const fn glyph(c: char) -> [u8; 5] {
    match c {
        '0' => [7, 5, 5, 5, 7],
        '1' => [2, 6, 2, 2, 7],
        '2' => [7, 1, 7, 4, 7],
        '3' => [7, 1, 3, 1, 7],
        '4' => [5, 5, 7, 1, 1],
        '5' => [7, 4, 7, 1, 7],
        '6' => [7, 4, 7, 5, 7],
        '7' => [7, 1, 1, 2, 2],
        '8' => [7, 5, 7, 5, 7],
        '9' => [7, 5, 7, 1, 7],
        '-' => [0, 0, 7, 0, 0],
        '.' => [0, 0, 0, 0, 2],
        'K' => [5, 5, 6, 5, 5],
        'M' => [5, 7, 7, 5, 5],
        'A' => [2, 5, 7, 5, 5],
        'G' => [7, 4, 5, 5, 7],
        'E' => [7, 4, 6, 4, 7],
        'I' => [7, 2, 2, 2, 7],
        'V' => [5, 5, 5, 5, 2],
        'X' => [5, 5, 2, 5, 5],
        _ => [0; 5],
    }
}

fn pixel_text(t: &str, x: f32, y: f32, k: f32, c: Color) {
    for (i, ch) in t.chars().enumerate() {
        let g = glyph(ch);
        for (j, row) in g.iter().enumerate() {
            for b in 0..3 {
                if row & (4 >> b) != 0 {
                    draw_rectangle(x + (i as f32 * 4.0 + b as f32) * k, y + j as f32 * k, k, k, c);
                }
            }
        }
    }
}

pub fn draw_label(l: &Label) {
    let u = (l.t / l.life) as f32;
    let k = if l.gold { 2.0 } else { 1.0 };
    let width = l.text.chars().count() as f32 * 4.0 * k;
    let (x, y) = (l.x as f32 - width / 2.0, l.y as f32 - if l.gold { 0.0 } else { 16.0 * u });
    let a = ((1.0 - u) * 4.0).min(1.0);
    let ink = Color::new(0.05, 0.04, 0.08, a);
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        pixel_text(&l.text, x + dx, y + dy, k, ink);
    }
    let c = if l.gold { GOLD } else { col(0xff5a4a) };
    pixel_text(&l.text, x, y, k, Color::new(c.r, c.g, c.b, a));
}

impl App {
    pub fn draw_hud(&mut self) {
        let (vw, vh) = (self.hud.vw, self.hud.vh);
        self.hud.rect(bx(0.0, 0.0, vw, 62.0), LINE);
        self.hud.rect(bx(2.0, 2.0, vw - 4.0, 58.0), PANEL);
        self.top_panel();
        self.hud.rect(bx(0.0, vh - BOTTOM_H, vw, BOTTOM_H), PANEL);
        self.hud.rect(bx(0.0, vh - BOTTOM_H, vw, 2.0), LINE);
        self.hud.ox = (vw - VW) / 2.0 * self.hud.s;
        self.hud.oy = (vh - VH) * self.hud.s;
        self.bottom_panels();
        self.hud.ox = 0.0;
        self.hud.oy = 0.0;
    }

    fn era_line(&self, era: usize) -> (String, String) {
        (self.lang.century(&self.data.eras[era].roman), self.names.era(self.lang, era).to_string())
    }

    fn top_panel(&self) {
        let h = &self.hud;
        let st = &self.sess.st;
        let mid = h.vw / 2.0;
        let side_w = mid - 180.0;
        for o in 0..2 {
            let p = &st.players[o];
            let (cent, name) = self.era_line(p.era);
            let (left, tint) = if o == 0 { (12.0, BLUE) } else { (mid * 2.0 - 12.0 - side_w, RED) };
            let line = format!("{cent} · {name}");
            let size = h.fit(&line, 14.0, side_w);
            let hp = format!("{} / {}", short(p.base_hp.max(0)), short(p.base_max));
            if o == 0 {
                h.text(&line, left, 24.0, size, GOLD);
            } else {
                h.right(&line, left + side_w, 24.0, size, GOLD);
            }
            h.bar(bx(left, 32.0, side_w, 18.0), p.base_hp as f32 / p.base_max as f32, tint, o == 1, &hp);
        }
        let n = self.data.eras.len();
        let (a, b) = (st.players[0].era, st.players[1].era);
        let x0 = mid - n as f32 * 6.0;
        for i in 0..n {
            let c = if i == a && i == b {
                ACCENT
            } else if i == a {
                BLUE
            } else if i == b {
                RED
            } else if i <= a.max(b) {
                col(0x4a3f88)
            } else {
                col(0x241f3d)
            };
            h.rect(bx(x0 + i as f32 * 12.0, 12.0, 10.0, 10.0), c);
        }
        let sec = st.tick / self.data.tps;
        let clock = format!("{}:{:02}", sec / 60, sec % 60);
        if st.tick >= self.data.sudden_tick {
            let blink = (get_time() * 2.5).fract() < 0.6;
            h.center(&format!("{clock}  {}", self.lang.ui().siege_clock), mid, 46.0, 14.0, if blink { RED } else { DIM });
        } else {
            h.center(&clock, mid, 46.0, 14.0, DIM);
        }
    }

    fn bottom_panels(&mut self) {
        let ui = self.lang.ui();
        let lang = self.lang;
        let p = self.sess.st.players[0].clone();
        let e = &self.data.eras[p.era];
        let auto = self.sess.auto;
        let mut acts: Vec<Cmd> = Vec::new();
        let mut slot_click = None;
        let pulse = ((get_time() * std::f64::consts::TAU).sin() as f32 + 1.0) / 2.0;
        let glow = |c: Color| Color::new(c.r, c.g, c.b, 0.55 + 0.45 * pulse);

        let y = BOTTOM_Y;
        let res = bx(0.0, y, 310.0, BOTTOM_H);
        self.hud.panel(res, PANEL, LINE);
        draw_circle(self.hud.ox + 22.0 * self.hud.s, self.hud.oy + (y + 26.0) * self.hud.s, 7.0 * self.hud.s, GOLD);
        let gold = short(p.gold);
        self.hud.text(&gold, 36.0, y + 35.0, 26.0, GOLD);
        let inc = (e.income * p.inc_pct) as f64 / 100_000.0;
        self.hud.right(&format!("+{}{}", short(inc.round() as i64), ui.per_sec), 298.0, y + 33.0, 14.0, DIM);
        let last = p.era >= self.data.eras.len() - 1;
        let frac = if last { 1.0 } else { p.xp as f32 / e.xp_need as f32 };
        let xp_label = if last { "MAX".to_string() } else { format!("{} / {}", short(p.xp), short(e.xp_need)) };
        self.hud.bar(bx(10.0, y + 54.0, 290.0, 20.0), frac, PURPLE, false, &xp_label);
        let can_evo = !last && p.xp >= e.xp_need && !auto;
        let evo = bx(10.0, y + 80.0, 290.0, 34.0);
        if self.hud.button(evo, can_evo, can_evo.then(|| glow(GOLD))) {
            acts.push(Cmd::Evolve { p: 0 });
        }
        self.hud.text(ui.evolve, evo.x + 10.0, evo.y + 23.0, 15.0, dim(if can_evo { GOLD } else { TXT }, can_evo));
        self.hud.kbd(evo, "E");

        let train = bx(316.0, y, 420.0, BOTTOM_H);
        self.hud.panel(train, PANEL, LINE);
        for r in 0..3 {
            let u = &e.units[r];
            let on = !auto && p.gold >= u.cost && p.queue.len() < self.data.queue_max;
            let b = bx(324.0 + r as f32 * 136.0, y + 8.0, 132.0, 82.0);
            if self.hud.button(b, on, None) {
                acts.push(Cmd::Train { p: 0, role: r as i64 });
            }
            self.hud.text(&ui.roles[r].to_uppercase(), b.x + 8.0, b.y + 18.0, 10.0, dim(DIM, on));
            self.hud.fitted(self.names.unit(lang, p.era, r), b.x + 8.0, b.y + 42.0, 13.0, 116.0, dim(TXT, on));
            self.hud.text(&short(u.cost), b.x + 8.0, b.y + 68.0, 15.0, dim(GOLD, on));
            self.hud.kbd(b, ["1", "2", "3"][r]);
        }
        for (i, q) in p.queue.iter().enumerate() {
            let b = bx(324.0 + i as f32 * 26.0, y + 96.0, 22.0, 16.0);
            self.hud.rect(b, LINE);
            let inner = bx(b.x + 1.0, b.y + 1.0, b.w - 2.0, b.h - 2.0);
            self.hud.rect(inner, QUEUE[q.role]);
            if i == 0 {
                let total = self.data.eras[q.era].units[q.role].train as f32;
                let k = 1.0 - q.t as f32 / total;
                self.hud.rect(bx(inner.x, inner.y, inner.w * k, inner.h), Color::new(1.0, 1.0, 1.0, 0.33));
            }
        }

        let spec = bx(742.0, y, 370.0, BOTTOM_H);
        self.hud.panel(spec, PANEL, LINE);
        for s in 0..self.data.turret_slots {
            let b = bx(750.0 + s as f32 * 119.0, y + 8.0, 116.0, 50.0);
            match &p.turrets[s] {
                Some(t) => {
                    let te = &self.data.eras[t.era];
                    if self.hud.button(b, !auto, Some(col(0x5b6bb0))) {
                        slot_click = Some(s);
                    }
                    self.hud.fitted(self.names.turret(lang, te.bucket), b.x + 7.0, b.y + 20.0, 11.0, 102.0, dim(TXT, !auto));
                    let sell = format!("{} +{}", ui.sell, short(te.turret.sell));
                    self.hud.fitted(&sell, b.x + 7.0, b.y + 39.0, 11.0, 102.0, dim(DIM, !auto));
                }
                None => {
                    let on = !auto && p.gold >= e.turret.cost;
                    if self.hud.button(b, on, None) {
                        slot_click = Some(s);
                    }
                    let name = format!("+ {}", self.names.turret(lang, e.bucket));
                    self.hud.fitted(&name, b.x + 7.0, b.y + 20.0, 11.0, 102.0, dim(TXT, on));
                    self.hud.text(&short(e.turret.cost), b.x + 7.0, b.y + 39.0, 12.0, dim(GOLD, on));
                }
            }
        }
        let cd = p.sp_cd;
        let on = cd == 0 && !auto;
        let sb = bx(750.0, y + 63.0, 354.0, 51.0);
        if self.hud.button(sb, on, on.then(|| glow(ORANGE))) {
            acts.push(Cmd::Special { p: 0 });
        }
        self.hud.text(self.names.special(lang, e.bucket), sb.x + 10.0, sb.y + 22.0, 14.0, dim(TXT, on));
        let cd_text = if cd > 0 { format!("{} {}", (cd + self.data.tps - 1) / self.data.tps, ui.sec) } else { ui.ready.to_string() };
        self.hud.text(&cd_text, sb.x + 10.0, sb.y + 41.0, 12.0, dim(DIM, on));
        if cd > 0 {
            let k = cd as f32 / self.data.special_cd as f32;
            let hh = (sb.h - 4.0) * k;
            self.hud.rect(bx(sb.x + 2.0, sb.y + sb.h - 2.0 - hh, sb.w - 4.0, hh), Color::new(0.0, 0.0, 0.0, 0.5));
        }
        self.hud.kbd(sb, "Q");

        let ctl = bx(1118.0, y, 162.0, BOTTOM_H);
        self.hud.panel(ctl, PANEL, LINE);
        let cell = |i: f32, j: f32| bx(1126.0 + i * 74.0, y + 8.0 + j * 54.0, 72.0, 50.0);
        let (bp, bs, ba, bm) = (cell(0.0, 0.0), cell(1.0, 0.0), cell(0.0, 1.0), cell(1.0, 1.0));
        let play = self.sess.mode == Mode::Play;
        if self.hud.button(bp, true, self.sess.paused.then_some(GOLD)) {
            self.toggle_pause();
        }
        self.hud.center(if self.sess.paused { ">" } else { "||" }, bp.x + 36.0, bp.y + 32.0, 16.0, TXT);
        if self.hud.button(bs, true, None) {
            self.cycle_speed();
        }
        self.hud.center(&format!("×{}", self.sess.speed), bs.x + 36.0, bs.y + 32.0, 16.0, TXT);
        let auto_lit = self.sess.auto && !self.sess.demo;
        if self.hud.button(ba, true, auto_lit.then_some(GOLD)) {
            self.toggle_auto();
        }
        self.hud.center(ui.auto, ba.x + 36.0, ba.y + 31.0, self.hud.fit(ui.auto, 13.0, 62.0), TXT);
        if self.hud.button(bm, true, None) {
            if play {
                self.sess.paused = true;
            }
            self.menu = true;
        }
        self.hud.center(ui.menu, bm.x + 36.0, bm.y + 31.0, self.hud.fit(ui.menu, 13.0, 62.0), TXT);

        for c in acts {
            self.sess.command(c);
        }
        if let Some(s) = slot_click {
            self.sess.toggle_slot(s);
        }
    }

    pub fn draw_toasts(&mut self) {
        let mut y = STAGE_Y + 10.0;
        for t in &self.toasts {
            let w = self.hud.width(&t.text, 13.0) + 24.0;
            let b = bx(self.hud.vw / 2.0 - w / 2.0, y, w, 26.0);
            let a = ((3.8 - t.t) / 0.6).min(1.0) as f32;
            let edge = Color::new(t.color.r, t.color.g, t.color.b, a);
            self.hud.panel(bx(b.x, b.y, b.w, b.h), Color::new(0.04, 0.03, 0.09, 0.92 * a), edge);
            self.hud.center(&t.text, self.hud.vw / 2.0, y + 18.0, 13.0, Color::new(TXT.r, TXT.g, TXT.b, a));
            y += 30.0;
        }
    }

    pub fn draw_overlays(&mut self) {
        if self.sess.paused && !self.menu {
            let a = self.hud.area_h();
            self.hud.rect(bx(0.0, STAGE_Y, self.hud.vw, a), rgba(0x0b0916, 0.3));
            self.hud.center(self.lang.ui().paused, self.hud.vw / 2.0, STAGE_Y + a / 2.0, 36.0, GOLD);
        }
        if self.menu {
            self.draw_menu();
        } else if self.sess.mode == Mode::Over && self.over_t > 1.8 {
            self.draw_over();
        }
    }

    fn card(&self, h: f32) -> Box2 {
        let a = self.hud.area_h();
        self.hud.rect(bx(0.0, STAGE_Y, self.hud.vw, a), rgba(0x0b0916, 0.3));
        let c = bx(self.hud.vw / 2.0 - 300.0, STAGE_Y + a / 2.0 - h / 2.0, 600.0, h);
        self.hud.glass(c, 28.0, &self.blur[1].texture);
        c
    }

    fn big_button(&self, b: Box2, label: &str, on_mark: bool) -> bool {
        let hover = self.hud.hover(b);
        let fill = if on_mark {
            rgba(0xffcf4a, 0.18)
        } else if hover {
            rgba(0xffffff, 0.2)
        } else {
            rgba(0xffffff, 0.09)
        };
        self.hud.gloss(b, 12.0, fill, Color::new(fill.r, fill.g, fill.b, fill.a * 0.6));
        self.hud.stroke(b, 12.0, if on_mark { GOLD } else { rgba(0xffffff, 0.24) });
        let clicked = hover && self.hud.click;
        let size = self.hud.fit(label, 16.0, b.w - 16.0);
        self.hud.center(label, b.x + b.w / 2.0, b.y + b.h / 2.0 + size * 0.36, size, if on_mark { GOLD } else { TXT });
        clicked
    }

    fn draw_menu(&mut self) {
        let ui = self.lang.ui();
        let resume = self.sess.mode == Mode::Play && !self.sess.demo;
        let c = self.card(if resume { 398.0 } else { 342.0 });
        let cx = c.x + c.w / 2.0;
        self.hud.image(&self.logo, cx, c.y + 72.0, 440.0, 1.0);
        for (i, l) in LANGS.into_iter().enumerate() {
            let b = bx(cx - 111.0 + i as f32 * 76.0, c.y + 158.0, 70.0, 34.0);
            if self.big_button(b, l.label(), l == self.lang) {
                self.set_lang(l);
            }
        }
        let current = level_index(self.level);
        for (i, lv) in [Level::Easy, Level::Normal, Level::Hard].into_iter().enumerate() {
            let b = bx(cx - 216.0 + i as f32 * 146.0, c.y + 202.0, 140.0, 40.0);
            if self.big_button(b, ui.levels[i], i == current) {
                self.set_level(lv);
            }
        }
        if self.big_button(bx(c.x + 24.0, c.y + 258.0, 180.0, 56.0), ui.play, false) {
            self.start(false, false);
        }
        if self.big_button(bx(c.x + 212.0, c.y + 258.0, 364.0, 56.0), ui.watch, false) {
            self.start(false, true);
        }
        if resume && self.big_button(bx(c.x + 24.0, c.y + 322.0, 552.0, 52.0), ui.resume, false) {
            self.menu = false;
            self.sess.paused = false;
        }
    }

    fn draw_over(&mut self) {
        let ui = self.lang.ui();
        let st = &self.sess.st;
        let (w, auto) = (st.winner, self.sess.auto);
        let (title, tint) = match (w, auto) {
            (2, _) => (ui.draw, TXT),
            (0, true) => (ui.blue_win, TXT),
            (_, true) => (ui.red_win, TXT),
            (0, false) => (ui.win, GREEN),
            _ => (ui.lose, RED),
        };
        let sec = st.tick / self.data.tps;
        let romans = format!("{} / {}", self.data.eras[st.players[0].era].roman, self.data.eras[st.players[1].era].roman);
        let stats = format!("{}:{:02} · {}", sec / 60, sec % 60, self.lang.century(&romans));
        let c = self.card(206.0);
        let cx = c.x + c.w / 2.0;
        self.hud.center(title, cx, c.y + 62.0, 34.0, tint);
        self.hud.center(&stats, cx, c.y + 102.0, 15.0, DIM);
        let row = |i: f32| bx(c.x + 24.0 + i * 188.0, c.y + 130.0, 176.0, 52.0);
        if self.big_button(row(0.0), ui.again, false) {
            self.start(false, auto);
        }
        if self.big_button(row(1.0), ui.replay, false) {
            self.save_replay();
        }
        if self.big_button(row(2.0), ui.menu, false) {
            self.start(true, false);
            self.menu = true;
        }
    }
}
