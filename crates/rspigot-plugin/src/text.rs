use std::fmt;

use rspigot_plugin_protocol::v1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChatColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Magic,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Reset,
}

impl ChatColor {
    const fn code(self) -> char {
        match self {
            Self::Black => '0',
            Self::DarkBlue => '1',
            Self::DarkGreen => '2',
            Self::DarkAqua => '3',
            Self::DarkRed => '4',
            Self::DarkPurple => '5',
            Self::Gold => '6',
            Self::Gray => '7',
            Self::DarkGray => '8',
            Self::Blue => '9',
            Self::Green => 'a',
            Self::Aqua => 'b',
            Self::Red => 'c',
            Self::LightPurple => 'd',
            Self::Yellow => 'e',
            Self::White => 'f',
            Self::Magic => 'k',
            Self::Bold => 'l',
            Self::Strikethrough => 'm',
            Self::Underline => 'n',
            Self::Italic => 'o',
            Self::Reset => 'r',
        }
    }
}

impl fmt::Display for ChatColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "§{}", self.code())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title {
    title: String,
    subtitle: Option<String>,
    fade_in: i32,
    stay: i32,
    fade_out: i32,
}

impl Title {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            fade_in: 10,
            stay: 70,
            fade_out: 20,
        }
    }

    pub fn subtitle(self, text: impl Into<String>) -> Self {
        Self {
            subtitle: Some(text.into()),
            ..self
        }
    }

    pub fn times(self, fade_in: i32, stay: i32, fade_out: i32) -> Self {
        Self {
            fade_in,
            stay,
            fade_out,
            ..self
        }
    }

    pub(crate) fn into_wire(self) -> v1::Title {
        v1::Title {
            title: self.title,
            subtitle: self.subtitle,
            fade_in: self.fade_in,
            stay: self.stay,
            fade_out: self.fade_out,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChatColor, Title};

    #[test]
    fn chat_colors_print_their_codes() {
        assert_eq!(format!("{}Welcome", ChatColor::Gold), "§6Welcome");
        assert_eq!(ChatColor::Reset.to_string(), "§r");
    }

    #[test]
    fn a_title_defaults_to_spigot_times() {
        let title = Title::new("A");
        assert_eq!((title.fade_in, title.stay, title.fade_out), (10, 70, 20));
    }
}
