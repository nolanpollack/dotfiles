use std::collections::BTreeMap;

use ratatui::style::Color;
use zellij_tile::prelude::{PaletteColor, Style as ZellijStyle};

pub struct Theme {
    pub(super) separator_fg: Color,
    pub(super) query_fg: Color,
    pub(super) list_match_fg: Color,
    pub(super) list_normal_fg: Color,
    pub(super) list_inactive_fg: Color,
    pub(super) list_current_marker_fg: Color,
    pub(super) list_selected_bg: Color,
    pub(super) hint_key_fg: Color,
    pub(super) hint_desc_fg: Color,
    pub(super) status_count_fg: Color,
    pub(super) error_fg: Color,
}

impl Theme {
    pub fn from_zellij(z: &ZellijStyle, overrides: &ThemeOverrides) -> Self {
        let s = &z.colors;
        Self {
            separator_fg: overrides.get("separator_fg").unwrap_or_else(|| {
                palette(s.frame_unselected.map(|f| f.base).unwrap_or(s.frame_selected.base))
            }),
            query_fg: overrides
                .get("query_fg")
                .unwrap_or_else(|| palette(s.text_unselected.base)),
            list_match_fg: overrides
                .get("list_match_fg")
                .unwrap_or_else(|| palette(s.list_unselected.emphasis_0)),
            list_normal_fg: overrides
                .get("list_normal_fg")
                .unwrap_or_else(|| palette(s.list_unselected.base)),
            list_inactive_fg: overrides
                .get("list_inactive_fg")
                .unwrap_or_else(|| palette(s.text_unselected.background)),
            list_current_marker_fg: overrides
                .get("list_current_marker_fg")
                .unwrap_or_else(|| palette(s.list_unselected.emphasis_1)),
            list_selected_bg: overrides
                .get("list_selected_bg")
                .unwrap_or_else(|| palette(s.list_selected.background)),
            hint_key_fg: overrides
                .get("hint_key_fg")
                .unwrap_or_else(|| palette(s.list_unselected.emphasis_1)),
            hint_desc_fg: overrides
                .get("hint_desc_fg")
                .unwrap_or_else(|| palette(s.list_unselected.base)),
            status_count_fg: overrides
                .get("status_count_fg")
                .unwrap_or_else(|| palette(s.text_unselected.background)),
            error_fg: overrides.get("error_fg").unwrap_or_else(|| palette(s.exit_code_error.base)),
        }
    }
}

fn palette(pc: PaletteColor) -> Color {
    match pc {
        PaletteColor::Rgb((r, g, b)) => Color::Rgb(r, g, b),
        PaletteColor::EightBit(n) => Color::Indexed(n),
    }
}

/// User-supplied colors from the plugin's `configuration` block, keyed by the same names as
/// `Theme`'s fields. Falls back to the Zellij theme wherever a key is absent or unparseable.
#[derive(Default)]
pub struct ThemeOverrides(BTreeMap<String, Color>);

impl ThemeOverrides {
    pub fn from_config(config: &BTreeMap<String, String>) -> Self {
        Self(
            config
                .iter()
                .filter_map(|(key, value)| parse_color(value).map(|c| (key.clone(), c)))
                .collect(),
        )
    }

    fn get(&self, key: &str) -> Option<Color> {
        self.0.get(key).copied()
    }
}

/// Parses a "r,g,b" config value, e.g. "49,50,68".
fn parse_color(s: &str) -> Option<Color> {
    let mut parts = s.split(',').map(|p| p.trim().parse::<u8>());
    let r = parts.next()?.ok()?;
    let g = parts.next()?.ok()?;
    let b = parts.next()?.ok()?;
    (parts.next().is_none()).then(|| Color::Rgb(r, g, b))
}
