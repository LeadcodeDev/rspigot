#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedColor {
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
}

impl NamedColor {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::DarkBlue => "dark_blue",
            Self::DarkGreen => "dark_green",
            Self::DarkAqua => "dark_aqua",
            Self::DarkRed => "dark_red",
            Self::DarkPurple => "dark_purple",
            Self::Gold => "gold",
            Self::Gray => "gray",
            Self::DarkGray => "dark_gray",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Aqua => "aqua",
            Self::Red => "red",
            Self::LightPurple => "light_purple",
            Self::Yellow => "yellow",
            Self::White => "white",
        }
    }

    pub fn from_legacy_code(code: char) -> Option<Self> {
        Some(match code.to_ascii_lowercase() {
            '0' => Self::Black,
            '1' => Self::DarkBlue,
            '2' => Self::DarkGreen,
            '3' => Self::DarkAqua,
            '4' => Self::DarkRed,
            '5' => Self::DarkPurple,
            '6' => Self::Gold,
            '7' => Self::Gray,
            '8' => Self::DarkGray,
            '9' => Self::Blue,
            'a' => Self::Green,
            'b' => Self::Aqua,
            'c' => Self::Red,
            'd' => Self::LightPurple,
            'e' => Self::Yellow,
            'f' => Self::White,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Named(NamedColor),
    Rgb(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    pub color: Option<Color>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Text(String),
    Translatable { key: String, args: Vec<Component> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub content: Content,
    pub style: Style,
    pub extra: Vec<Component>,
}

impl Component {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: Content::Text(text.into()),
            style: Style::default(),
            extra: Vec::new(),
        }
    }

    pub fn translatable(key: impl Into<String>, args: Vec<Component>) -> Self {
        Self {
            content: Content::Translatable {
                key: key.into(),
                args,
            },
            style: Style::default(),
            extra: Vec::new(),
        }
    }

    pub fn color(mut self, color: NamedColor) -> Self {
        self.style.color = Some(Color::Named(color));
        self
    }

    pub fn bold(mut self) -> Self {
        self.style.bold = Some(true);
        self
    }

    pub fn italic(mut self, value: bool) -> Self {
        self.style.italic = Some(value);
        self
    }

    pub fn append(mut self, child: Component) -> Self {
        self.extra.push(child);
        self
    }

    pub fn from_legacy(input: &str) -> Self {
        parse_legacy(input, |c| c == '§' || c == '&')
    }

    pub fn from_section_codes(input: &str) -> Self {
        parse_legacy(input, |c| c == '§')
    }

    pub fn render(&self, translate: &impl Fn(&str) -> Option<&'static str>) -> String {
        let mut out = String::new();
        self.write_rendered(&mut out, translate);
        out
    }

    fn write_rendered(&self, out: &mut String, translate: &impl Fn(&str) -> Option<&'static str>) {
        match &self.content {
            Content::Text(t) => out.push_str(t),
            Content::Translatable { key, args } => match translate(key) {
                Some(template) => out.push_str(&fill_template(template, args, translate)),
                None => {
                    out.push_str(key);
                    for arg in args {
                        out.push(' ');
                        arg.write_rendered(out, translate);
                    }
                }
            },
        }
        for child in &self.extra {
            child.write_rendered(out, translate);
        }
    }

    pub fn plain_text(&self) -> String {
        let mut out = String::new();
        self.write_plain(&mut out);
        out
    }

    fn write_plain(&self, out: &mut String) {
        match &self.content {
            Content::Text(t) => out.push_str(t),
            Content::Translatable { key, args } => {
                out.push_str(key);
                for arg in args {
                    out.push(' ');
                    arg.write_plain(out);
                }
            }
        }
        for child in &self.extra {
            child.write_plain(out);
        }
    }
}

fn parse_legacy(input: &str, is_marker: impl Fn(char) -> bool) -> Component {
    let mut root = Component::text("");
    let mut current = Style::default();
    let mut buffer = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if is_marker(ch) && chars.peek().is_some_and(|c| is_legacy_code(*c)) {
            let code = chars.next().unwrap_or('r');
            if !buffer.is_empty() {
                root.extra.push(Component {
                    content: Content::Text(std::mem::take(&mut buffer)),
                    style: current.clone(),
                    extra: Vec::new(),
                });
            }
            apply_legacy(&mut current, code);
        } else {
            buffer.push(ch);
        }
    }
    if !buffer.is_empty() {
        root.extra.push(Component {
            content: Content::Text(buffer),
            style: current,
            extra: Vec::new(),
        });
    }
    if root.extra.len() == 1 {
        return root.extra.remove(0);
    }
    root
}

fn fill_template(
    template: &str,
    args: &[Component],
    translate: &impl Fn(&str) -> Option<&'static str>,
) -> String {
    let mut out = String::with_capacity(template.len());
    let mut next = 0usize;
    let mut rest = template;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(tail) = after.strip_prefix('s') {
            if let Some(arg) = args.get(next) {
                arg.write_rendered(&mut out, translate);
            }
            next += 1;
            rest = tail;
        } else if let Some((index, tail)) = after
            .split_once("$s")
            .and_then(|(n, t)| n.parse::<usize>().ok().map(|n| (n, t)))
        {
            if let Some(arg) = index.checked_sub(1).and_then(|i| args.get(i)) {
                arg.write_rendered(&mut out, translate);
            }
            rest = tail;
        } else if let Some(tail) = after.strip_prefix('%') {
            out.push('%');
            rest = tail;
        } else {
            out.push('%');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

fn is_legacy_code(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), '0'..='9' | 'a'..='f' | 'k'..='o' | 'r')
}

fn apply_legacy(style: &mut Style, code: char) {
    if let Some(color) = NamedColor::from_legacy_code(code) {
        *style = Style {
            color: Some(Color::Named(color)),
            ..Style::default()
        };
        return;
    }
    match code.to_ascii_lowercase() {
        'k' => style.obfuscated = Some(true),
        'l' => style.bold = Some(true),
        'm' => style.strikethrough = Some(true),
        'n' => style.underlined = Some(true),
        'o' => style.italic = Some(true),
        _ => *style = Style::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_fills_sequential_and_positional_arguments() {
        let lookup = |key: &str| match key {
            "a" => Some("%s joined %s"),
            "b" => Some("%2$s then %1$s, 100%%"),
            _ => None,
        };
        let c = Component::translatable("a", vec![Component::text("Alice"), Component::text("x")]);
        assert_eq!(c.render(&lookup), "Alice joined x");
        let c = Component::translatable("b", vec![Component::text("one"), Component::text("two")]);
        assert_eq!(c.render(&lookup), "two then one, 100%");
        assert_eq!(
            Component::translatable("missing", Vec::new()).render(&lookup),
            "missing"
        );
    }

    #[test]
    fn legacy_codes_split_into_styled_parts() {
        let c = Component::from_legacy("&aHello §lWorld");
        assert_eq!(c.plain_text(), "Hello World");
        assert_eq!(c.extra.len(), 2);
        assert_eq!(
            c.extra[0].style.color,
            Some(Color::Named(NamedColor::Green))
        );
        assert_eq!(c.extra[1].style.bold, Some(true));
    }

    #[test]
    fn section_codes_ignore_ampersands() {
        let c = Component::from_section_codes("R&D &aok §6Gold §lnow");
        assert_eq!(c.plain_text(), "R&D &aok Gold now");
        assert_eq!(c.extra.len(), 3);
        assert_eq!(c.extra[0].style, Style::default());
        assert_eq!(c.extra[1].style.color, Some(Color::Named(NamedColor::Gold)));
        assert_eq!(c.extra[2].style.bold, Some(true));
        assert_eq!(c.extra[2].style.color, Some(Color::Named(NamedColor::Gold)));

        let plain = Component::from_section_codes("R&D &aok");
        assert_eq!(plain, Component::text("R&D &aok"));
        assert_eq!(Component::from_section_codes(""), Component::text(""));
    }
}
