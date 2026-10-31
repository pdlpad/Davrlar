use macroquad::miniquad::conf::Icon;
use macroquad::prelude::{Image, ImageFormat};

fn decode<const N: usize>(png: &[u8]) -> [u8; N] {
    let img = Image::from_file_with_format(png, Some(ImageFormat::Png)).expect("иконка");
    img.bytes.try_into().expect("размер иконки")
}

pub fn icon() -> Icon {
    Icon {
        small: decode(include_bytes!("../assets/icon/icon-16.png")),
        medium: decode(include_bytes!("../assets/icon/icon-32.png")),
        big: decode(include_bytes!("../assets/icon/icon-64.png")),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn icon_sizes_match_window_api() {
        let i = super::icon();
        assert_eq!((i.small[3], i.big[3]), (0, 0));
    }
}

#[cfg(windows)]
pub fn style_title_bar() {
    use std::ffi::c_void;

    const DARK_MODE: u32 = 20;
    const BORDER_COLOR: u32 = 34;
    const TEXT_COLOR: u32 = 36;
    const BACKDROP: u32 = 38;
    const ACRYLIC: u32 = 3;

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attr: u32, value: *const c_void, size: u32) -> i32;
    }
    #[link(name = "user32")]
    extern "system" {
        fn EnumThreadWindows(thread: u32, f: extern "system" fn(isize, isize) -> i32, param: isize) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentThreadId() -> u32;
    }

    extern "system" fn apply(hwnd: isize, _: isize) -> i32 {
        for (attr, value) in [(DARK_MODE, 1u32), (BACKDROP, ACRYLIC), (BORDER_COLOR, 0x0060_313a), (TEXT_COLOR, 0x00ff_e8ec)] {
            unsafe { DwmSetWindowAttribute(hwnd, attr, &value as *const u32 as *const c_void, 4) };
        }
        1
    }

    unsafe { EnumThreadWindows(GetCurrentThreadId(), apply, 0) };
}

#[cfg(not(windows))]
pub fn style_title_bar() {}
