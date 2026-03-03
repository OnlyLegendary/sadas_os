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
pub enum GlassMode {
    Disabled,
    Frosted,
    ObsidianGlass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemePalette {
    pub background: u32,
    pub foreground: u32,
    pub accent: u32,
    pub surface: u32,
}

impl ThemePalette {
    pub const fn from_mode(mode: ThemeMode, glass: GlassMode) -> Self {
        match (mode, glass) {
            (ThemeMode::Light, GlassMode::Disabled) => Self {
                background: 0xF2F4F8,
                foreground: 0x1A1F2E,
                accent: 0x3967FF,
                surface: 0xFFFFFF,
            },
            (ThemeMode::Light, GlassMode::Frosted) => Self {
                background: 0xE8EEF8,
                foreground: 0x172033,
                accent: 0x4678FF,
                surface: 0xF8FBFF,
            },
            (ThemeMode::Light, GlassMode::ObsidianGlass) => Self {
                background: 0xDFE7F6,
                foreground: 0x1E2434,
                accent: 0x5D8BFF,
                surface: 0xDDE3F0,
            },
            (ThemeMode::Dark, GlassMode::Disabled) => Self {
                background: 0x0F1117,
                foreground: 0xE7EBF7,
                accent: 0x7DA2FF,
                surface: 0x171B25,
            },
            (ThemeMode::Dark, GlassMode::Frosted) => Self {
                background: 0x111827,
                foreground: 0xEAF0FF,
                accent: 0x93B4FF,
                surface: 0x202A3D,
            },
            (ThemeMode::Dark, GlassMode::ObsidianGlass) => Self {
                background: 0x0B0D12,
                foreground: 0xF2F4FF,
                accent: 0xA78BFA,
                surface: 0x161924,
            },
            (ThemeMode::HighContrast, _) => Self {
                background: 0x000000,
                foreground: 0xFFFFFF,
                accent: 0x00FF88,
                surface: 0x050505,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiProfile {
    pub theme: ThemeMode,
    pub density: Density,
    pub animation: AnimationLevel,
    pub glass_mode: GlassMode,
    pub corner_radius_px: u8,
}

impl UiProfile {
    pub const fn for_legacy_device() -> Self {
        Self {
            theme: ThemeMode::HighContrast,
            density: Density::Compact,
            animation: AnimationLevel::Off,
            glass_mode: GlassMode::Disabled,
            corner_radius_px: 4,
        }
    }

    pub const fn for_balanced_device() -> Self {
        Self {
            theme: ThemeMode::Dark,
            density: Density::Comfortable,
            animation: AnimationLevel::Reduced,
            glass_mode: GlassMode::Frosted,
            corner_radius_px: 10,
        }
    }

    pub const fn for_modern_device() -> Self {
        Self {
            theme: ThemeMode::Dark,
            density: Density::Comfortable,
            animation: AnimationLevel::Full,
            glass_mode: GlassMode::ObsidianGlass,
            corner_radius_px: 14,
        }
    }

    pub const fn palette(&self) -> ThemePalette {
        ThemePalette::from_mode(self.theme, self.glass_mode)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Surface {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopLayout {
    pub sidebar_width: u16,
    pub topbar_height: u16,
    pub content_width: u16,
    pub content_height: u16,
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

    pub fn desktop_layout(&self, density: Density) -> DesktopLayout {
        let sidebar_width = match density {
            Density::Compact => self.width / 6,
            Density::Comfortable => self.width / 5,
        };
        let topbar_height = match density {
            Density::Compact => 42,
            Density::Comfortable => 56,
        };

        DesktopLayout {
            sidebar_width,
            topbar_height,
            content_width: self.width.saturating_sub(sidebar_width),
            content_height: self.height.saturating_sub(topbar_height),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_layout_never_exceeds_surface() {
        let surface = Surface {
            width: 1920,
            height: 1080,
        };
        let layout = surface.desktop_layout(Density::Comfortable);
        assert!(layout.sidebar_width <= surface.width);
        assert!(layout.content_width <= surface.width);
        assert!(layout.content_height <= surface.height);
    }

    #[test]
    fn modern_profile_uses_obsidian_glass() {
        let profile = UiProfile::for_modern_device();
        assert_eq!(profile.glass_mode, GlassMode::ObsidianGlass);
        let palette = profile.palette();
        assert_eq!(palette.background, 0x0B0D12);
    }
}
