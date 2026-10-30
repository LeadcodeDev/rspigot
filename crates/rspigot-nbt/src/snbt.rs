use crate::compound::Compound;
use crate::tag::Tag;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{key} at {position}")]
pub struct SnbtError {
    pub key: &'static str,
    pub position: usize,
}

const MAX_DEPTH: usize = 512;

struct Parser<'a> {
    text: &'a [u8],
    source: &'a str,
    pos: usize,
    depth: usize,
}

fn unquoted(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.' | b'+')
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IntType {
    Byte,
    Short,
    Int,
    Long,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            text: source.as_bytes(),
            source,
            pos: 0,
            depth: 0,
        }
    }

    fn fail<T>(&self, key: &'static str) -> Result<T, SnbtError> {
        Err(SnbtError {
            key,
            position: self.pos,
        })
    }

    fn peek(&self) -> Option<u8> {
        self.text.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, c: u8, key: &'static str) -> Result<(), SnbtError> {
        self.skip_ws();
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            self.fail(key)
        }
    }

    fn value(&mut self) -> Result<Tag, SnbtError> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.compound().map(Tag::Compound),
            Some(b'[') => self.list_or_array(),
            Some(b'"' | b'\'') => self.quoted().map(Tag::String),
            Some(_) => self.literal(),
            None => self.fail("parsing.expected.value"),
        }
    }

    fn enter(&mut self) -> Result<(), SnbtError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return self.fail("snbt.parser.too_deep");
        }
        Ok(())
    }

    fn compound(&mut self) -> Result<Compound, SnbtError> {
        self.expect(b'{', "parsing.expected.compound")?;
        self.enter()?;
        let mut out = Compound::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            self.depth -= 1;
            return Ok(out);
        }
        loop {
            self.skip_ws();
            let key = match self.peek() {
                Some(b'"' | b'\'') => self.quoted()?,
                _ => {
                    let start = self.pos;
                    while self.peek().is_some_and(unquoted) {
                        self.pos += 1;
                    }
                    if start == self.pos {
                        return self.fail("snbt.parser.empty_key");
                    }
                    self.source[start..self.pos].to_owned()
                }
            };
            if key.is_empty() {
                return self.fail("snbt.parser.empty_key");
            }
            self.expect(b':', "parsing.expected.colon")?;
            let value = self.value()?;
            out.insert(key, value);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    self.skip_ws();
                    if self.peek() == Some(b'}') {
                        self.pos += 1;
                        break;
                    }
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => return self.fail("parsing.expected.closing_brace"),
            }
        }
        self.depth -= 1;
        Ok(out)
    }

    fn list_or_array(&mut self) -> Result<Tag, SnbtError> {
        self.expect(b'[', "parsing.expected.list")?;
        self.enter()?;
        let prefix = match (self.peek(), self.text.get(self.pos + 1)) {
            (Some(c @ (b'B' | b'I' | b'L')), Some(b';')) => {
                self.pos += 2;
                Some(c)
            }
            _ => None,
        };
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
        } else {
            loop {
                items.push(self.value()?);
                self.skip_ws();
                match self.peek() {
                    Some(b',') => {
                        self.pos += 1;
                        self.skip_ws();
                        if self.peek() == Some(b']') {
                            self.pos += 1;
                            break;
                        }
                    }
                    Some(b']') => {
                        self.pos += 1;
                        break;
                    }
                    _ => return self.fail("parsing.expected.closing_bracket"),
                }
            }
        }
        self.depth -= 1;
        let Some(prefix) = prefix else {
            return Ok(Tag::List(items));
        };
        let numbers: Option<Vec<i64>> = items
            .iter()
            .map(|t| match t {
                Tag::Byte(_) | Tag::Short(_) | Tag::Int(_) | Tag::Long(_) => t.as_i64(),
                _ => None,
            })
            .collect();
        let Some(numbers) = numbers else {
            return self.fail("snbt.parser.invalid_array_element_type");
        };
        Ok(match prefix {
            b'B' => Tag::ByteArray(numbers.iter().map(|n| *n as i8 as u8).collect()),
            b'I' => Tag::IntArray(numbers.iter().map(|n| *n as i32).collect()),
            _ => Tag::LongArray(numbers),
        })
    }

    fn hex(&mut self, digits: usize) -> Result<u32, SnbtError> {
        let end = self.pos + digits;
        let Some(slice) = self.source.get(self.pos..end) else {
            return self.fail("snbt.parser.expected_hex_escape");
        };
        let value = u32::from_str_radix(slice, 16)
            .or_else(|_| self.fail("snbt.parser.expected_hex_escape"))?;
        self.pos = end;
        Ok(value)
    }

    fn quoted(&mut self) -> Result<String, SnbtError> {
        let Some(quote) = self.peek() else {
            return self.fail("parsing.quote.expected.start");
        };
        self.pos += 1;
        let mut out = String::new();
        loop {
            let Some(rest) = self.source.get(self.pos..) else {
                return self.fail("parsing.quote.expected.end");
            };
            let Some(c) = rest.chars().next() else {
                return self.fail("parsing.quote.expected.end");
            };
            self.pos += c.len_utf8();
            if c as u32 == u32::from(quote) {
                return Ok(out);
            }
            if c != '\\' {
                out.push(c);
                continue;
            }
            let Some(escape) = self.peek() else {
                return self.fail("parsing.quote.expected.end");
            };
            self.pos += 1;
            let code = match escape {
                b'\\' => '\\' as u32,
                b'"' => '"' as u32,
                b'\'' => '\'' as u32,
                b'b' => 8,
                b'f' => 12,
                b'n' => 10,
                b'r' => 13,
                b's' => 32,
                b't' => 9,
                b'x' => self.hex(2)?,
                b'u' => self.hex(4)?,
                b'U' => self.hex(8)?,
                _ => return self.fail("snbt.parser.invalid_string_contents"),
            };
            match char::from_u32(code) {
                Some(ch) => out.push(ch),
                None => return self.fail("snbt.parser.invalid_codepoint"),
            }
        }
    }

    fn literal(&mut self) -> Result<Tag, SnbtError> {
        let start = self.pos;
        while self.peek().is_some_and(unquoted) {
            self.pos += 1;
        }
        let token = &self.source[start..self.pos];
        if token.is_empty() {
            return self.fail("snbt.parser.expected_unquoted_string");
        }
        self.skip_ws();
        if self.peek() == Some(b'(') {
            return self.operation(token, start);
        }
        if token.eq_ignore_ascii_case("true") {
            return Ok(Tag::Byte(1));
        }
        if token.eq_ignore_ascii_case("false") {
            return Ok(Tag::Byte(0));
        }
        if let Some(number) = number(token) {
            return number.map_err(|key| SnbtError {
                key,
                position: start,
            });
        }
        let first = token.as_bytes()[0];
        if first.is_ascii_digit() || matches!(first, b'+' | b'-' | b'.') {
            self.pos = start;
            return self.fail("snbt.parser.invalid_unquoted_start");
        }
        Ok(Tag::String(token.to_owned()))
    }

    fn operation(&mut self, name: &str, start: usize) -> Result<Tag, SnbtError> {
        self.pos += 1;
        let mut args = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b')') {
            self.pos += 1;
        } else {
            loop {
                args.push(self.value()?);
                self.skip_ws();
                match self.peek() {
                    Some(b',') => self.pos += 1,
                    Some(b')') => {
                        self.pos += 1;
                        break;
                    }
                    _ => return self.fail("parsing.expected.closing_parenthesis"),
                }
            }
        }
        let fail = |key| SnbtError {
            key,
            position: start,
        };
        match (name, args.as_slice()) {
            ("bool", [Tag::Byte(v)]) => Ok(Tag::Byte(i8::from(*v != 0))),
            ("bool", [value]) => match value.as_f64() {
                Some(v) => Ok(Tag::Byte(i8::from(v != 0.0))),
                None => Err(fail("snbt.parser.expected_number_or_boolean")),
            },
            ("uuid", [Tag::String(text)]) => parse_uuid(text)
                .map(Tag::IntArray)
                .ok_or(fail("snbt.parser.expected_string_uuid")),
            _ => Err(fail("snbt.parser.no_such_operation")),
        }
    }
}

