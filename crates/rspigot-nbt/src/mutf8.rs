use std::borrow::Cow;

pub fn encode(s: &str) -> Cow<'_, [u8]> {
    if s.bytes().all(|b| b != 0 && b < 0x80) {
        return Cow::Borrowed(s.as_bytes());
    }
    let mut out = Vec::with_capacity(s.len() + 8);
    for ch in s.chars() {
        let mut units = [0u16; 2];
        for unit in ch.encode_utf16(&mut units) {
            push_unit(&mut out, *unit);
        }
    }
    Cow::Owned(out)
}

fn push_unit(out: &mut Vec<u8>, unit: u16) {
    match unit {
        0x0001..=0x007F => out.push(unit as u8),
        0x0000 | 0x0080..=0x07FF => {
            out.push(0xC0 | ((unit >> 6) & 0x1F) as u8);
            out.push(0x80 | (unit & 0x3F) as u8);
        }
        _ => {
            out.push(0xE0 | ((unit >> 12) & 0x0F) as u8);
            out.push(0x80 | ((unit >> 6) & 0x3F) as u8);
            out.push(0x80 | (unit & 0x3F) as u8);
        }
    }
}

pub fn decode(bytes: &[u8]) -> Option<Cow<'_, str>> {
    if bytes.iter().all(|b| *b != 0 && *b < 0x80) {
        return std::str::from_utf8(bytes).ok().map(Cow::Borrowed);
    }
    let mut units = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        if b0 & 0x80 == 0 {
            units.push(u16::from(b0));
            i += 1;
        } else if b0 & 0xE0 == 0xC0 {
            let b1 = *bytes.get(i + 1)?;
            units.push((u16::from(b0 & 0x1F) << 6) | u16::from(b1 & 0x3F));
            i += 2;
        } else if b0 & 0xF0 == 0xE0 {
            let b1 = *bytes.get(i + 1)?;
            let b2 = *bytes.get(i + 2)?;
            units.push(
                (u16::from(b0 & 0x0F) << 12) | (u16::from(b1 & 0x3F) << 6) | u16::from(b2 & 0x3F),
            );
            i += 3;
        } else {
            return None;
        }
    }
    String::from_utf16(&units).ok().map(Cow::Owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_and_supplementary_roundtrip() {
        let s = "a\0b\u{1F600}é";
        let encoded = encode(s);
        assert!(!encoded.contains(&0));
        assert_eq!(decode(&encoded).unwrap(), s);
    }
}
