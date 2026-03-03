#![no_std]

use sadas_sysapi::ThemeMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Density {
    Compact,
    Comfortable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiProfile {
    pub theme: ThemeMode,
    pub density: Density,
    pub animations_enabled: bool,
}

impl UiProfile {
    pub const fn for_legacy_device() -> Self {
        Self {
            theme: ThemeMode::Dark,
            density: Density::Compact,
            animations_enabled: false,
        }
    }

    pub const fn for_modern_device() -> Self {
        Self {
            theme: ThemeMode::Dark,
            density: Density::Comfortable,
            animations_enabled: true,
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
}