fn parse_uuid(text: &str) -> Option<Vec<i32>> {
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 5 {
        return None;
    }
    let most = (u64::from_str_radix(parts[0], 16).ok()? << 32)
        | (u64::from_str_radix(parts[1], 16).ok()? << 16)
        | u64::from_str_radix(parts[2], 16).ok()?;
    let least =
        (u64::from_str_radix(parts[3], 16).ok()? << 48) | u64::from_str_radix(parts[4], 16).ok()?;
    Some(vec![
        (most >> 32) as i32,
        most as i32,
        (least >> 32) as i32,
        least as i32,
    ])
}

fn strip_underscores(digits: &str) -> Result<String, &'static str> {
    if digits.starts_with('_') || digits.ends_with('_') {
        return Err("snbt.parser.underscore_not_allowed");
    }
    Ok(digits.replace('_', ""))
}

fn number(token: &str) -> Option<Result<Tag, &'static str>> {
    let (negative, body) = match token.as_bytes()[0] {
        b'-' => (true, &token[1..]),
        b'+' => (false, &token[1..]),
        _ => (false, token),
    };
    let first = *body.as_bytes().first()?;
    if !first.is_ascii_digit() && first != b'.' {
        return None;
    }
    if let Some(result) = integer(negative, body) {
        return Some(result);
    }
    float(negative, body)
}

