mod color;
mod draw;
mod fx;
mod hud;
mod i18n;
mod session;
mod window;

use davrlar_core::ai::Level;
use davrlar_core::data::Data;
use davrlar_core::sim::{Cmd, Event};
use draw::{Anim, Bucket, Frame, GROUND_Y, H, W};
use fx::{turret_key, Fx, SHEETS};
use hud::{Hud, BG};
use i18n::{Lang, Names};
use macroquad::prelude::*;
use session::{level_key, Mode, Session, SPEEDS};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const FW: f32 = W as f32;
const FH: f32 = H as f32;
const GAME: &str = include_str!("../../data/game.json");
const NAMES: &str = include_str!("../../data/names.json");
const GOOD: Color = Color::new(0.24, 0.48, 1.0, 1.0);
const BAD: Color = Color::new(1.0, 0.29, 0.24, 1.0);
const NEUTRAL: Color = Color::new(0.23, 0.19, 0.38, 1.0);
const DEMO_RESTART: f64 = 2.5;
const FAST_SPEED: i32 = 8;
const SPLASH: f64 = 2.2;
const SPLASH_FADE: f64 = 0.4;

pub struct Toast {
    text: String,
    color: Color,
    t: f64,
}

pub struct App {
    data: Data,
    buckets: Vec<Bucket>,
    names: Names,
    lang: Lang,
    level: Level,
    sess: Session,
    fx: Fx,
    frame: Frame,
    world: Image,
    world_tex: Texture2D,
    target: RenderTarget,
    blur: [RenderTarget; 2],
    sheets: Vec<Texture2D>,
    logo: Texture2D,
    splash_t: f64,
    fullscreen: bool,
    hud: Hud,
    toasts: Vec<Toast>,
    menu: bool,
    over_t: f64,
    demo_t: f64,
}

fn new_seed() -> i64 {
    let n = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(1);
    (n as i64 & 0x7fff_ffff).max(1)
}

fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("Davrlar")).unwrap_or_else(|| PathBuf::from("."))
}

fn read_settings() -> (Option<String>, Option<String>) {
    let text = std::fs::read_to_string(data_dir().join("settings.txt")).unwrap_or_default();
    let get = |key: &str| text.lines().find_map(|l| l.strip_prefix(key)).map(str::to_string);
    (get("lang="), get("level="))
}

impl App {
    fn start(&mut self, demo: bool, auto: bool) {
        self.sess = Session::new(&self.data, new_seed(), self.level, demo, auto);
        self.fx.clear();
        self.toasts.clear();
        self.menu = demo;
        self.over_t = 0.0;
        self.demo_t = 0.0;
    }

    fn save_settings(&self) {
        let dir = data_dir();
        let text = format!("lang={}\nlevel={}\n", self.lang.key(), level_key(self.level));
        if std::fs::create_dir_all(&dir).is_ok() {
            let _ = std::fs::write(dir.join("settings.txt"), text);
        }
    }

    fn set_lang(&mut self, l: Lang) {
        self.lang = l;
        self.save_settings();
    }

    fn set_level(&mut self, l: Level) {
        self.level = l;
        self.save_settings();
    }

