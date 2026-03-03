#![no_std]

use sadas_sysapi::ThemeMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Density {
    Compact,
    Comfortable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnimationLevel {
    Off,
    Reduced,
    Full,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiProfile {
    pub theme: ThemeMode,
    pub density: Density,
    pub animation: AnimationLevel,
    pub blur_enabled: bool,
}

impl UiProfile {
    pub const fn for_legacy_device() -> Self {
        Self {
            theme: ThemeMode::HighContrast,
            density: Density::Compact,
            animation: AnimationLevel::Off,
            blur_enabled: false,
        }
    }

    pub const fn for_balanced_device() -> Self {
        Self {
            theme: ThemeMode::Dark,
            density: Density::Comfortable,
            animation: AnimationLevel::Reduced,
            blur_enabled: false,
        }
    }

    pub const fn for_modern_device() -> Self {
        Self {
            theme: ThemeMode::Dark,
            density: Density::Comfortable,
            animation: AnimationLevel::Full,
            blur_enabled: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Surface {
    pub width: u16,
    pub height: u16,
}

impl Surface {
    pub fn frame_budget_ms(&self, refresh_hz: u16) -> u16 {
        if refresh_hz == 0 {
            return 16;
        }
        1000 / refresh_hz
    }

    pub fn recommended_render_scale(&self) -> u8 {
        let pixels = self.width as u32 * self.height as u32;
        if pixels <= 1280 * 720 {
            100
        } else if pixels <= 1920 * 1080 {
            85
        } else {
            70
        }
    }
}