fn integer(negative: bool, body: &str) -> Option<Result<Tag, &'static str>> {
    let lower = body.to_ascii_lowercase();
    let (radix, digits) = if let Some(rest) = lower.strip_prefix("0x") {
        (16, rest)
    } else if let Some(rest) = lower
        .strip_prefix("0b")
        .filter(|rest| rest.starts_with(['0', '1', '_']))
    {
        (2, rest)
    } else {
        (10, lower.as_str())
    };
    let is_digit = |c: char| c == '_' || c.is_digit(radix);
    let end = digits.find(|c: char| !is_digit(c)).unwrap_or(digits.len());
    let (number, suffix) = digits.split_at(end);
    if number.is_empty() {
        return None;
    }
    let (signedness, kind) = match suffix {
        "" => (None, IntType::Int),
        _ => {
            let mut chars = suffix.chars();
            let first = chars.next()?;
            let (signedness, rest) = match first {
                's' | 'u' if suffix.len() == 2 => (Some(first), chars.as_str()),
                _ => (None, suffix),
            };
            let kind = match rest {
                "b" => IntType::Byte,
                "s" => IntType::Short,
                "i" => IntType::Int,
                "l" => IntType::Long,
                _ => return None,
            };
            (signedness, kind)
        }
    };
    let cleaned = match strip_underscores(number) {
        Ok(cleaned) => cleaned,
        Err(key) => return Some(Err(key)),
    };
    if radix == 10 && cleaned.len() > 1 && cleaned.starts_with('0') {
        return Some(Err("snbt.parser.leading_zero_not_allowed"));
    }
    let unsigned = signedness == Some('u') || (signedness.is_none() && radix != 10);
    if signedness == Some('u') && negative {
        return Some(Err("snbt.parser.expected_non_negative_number"));
    }
    let Ok(magnitude) = u64::from_str_radix(&cleaned, radix) else {
        return Some(Err("snbt.parser.number_parse_failure"));
    };
    let bits = match kind {
        IntType::Byte => 8,
        IntType::Short => 16,
        IntType::Int => 32,
        IntType::Long => 64,
    };
    let value: i64 = if unsigned && !negative {
        if bits < 64 && magnitude >= 1u64 << bits {
            return Some(Err("snbt.parser.number_parse_failure"));
        }
        if bits == 64 {
            magnitude as i64
        } else {
            let shifted = magnitude as i64;
            if shifted >= 1i64 << (bits - 1) {
                shifted - (1i64 << bits)
            } else {
                shifted
            }
        }
    } else {
        let limit = if bits == 64 {
            i64::MAX as u64
        } else {
            (1u64 << (bits - 1)) - 1
        };
        let max = if negative { limit + 1 } else { limit };
        if magnitude > max {
            return Some(Err("snbt.parser.number_parse_failure"));
        }
        if negative {
            (magnitude as i128).wrapping_neg() as i64
        } else {
            magnitude as i64
        }
    };
    Some(Ok(match kind {
        IntType::Byte => Tag::Byte(value as i8),
        IntType::Short => Tag::Short(value as i16),
        IntType::Int => Tag::Int(value as i32),
        IntType::Long => Tag::Long(value),
    }))
}

fn float(negative: bool, body: &str) -> Option<Result<Tag, &'static str>> {
    let lower = body.to_ascii_lowercase();
    let (number, suffix) = match lower.chars().last()? {
        'f' => (&lower[..lower.len() - 1], Some('f')),
        'd' => (&lower[..lower.len() - 1], Some('d')),
        _ => (lower.as_str(), None),
    };
    let has_shape = number.contains('.') || number.contains('e');
    if suffix.is_none() && !has_shape {
        return None;
    }
    let valid = number
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | '-' | '+' | '_'));
    if !valid || number.is_empty() {
        return None;
    }
    if number.contains("__") || number.starts_with('_') || number.ends_with('_') {
        return Some(Err("snbt.parser.underscore_not_allowed"));
    }
    let cleaned = number.replace('_', "");
    let text = if negative {
        format!("-{cleaned}")
    } else {
        cleaned
    };
    Some(match suffix {
        Some('f') => match text.parse::<f32>() {
            Ok(v) if v.is_finite() => Ok(Tag::Float(v)),
            Ok(_) => Err("snbt.parser.infinity_not_allowed"),
            Err(_) => Err("snbt.parser.number_parse_failure"),
        },
        _ => match text.parse::<f64>() {
            Ok(v) if v.is_finite() => Ok(Tag::Double(v)),
            Ok(_) => Err("snbt.parser.infinity_not_allowed"),
            Err(_) => Err("snbt.parser.number_parse_failure"),
        },
    })
}

