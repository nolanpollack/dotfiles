use std::collections::BTreeMap;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemePalette {
    #[serde(with = "color_serde")]
    pub separator_fg: Color,
    #[serde(with = "color_serde")]
    pub query_fg: Color,
    #[serde(with = "color_serde")]
    pub list_match_fg: Color,
    #[serde(with = "color_serde")]
    pub list_normal_fg: Color,
    #[serde(with = "color_serde")]
    pub list_inactive_fg: Color,
    #[serde(with = "color_serde")]
    pub list_current_marker_fg: Color,
    #[serde(with = "color_serde")]
    pub list_selected_bg: Color,
    #[serde(with = "color_serde")]
    pub hint_key_fg: Color,
    #[serde(with = "color_serde")]
    pub hint_desc_fg: Color,
    #[serde(with = "color_serde")]
    pub status_count_fg: Color,
    #[serde(with = "color_serde")]
    pub error_fg: Color,
    #[serde(with = "color_serde")]
    pub agent_blocked_fg: Color,
    #[serde(with = "color_serde")]
    pub agent_working_fg: Color,
    #[serde(with = "color_serde")]
    pub agent_done_fg: Color,
    #[serde(with = "color_serde")]
    pub agent_idle_fg: Color,
    #[serde(with = "color_serde")]
    pub agent_unknown_fg: Color,
    #[serde(with = "color_serde")]
    pub panel_divider_fg: Color,
}

mod color_serde {
    use ratatui::style::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    enum Repr {
        Reset,
        Black,
        Red,
        Green,
        Yellow,
        Blue,
        Magenta,
        Cyan,
        Gray,
        DarkGray,
        LightRed,
        LightGreen,
        LightYellow,
        LightBlue,
        LightMagenta,
        LightCyan,
        White,
        Rgb(u8, u8, u8),
        Indexed(u8),
    }

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let repr = match color {
            Color::Reset => Repr::Reset,
            Color::Black => Repr::Black,
            Color::Red => Repr::Red,
            Color::Green => Repr::Green,
            Color::Yellow => Repr::Yellow,
            Color::Blue => Repr::Blue,
            Color::Magenta => Repr::Magenta,
            Color::Cyan => Repr::Cyan,
            Color::Gray => Repr::Gray,
            Color::DarkGray => Repr::DarkGray,
            Color::LightRed => Repr::LightRed,
            Color::LightGreen => Repr::LightGreen,
            Color::LightYellow => Repr::LightYellow,
            Color::LightBlue => Repr::LightBlue,
            Color::LightMagenta => Repr::LightMagenta,
            Color::LightCyan => Repr::LightCyan,
            Color::White => Repr::White,
            Color::Rgb(r, g, b) => Repr::Rgb(*r, *g, *b),
            Color::Indexed(index) => Repr::Indexed(*index),
        };
        repr.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Repr::deserialize(deserializer)? {
            Repr::Reset => Color::Reset,
            Repr::Black => Color::Black,
            Repr::Red => Color::Red,
            Repr::Green => Color::Green,
            Repr::Yellow => Color::Yellow,
            Repr::Blue => Color::Blue,
            Repr::Magenta => Color::Magenta,
            Repr::Cyan => Color::Cyan,
            Repr::Gray => Color::Gray,
            Repr::DarkGray => Color::DarkGray,
            Repr::LightRed => Color::LightRed,
            Repr::LightGreen => Color::LightGreen,
            Repr::LightYellow => Color::LightYellow,
            Repr::LightBlue => Color::LightBlue,
            Repr::LightMagenta => Color::LightMagenta,
            Repr::LightCyan => Color::LightCyan,
            Repr::White => Color::White,
            Repr::Rgb(r, g, b) => Color::Rgb(r, g, b),
            Repr::Indexed(index) => Color::Indexed(index),
        })
    }
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self {
            separator_fg: Color::DarkGray,
            query_fg: Color::White,
            list_match_fg: Color::Cyan,
            list_normal_fg: Color::White,
            list_inactive_fg: Color::DarkGray,
            list_current_marker_fg: Color::Green,
            list_selected_bg: Color::DarkGray,
            hint_key_fg: Color::Green,
            hint_desc_fg: Color::White,
            status_count_fg: Color::DarkGray,
            error_fg: Color::Red,
            agent_blocked_fg: Color::Red,
            agent_working_fg: Color::Yellow,
            agent_done_fg: Color::Green,
            agent_idle_fg: Color::Cyan,
            agent_unknown_fg: Color::DarkGray,
            panel_divider_fg: Color::DarkGray,
        }
    }
}

#[derive(Clone)]
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
    pub(super) agent_blocked_fg: Color,
    pub(super) agent_working_fg: Color,
    pub(super) agent_done_fg: Color,
    pub(super) agent_idle_fg: Color,
    pub(super) agent_unknown_fg: Color,
    pub(super) panel_divider_fg: Color,
}

impl Theme {
    pub fn from_palette(p: ThemePalette, overrides: &ThemeOverrides) -> Self {
        macro_rules! color {
            ($name:ident) => {
                overrides.get(stringify!($name)).unwrap_or(p.$name)
            };
        }
        Self {
            separator_fg: color!(separator_fg),
            query_fg: color!(query_fg),
            list_match_fg: color!(list_match_fg),
            list_normal_fg: color!(list_normal_fg),
            list_inactive_fg: color!(list_inactive_fg),
            list_current_marker_fg: color!(list_current_marker_fg),
            list_selected_bg: color!(list_selected_bg),
            hint_key_fg: color!(hint_key_fg),
            hint_desc_fg: color!(hint_desc_fg),
            status_count_fg: color!(status_count_fg),
            error_fg: color!(error_fg),
            agent_blocked_fg: color!(agent_blocked_fg),
            agent_working_fg: color!(agent_working_fg),
            agent_done_fg: color!(agent_done_fg),
            agent_idle_fg: color!(agent_idle_fg),
            agent_unknown_fg: color!(agent_unknown_fg),
            panel_divider_fg: color!(panel_divider_fg),
        }
    }

    #[cfg(test)]
    pub fn test_default() -> Self {
        Self::from_palette(ThemePalette::default(), &ThemeOverrides::default())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeOverrides(BTreeMap<String, Color>);

impl ThemeOverrides {
    pub fn from_entries(entries: impl IntoIterator<Item = (String, String)>) -> Self {
        Self(
            entries
                .into_iter()
                .filter_map(|(key, value)| parse_color(&value).map(|color| (key, color)))
                .collect(),
        )
    }

    fn get(&self, key: &str) -> Option<Color> {
        self.0.get(key).copied()
    }
}

fn parse_color(value: &str) -> Option<Color> {
    let mut parts = value.split(',').map(|part| part.trim().parse::<u8>());
    let r = parts.next()?.ok()?;
    let g = parts.next()?.ok()?;
    let b = parts.next()?.ok()?;
    (parts.next().is_none()).then_some(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_override_replaces_palette_color() {
        let overrides = ThemeOverrides::from_entries(vec![("query_fg".into(), "1, 2, 3".into())]);
        let theme = Theme::from_palette(ThemePalette::default(), &overrides);
        assert_eq!(theme.query_fg, Color::Rgb(1, 2, 3));
    }

    #[test]
    fn invalid_override_uses_palette_color() {
        let palette = ThemePalette::default();
        let expected = palette.query_fg;
        let overrides = ThemeOverrides::from_entries(vec![("query_fg".into(), "nope".into())]);
        let theme = Theme::from_palette(palette, &overrides);
        assert_eq!(theme.query_fg, expected);
    }
}
