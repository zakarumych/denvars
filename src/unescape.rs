use alloc::{borrow::Cow, string::String};

#[derive(Debug)]
pub struct EscapeError;

pub fn unescape(s: &str) -> Result<(String, Option<&str>), EscapeError> {
    let mut chars = s.chars();
    let mut result = String::new();
    while let Some(c) = chars.next() {
        if c == '\\' {
            let Some(c) = chars.next() else {
                return Err(EscapeError);
            };
            match c {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '0' => result.push('\0'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                '\'' => result.push('\''),
                'x' | 'u' | 'U' => {
                    let s = chars.as_str();
                    let hex = if s.starts_with('{') {
                        let n = s.find('}').ok_or(EscapeError)?;
                        chars.nth(n);
                        s.get(1..n)
                    } else {
                        let n = match c {
                            'x' => 2,
                            'u' => 4,
                            'U' => 8,
                            _ => unreachable!(),
                        };
                        chars.nth(n - 1);
                        s.get(0..n)
                    }
                    .ok_or(EscapeError)?;
                    if !hex.is_ascii() {
                        return Err(EscapeError);
                    }
                    let v = u32::from_str_radix(&hex, 16).map_err(|_| EscapeError)?;
                    result.push(char::from_u32(v).ok_or(EscapeError)?);
                }
                _ => return Err(EscapeError),
            }
        } else if c == '"' {
            return Ok((result, Some(chars.as_str())));
        } else {
            result.push(c);
        }
    }
    Ok((result, None))
}

pub fn unescaped(s: &str) -> Result<Cow<'_, str>, EscapeError> {
    match s.strip_prefix('"') {
        None => Ok(Cow::Borrowed(s)),
        Some(s) => {
            let (s, tail) = unescape(s)?;
            if let Some(tail) = tail {
                if tail.is_empty() {
                    Ok(Cow::Owned(s))
                } else {
                    Err(EscapeError)
                }
            } else {
                Ok(Cow::Owned(s))
            }
        }
    }
}

#[test]
fn test_unescape() {
    assert_eq!(
        unescape(r#"\x20\u0020\U00000020\x{00020}\u{20}\U{020}"#).unwrap(),
        ("      ".to_string(), None)
    );
}