    fn toast(&mut self, text: String, color: Color) {
        self.toasts.push(Toast { text, color, t: 0.0 });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    fn save_replay(&mut self) {
        let dir = data_dir().join("replays");
        let path = dir.join(format!("davrlar-replay-{}.json", self.sess.seed));
        let saved = std::fs::create_dir_all(&dir).is_ok() && std::fs::write(&path, self.sess.replay_json()).is_ok();
        if saved {
            self.toast(format!("{}: {}", self.lang.ui().saved, path.display()), GOOD);
        } else {
            self.toast(path.display().to_string(), BAD);
        }
    }

    fn toggle_pause(&mut self) {
        if self.sess.mode == Mode::Play {
            self.sess.paused = !self.sess.paused;
        }
    }

    fn cycle_speed(&mut self) {
        let i = SPEEDS.iter().position(|s| *s == self.sess.speed).unwrap_or(0);
        self.sess.speed = SPEEDS[(i + 1) % SPEEDS.len()];
    }

    fn toggle_auto(&mut self) {
        if self.sess.mode == Mode::Play {
            self.sess.auto = !self.sess.auto;
            let ui = self.lang.ui();
            self.toast(if self.sess.auto { ui.auto_on } else { ui.auto_off }.to_string(), NEUTRAL);
        }
    }

    fn input(&mut self) {
        for key in get_keys_pressed() {
            match key {
                KeyCode::Key1 => self.sess.command(Cmd::Train { p: 0, role: 0 }),
                KeyCode::Key2 => self.sess.command(Cmd::Train { p: 0, role: 1 }),
                KeyCode::Key3 => self.sess.command(Cmd::Train { p: 0, role: 2 }),
                KeyCode::E => self.sess.command(Cmd::Evolve { p: 0 }),
                KeyCode::Q => self.sess.command(Cmd::Special { p: 0 }),
                KeyCode::Z => self.sess.toggle_slot(0),
                KeyCode::X => self.sess.toggle_slot(1),
                KeyCode::C => self.sess.toggle_slot(2),
                KeyCode::Space => self.toggle_pause(),
                KeyCode::A => self.toggle_auto(),
                KeyCode::F => self.cycle_speed(),
                KeyCode::F11 => {
                    self.fullscreen = !self.fullscreen;
                    set_fullscreen(self.fullscreen);
                }
                _ => {}
            }
        }
    }

    fn on_event(&mut self, e: &Event) {
        if self.sess.demo {
            return;
        }
        let ui = self.lang.ui();
        let who = |o: usize| if o == 0 { "" } else { ui.enemy };
        let tint = |o: usize| if o == 0 { GOOD } else { BAD };
        match e {
            Event::Evolve { o, era } => {
                let text = format!("{}{}", who(*o), self.lang.century(&self.data.eras[*era].roman));
                self.toast(text, tint(*o));
            }
            Event::Special { o, bucket, .. } => {
                let text = format!("{}{}", who(*o), self.names.special(self.lang, *bucket));
                self.toast(text, tint(*o));
            }
            Event::Siege if self.sess.st.tick == self.data.sudden_tick => self.toast(ui.siege.to_string(), BAD),
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        let ev = self.sess.advance(&self.data, dt);
        self.fx.fast = self.sess.speed >= FAST_SPEED;
        self.fx.handle(&self.data, &ev);
        for e in &ev {
            self.on_event(e);
        }
        let live = match (self.sess.mode, self.sess.paused) {
            (Mode::Over, _) => 1.0,
            (_, true) => 0.0,
            _ => self.sess.speed.min(3) as f64,
        };
        self.fx.update(dt * live);
        for t in self.toasts.iter_mut() {
            t.t += dt;
        }
        self.toasts.retain(|t| t.t < 3.8);
        if self.sess.mode == Mode::Over {
            self.over_t += dt;
        }
        if self.sess.demo && self.sess.st.winner >= 0 {
            self.demo_t += dt;
            if self.demo_t >= DEMO_RESTART {
                self.start(true, false);
            }
        }
    }

    fn paint_world(&mut self) {
        let st = &self.sess.st;
        let lead = st.players[0].era.max(st.players[1].era);
        self.frame.background(&self.data, &self.buckets, lead);
        for o in 0..2 {
            let p = &st.players[o];
            let flash = if self.fx.base_flash[o] > 0.0 { 0.5 } else { 0.0 };
            self.frame.base(&self.data, o, p.era, p.base_hp as f64 / p.base_max as f64, flash);
            for (s, t) in p.turrets.iter().enumerate() {
                if let Some(t) = t {
                    let rec = self.fx.recoil.get(&turret_key(o, s as i64)).copied().unwrap_or(0.0);
                    self.frame.turret(&self.data, o, s, t.era, p.era, rec);
                }
            }
        }
        let mut rows: Vec<_> = st.units.iter().collect();
        rows.sort_by_key(|u| (u.id % 3, u.x));
        for u in rows {
            let cx = (u.x / 10) as i32;
            let dy = (u.id % 3) as i32 * 3 - 3;
            let dir = if u.o == 0 { 1 } else { -1 };
            let rec = self.fx.recoil.get(&u.id).copied().unwrap_or(0.0);
            let anim = Anim {
                walk: (u.x / 6) as i32,
                atk: if rec > 0.0 { (rec / 0.1).min(1.0) } else { 0.0 },
                flash: if self.fx.flashes.contains_key(&u.id) { 0.7 } else { 0.0 },
            };
            self.frame.unit(&self.data, u, cx, dy, dir, &anim);
            let max = self.data.eras[u.era].units[u.role].hp;
            if u.hp < max {
                let top = GROUND_Y + dy - if u.role == 2 { 30 } else { 24 };
                self.frame.hp_bar(cx, top, u.hp.max(0) as f64 / max as f64);
            }
        }
        for (px, c) in self.world.bytes.chunks_exact_mut(4).zip(&self.frame.color) {
            px.copy_from_slice(&c.to_le_bytes());
        }
        self.world_tex.update(&self.world);
    }

    fn splash(&mut self, dt: f64) -> bool {
        if self.splash_t >= SPLASH {
            return false;
        }
        self.splash_t += dt;
        if is_mouse_button_pressed(MouseButton::Left) || !get_keys_pressed().is_empty() {
            self.splash_t = SPLASH;
        }
        let t = self.splash_t;
        let alpha = (t / SPLASH_FADE).min((SPLASH - t) / SPLASH_FADE).clamp(0.0, 1.0) as f32;
        clear_background(BG);
        self.hud.begin();
        self.hud.image(&self.logo, self.hud.vw / 2.0, self.hud.vh / 2.0, 1000.0, alpha);
        true
    }

    fn blur_stage(&self) {
        let mut src = &self.target.texture;
        for rt in &self.blur {
            let (w, h) = (rt.texture.width(), rt.texture.height());
            set_camera(&Camera2D {
                zoom: vec2(2.0 / w, 2.0 / h),
                target: vec2(w / 2.0, h / 2.0),
                render_target: Some(rt.clone()),
                ..Default::default()
            });
            draw_texture_ex(src, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
            src = &rt.texture;
        }
        set_default_camera();
    }

    fn draw_stage(&mut self) {
        set_camera(&Camera2D {
            zoom: vec2(2.0 / FW, 2.0 / FH),
            target: vec2(FW / 2.0, FH / 2.0),
            render_target: Some(self.target.clone()),
            ..Default::default()
        });
        clear_background(BLACK);
        draw_texture(&self.world_tex, 0.0, 0.0, WHITE);
        for it in &self.fx.items {
            let i = &self.fx.info(it.sheet);
            let f = self.fx.frame_of(it);
            let src = Rect::new((f % i.cols) as f32 * i.w, (f / i.cols) as f32 * i.h, i.w, i.h);
            let (x, y) = (it.x as f32, it.y as f32);
            let dx = if it.flip { x - (i.w - i.ox) } else { x - i.ox };
            draw_texture_ex(&self.sheets[it.sheet], dx, y - i.oy, WHITE, DrawTextureParams {
                source: Some(src),
                rotation: it.angle as f32,
                pivot: Some(vec2(x, y)),
                flip_x: it.flip,
                ..Default::default()
            });
        }
        for l in &self.fx.labels {
            hud::draw_label(l);
        }
        self.blur_stage();
        clear_background(BG);
        let a = self.hud.area();
        draw_texture_ex(&self.blur[1].texture, a.x, a.y, WHITE, DrawTextureParams { dest_size: Some(vec2(a.w, a.h)), ..Default::default() });
        let s = self.hud.s;
        let shake = self.fx.shake;
        let (mut sx, mut sy) = (0.0, 0.0);
        if shake > 0.2 {
            sx = ((self.fx.random() - 0.5) * shake * 1.6) as f32 * 2.0;
            sy = ((self.fx.random() - 0.5) * shake * 1.6) as f32 * 2.0;
        }
        let d = self.hud.dest;
        draw_texture_ex(&self.target.texture, d.x + sx * s, d.y + sy * s, WHITE, DrawTextureParams {
            source: Some(self.hud.src),
            dest_size: Some(vec2(d.w, d.h)),
            ..Default::default()
        });
    }
}

fn conf() -> Conf {
    Conf {
        window_title: "Davrlar".to_string(),
        window_width: 1120,
        window_height: 796,
        window_resizable: true,
        icon: Some(window::icon()),
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    window::style_title_bar();
    let data = Data::load(GAME).expect("разбор game.json");
    let (saved_lang, saved_level) = read_settings();
    let level = saved_level.as_deref().and_then(Level::parse).unwrap_or(Level::Normal);
    let font = load_ttf_font_from_bytes(dejavu::sans_mono::regular()).expect("шрифт");
    let sheets: Vec<Texture2D> = SHEETS
        .iter()
        .map(|(_, bytes)| {
            let t = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
            t.set_filter(FilterMode::Nearest);
            t
        })
        .collect();
    let logo = Texture2D::from_file_with_format(include_bytes!("../assets/logo.png"), Some(ImageFormat::Png));
    let world = Image::gen_image_color(W as u16, H as u16, BLACK);
    let world_tex = Texture2D::from_image(&world);
    world_tex.set_filter(FilterMode::Nearest);
    let target = render_target(W as u32, H as u32);
    target.texture.set_filter(FilterMode::Nearest);
    let blur = [render_target(128, 72), render_target(48, 27)];
    for rt in &blur {
        rt.texture.set_filter(FilterMode::Linear);
    }
    let sess = Session::new(&data, new_seed(), level, true, false);

    let mut app = App {
        buckets: draw::load_buckets(GAME),
        names: Names::load(NAMES),
        lang: Lang::detect(saved_lang.as_deref()),
        level,
        sess,
        fx: Fx::new(fx::load_infos()),
        frame: Frame::new(),
        world,
        world_tex,
        target,
        blur,
        sheets,
        logo,
        splash_t: 0.0,
        fullscreen: false,
        hud: Hud::new(font),
        toasts: Vec::new(),
        menu: true,
        over_t: 0.0,
        demo_t: 0.0,
        data,
    };

    loop {
        let dt = (get_frame_time() as f64).min(0.1);
        if app.splash(dt) {
            next_frame().await;
            continue;
        }
        app.hud.begin();
        app.input();
        app.update(dt);
        app.paint_world();
        app.draw_stage();
        app.draw_hud();
        app.draw_toasts();
        app.draw_overlays();
        next_frame().await;
    }
}