pub fn parse_compound(input: &str) -> Result<(Compound, usize), SnbtError> {
    let mut parser = Parser::new(input);
    let compound = parser.compound()?;
    Ok((compound, parser.pos))
}

pub fn parse_tag(input: &str) -> Result<Tag, SnbtError> {
    let mut parser = Parser::new(input);
    let tag = parser.value()?;
    parser.skip_ws();
    if parser.pos != input.len() {
        return parser.fail("parsing.expected.end");
    }
    Ok(tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(input: &str) -> Compound {
        parse_compound(input).map(|(c, _)| c).unwrap()
    }

    #[test]
    fn typed_numbers_and_booleans() {
        let c = compound(
            "{a:1b,b:-2s,c:3,d:4L,e:1.5f,f:2.0,g:.5d,h:true,i:false,j:1e2,k:0x1F,l:0b101,m:255ub,n:1_000}",
        );
        assert_eq!(c.get("a"), Some(&Tag::Byte(1)));
        assert_eq!(c.get("b"), Some(&Tag::Short(-2)));
        assert_eq!(c.get("c"), Some(&Tag::Int(3)));
        assert_eq!(c.get("d"), Some(&Tag::Long(4)));
        assert_eq!(c.get("e"), Some(&Tag::Float(1.5)));
        assert_eq!(c.get("f"), Some(&Tag::Double(2.0)));
        assert_eq!(c.get("g"), Some(&Tag::Double(0.5)));
        assert_eq!(c.get("h"), Some(&Tag::Byte(1)));
        assert_eq!(c.get("i"), Some(&Tag::Byte(0)));
        assert_eq!(c.get("j"), Some(&Tag::Double(100.0)));
        assert_eq!(c.get("k"), Some(&Tag::Int(31)));
        assert_eq!(c.get("l"), Some(&Tag::Int(5)));
        assert_eq!(c.get("m"), Some(&Tag::Byte(-1)));
        assert_eq!(c.get("n"), Some(&Tag::Int(1000)));
    }

    #[test]
    fn strings_lists_arrays_and_nesting() {
        let c = compound(
            r#"{Name:"Bob \"the\" cow",'q':'it\'s',Tags:[a,"b c"],B:[B;1b,2b],I:[I;1,-2],L:[L;3L],Nested:{x:[{}]},}"#,
        );
        assert_eq!(c.get_str("Name"), Some("Bob \"the\" cow"));
        assert_eq!(c.get_str("q"), Some("it's"));
        assert_eq!(
            c.get("Tags"),
            Some(&Tag::List(vec![
                Tag::String("a".into()),
                Tag::String("b c".into())
            ]))
        );
        assert_eq!(c.get("B"), Some(&Tag::ByteArray(vec![1, 2])));
        assert_eq!(c.get("I"), Some(&Tag::IntArray(vec![1, -2])));
        assert_eq!(c.get("L"), Some(&Tag::LongArray(vec![3])));
        assert!(c.get_compound("Nested").is_some());
    }

    #[test]
    fn escapes_and_operations() {
        let c =
            compound(r#"{a:"\n\t\x41é",b:bool(2),c:uuid("00000000-0000-0001-0000-000000000002")}"#);
        assert_eq!(c.get_str("a"), Some("\n\tAé"));
        assert_eq!(c.get("b"), Some(&Tag::Byte(1)));
        assert_eq!(c.get("c"), Some(&Tag::IntArray(vec![0, 1, 0, 2])));
    }

    #[test]
    fn zero_byte_is_not_a_binary_prefix() {
        let c = compound("{a:0b,b:-0b,c:0B,d:0b1b}");
        assert_eq!(c.get("a"), Some(&Tag::Byte(0)));
        assert_eq!(c.get("b"), Some(&Tag::Byte(0)));
        assert_eq!(c.get("c"), Some(&Tag::Byte(0)));
        assert_eq!(c.get("d"), Some(&Tag::Byte(1)));
        assert!(parse_compound("{a:0x}").is_err());
    }

    #[test]
    fn errors_point_at_the_problem() {
        assert!(parse_compound("{a:01}").is_err());
        assert!(parse_compound("{a:1").is_err());
        assert!(parse_compound("{:1}").is_err());
        assert!(parse_compound("{a:-x}").is_err());
        assert!(parse_compound("{a:300b}").is_err());
        assert!(parse_compound("{a:[I;1.5]}").is_err());
        assert_eq!(parse_compound("{NoAI:1b} rest").map(|(_, end)| end), Ok(9));
    }
}
