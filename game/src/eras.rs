use crate::color::hex;

#[derive(Clone, Copy, PartialEq)]
pub enum Helm {
    Galea,
    Ridge,
    Hood,
    Spire,
    Turban,
    Nasal,
    Spectacle,
    Great,
    Kettle,
    Fur,
    Bascinet,
    Morion,
    Feather,
    Tricorn,
    Shako,
    Brodie,
    Modern,
    Robot,
    Visor,
    Dome,
    Space,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Skyline {
    Temples,
    Aqueduct,
    Ruins,
    Pines,
    Domes,
    Minarets,
    Castles,
    Cathedrals,
    Yurts,
    Windmills,
    Town,
    Factories,
    City,
    Towers,
    Neon,
    Spires,
    Orbit,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Crown {
    Eagle,
    Palisade,
    Dome,
    Minaret,
    Yurt,
    Bell,
    Stack,
    Radar,
    Antenna,
    Orb,
}

pub struct Style {
    pub metal: u32,
    pub metal_d: u32,
    pub cloth: u32,
    pub dark: u32,
    pub accent: u32,
    pub stone: u32,
    pub sky: u32,
    pub ground: u32,
    pub helm: Helm,
    pub skyline: Skyline,
    pub crown: Crown,
    pub mount: bool,
    pub pole: bool,
}

macro_rules! style {
    ($m:literal $md:literal $c:literal $d:literal $a:literal $st:literal $sk:literal $g:literal $h:ident $l:ident $cr:ident $mt:literal $p:literal) => {
        Style {
            metal: hex($m),
            metal_d: hex($md),
            cloth: hex($c),
            dark: hex($d),
            accent: hex($a),
            stone: hex($st),
            sky: hex($sk),
            ground: hex($g),
            helm: Helm::$h,
            skyline: Skyline::$l,
            crown: Crown::$cr,
            mount: $mt,
            pole: $p,
        }
    };
}

pub const STYLES: [Style; 25] = [
    style!(0xc9a45c 0x8a6a36 0xc8b38a 0x4a3820 0xd02a24 0xe0d6be 0x6fa8dc 0x9a7a42 Galea Temples Eagle false false),
    style!(0xe0c060 0x9a7a30 0xe8dcc0 0x4a3820 0x7a2a9a 0xf0e8d4 0x7fc0ea 0xa88a48 Galea Aqueduct Eagle false false),
    style!(0xa8a090 0x6e685a 0xa08a68 0x3a2c1c 0xa03a20 0xb0a68e 0xc09a70 0x7a5c34 Ridge Ruins Eagle true false),
    style!(0xb0b4b8 0x70747a 0x8a6a7a 0x342a30 0x8a2a6a 0xc4bcae 0x8f8aa8 0x74623e Ridge Aqueduct Eagle true false),
    style!(0x8a8478 0x5a564c 0x7a5a3a 0x2a2018 0x3a6a3a 0x8e8878 0x7a7f88 0x5e5a34 Hood Pines Palisade false false),
    style!(0xd8b860 0x8a6a2a 0x6a2a7a 0x2a1a30 0xf0d060 0xd8ccb0 0xa8b8d8 0x8a7a48 Spire Domes Dome true false),
    style!(0xc8ccd4 0x7a808c 0xece4d0 0x3a3020 0x2a8a5a 0xe8d8b0 0xe8c080 0xb08a50 Turban Minarets Minaret true false),
    style!(0x9aa4b4 0x5e6878 0x3a5a9a 0x262a38 0xd8b040 0xb0a890 0x8aa0c0 0x6e6e40 Nasal Castles Bell true false),
    style!(0x8a9098 0x585e66 0x8a6a48 0x2a2420 0xb83020 0x7a6a54 0x6a8498 0x4e5a40 Spectacle Pines Palisade false false),
    style!(0xb4bcc8 0x6e7886 0xb8342a 0x2a2d38 0xe8e0d0 0xbcb49c 0x90a8c8 0x6a7040 Nasal Castles Bell true true),
    style!(0xc0c8d4 0x7a8494 0xece8dc 0x2a2d38 0xc8201e 0xd4c8a8 0xe0c890 0xa89050 Great Cathedrals Bell true false),
    style!(0xb8c0cc 0x6c7684 0x2a2a30 0x20222c 0xece8dc 0xc0b8a0 0x8ea8d0 0x70783e Kettle Cathedrals Bell true true),
    style!(0x9a8a6a 0x5e5240 0x7a4a2a 0x2a2018 0x3a8ac8 0xc0a878 0xb8d0e0 0xa09a58 Fur Yurts Yurt true false),
    style!(0xc4ccd8 0x747e8e 0x1f3f8a 0x20222c 0xe0b040 0xa8a090 0x8a94a8 0x5a6a3a Bascinet Windmills Bell false true),
    style!(0xc0c4cc 0x6e727a 0xb83a3a 0x2a2118 0xe8c860 0xd8c0a0 0x98b8d8 0x6e7a44 Morion Town Dome false true),
    style!(0x9a9ea8 0x5a5e68 0xd8a020 0x2a2118 0xd02a4a 0xb8a488 0xa0a8b0 0x6a6a42 Feather Windmills Bell true false),
    style!(0x8a8e98 0x4f535c 0x6a4a30 0x241c14 0xc8c8d0 0x8a7e68 0x8a8a84 0x54523a Feather Ruins Bell true true),
    style!(0xc8ccd0 0x7a7e86 0xe8e2cf 0x1e2440 0xc01f2a 0xd4c8b0 0xb0c4d4 0x747a48 Tricorn Town Bell false false),
    style!(0xd0d4dc 0x7e8490 0x1f2a6a 0x18181e 0xe02a2a 0x8a5a4a 0xb0a090 0x5a5238 Shako Factories Stack true false),
    style!(0x6e7a58 0x48523a 0x7a7458 0x2a2c1e 0xb8b070 0x7a7a70 0x8a8878 0x4a4a34 Brodie Factories Stack false false),
    style!(0x8f9a74 0x586149 0x5a6a50 0x20261c 0x3af0a0 0x9aa4a8 0x9ab0c0 0x5a6448 Modern City Radar false false),
    style!(0xb8c4d0 0x5a6a80 0x3a4658 0x121822 0xff8a2a 0x6a7890 0x3a3a70 0x2e3048 Robot Towers Antenna false false),
    style!(0xd0e0f0 0x4a6aa0 0x2c3558 0x0e1224 0x4af0ff 0x4a6288 0x2a1a60 0x28244a Visor Neon Antenna false false),
    style!(0x9af0c8 0x2a8a70 0x143a3a 0x061a1e 0x5affa0 0x2a6a64 0x0e3a4a 0x143038 Dome Spires Orb false false),
    style!(0xf0f0f8 0x8088a8 0x3a2a70 0x0a0a20 0xffc83a 0x7070a8 0x0a0820 0x1a1838 Space Orbit Orb false false),
];

pub fn style(era: usize) -> &'static Style {
    &STYLES[era.min(STYLES.len() - 1)]
}
