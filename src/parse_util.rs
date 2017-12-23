use std::str::FromStr;

pub type ParseResult<'a, T> = Result<(Buffer<'a>, T), ParseError>;
pub type ParseSuccess<'a> = Result<Buffer<'a>, ParseError>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Buffer<'a> {
    pub text: &'a str,
    pub row: usize,
    pub col: usize,
}

// Note: These implementations aren't fully general and assume that text is one line only
impl<'a> Buffer<'a> {
    pub fn advance(&self, offset: usize) -> Buffer<'a> {
        let offset = ::std::cmp::min(offset, self.text.len());
        Buffer {
            text: &self.text[offset..],
            row: self.row,
            col: self.col + offset,
        }
    }

    pub fn trim_left(&self) -> Buffer<'a> {
        match self.text.find(|x: char| !x.is_whitespace()) {
            Some(offset) => self.advance(offset),
            None => self.advance(self.text.len()),
        }
    }

    pub fn trim_right(&self) -> Buffer<'a> {
        Buffer {
            text: self.text.trim_right(),
            ..*self
        }
    }

    pub fn trim(&self) -> Buffer<'a> {
        self.trim_left().trim_right()
    }

    pub fn end(&self) -> ParseSuccess<'a> {
        if self.text.is_empty() {
            Ok(*self)
        } else {
            Err(self.expected("end of input"))
        }
    }

    pub fn space(&self) -> ParseSuccess<'a> {
        let new_input = self.trim_left();
        if new_input == *self {
            Err(self.expected("whitespace"))
        } else {
            Ok(new_input)
        }
    }

    pub fn space_or_end(&self) -> ParseSuccess<'a> {
        if self.text.is_empty() {
            Ok(*self)
        } else {
            self.space()
        }
    }

    pub fn token<S: AsRef<str>>(&self, token: S) -> ParseSuccess<'a> {
        let token = token.as_ref();
        if self.starts_with(token) {
            Ok(self.advance(token.len()))
        } else {
            Err(self.expected(format!("\"{}\"", token)))
        }
    }

    pub fn first_token_of(&self, tokens: &[&str]) -> ParseResult<'a, &'a str> {
        if tokens.is_empty() {
            return Ok((*self, ""));
        }

        for token in tokens {
            if self.starts_with(token) {
                return Ok((self.advance(token.len()), &self.text[..token.len()]));
            }
        }

        Err(self.first_token_err(tokens))
    }

    fn first_token_err(&self, tokens: &[&str]) -> ParseError {
        if tokens.len() == 1 {
            self.expected(format!("\"{}\"", tokens[0]))
        } else if tokens.len() == 2 {
            self.expected(format!("either \"{}\" or \"{}\"", tokens[0], tokens[1]))
        } else {
            let prefix = tokens[..tokens.len() - 1]
                .iter()
                .map(|x| format!("\"{}\"", x))
                .collect::<Vec<_>>()
                .join(", ");
            let last = tokens.last().unwrap();
            self.expected(format!("one of {}, or \"{}\"", prefix, last))
        }
    }

    pub fn starts_with<S: AsRef<str>>(&self, prefix: S) -> bool {
        self.text.starts_with(prefix.as_ref())
    }

    pub fn til<P: Fn(char) -> bool>(&self, pat: P) -> ParseResult<'a, &'a str> {
        let idx = self.text.find(pat);
        if let Some(offset) = idx {
            Ok((self.advance(offset), &self.text[..offset]))
        } else {
            Ok((self.advance(self.text.len()), self.text))
        }
    }

    pub fn parse_til<T: FromStr, P: Fn(char) -> bool>(&self, pat: P) -> ParseResult<'a, T>
    where
        <T as FromStr>::Err: ::std::error::Error,
    {
        let (buf, text) = self.til(pat)?;
        Ok((
            buf,
            text.parse().map_err(|err| {
                self.expected(format!("error parsing token: {}, error {}", text, err))
            })?,
        ))
    }

    pub fn expected<S: Into<String>>(&self, message: S) -> ParseError {
        ParseError::expected(message, self.row, self.col)
    }
}

pub type Span = Option<(usize, usize)>;

#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    Expected { msg: String, row: usize, span: Span },
}

impl ParseError {
    pub fn expected<St: Into<String>, Sp: IntoSpan>(msg: St, row: usize, span: Sp) -> Self {
        ParseError::Expected {
            msg: msg.into(),
            row,
            span: span.into_span(),
        }
    }
}

pub trait IntoSpan {
    fn into_span(self) -> Span;
}

impl IntoSpan for usize {
    fn into_span(self) -> Span {
        Some((self, self))
    }
}

impl IntoSpan for (usize, usize) {
    fn into_span(self) -> Span {
        Some(self)
    }
}

impl IntoSpan for Option<()> {
    fn into_span(self) -> Span {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_advance() {
        let input = Buffer {
            row: 1,
            col: 0,
            text: "Hello",
        };

        assert_eq!(
            input.advance(1),
            Buffer {
                row: 1,
                col: 1,
                text: "ello",
            }
        );
        assert_eq!(input.advance(0), input);
        assert_eq!(
            input.advance(5),
            Buffer {
                row: 1,
                col: 5,
                text: "",
            }
        );
        assert_eq!(
            input.advance(10),
            Buffer {
                row: 1,
                col: 5,
                text: "",
            }
        );
    }

    #[test]
    fn test_trim() {
        let input = Buffer {
            row: 1,
            col: 2,
            text: "  Hello  ",
        };

        assert_eq!(
            input.trim_left(),
            Buffer {
                row: 1,
                col: 4,
                text: "Hello  ",
            }
        );
        assert_eq!(
            input.trim_right(),
            Buffer {
                row: 1,
                col: 2,
                text: "  Hello",
            }
        );
        assert_eq!(
            input.trim(),
            Buffer {
                row: 1,
                col: 4,
                text: "Hello",
            }
        );

        // Idempotent
        assert_eq!(input.trim_left(), input.trim_left().trim_left());
        assert_eq!(input.trim_right(), input.trim_right().trim_right());
        assert_eq!(input.trim(), input.trim().trim());
    }

    #[test]
    fn test_space() {
        let good_input = Buffer {
            row: 0,
            col: 0,
            text: "  Consume the space",
        };

        assert_eq!(
            good_input.space(),
            Ok(Buffer {
                row: 0,
                col: 2,
                text: "Consume the space",
            })
        );

        let bad_input = Buffer {
            row: 0,
            col: 0,
            text: "No space here",
        };

        assert!(bad_input.space().is_err());
        assert!(bad_input.space_or_end().is_err());

        let is_end = Buffer {
            row: 7,
            col: 42,
            text: "",
        };

        assert!(is_end.space().is_err());
        assert_eq!(is_end.space_or_end(), Ok(is_end));
    }

    #[test]
    fn test_token() {
        let input = Buffer {
            row: 0,
            col: 0,
            text: "Token",
        };
        let empty = Buffer {
            row: 0,
            col: 5,
            text: "",
        };
        let en_input = Buffer {
            row: 0,
            col: 3,
            text: "en",
        };

        assert_eq!(input.token("Token"), Ok(empty));
        assert!(input.token("token").is_err());
        assert_eq!(input.token("Tok"), Ok(en_input));
    }

    #[test]
    fn test_first_token() {
        let input = Buffer {
            row: 0,
            col: 0,
            text: "Tokens",
        };
        let empty = Buffer {
            row: 0,
            col: 6,
            text: "",
        };
        let s_input = Buffer {
            row: 0,
            col: 5,
            text: "s",
        };

        assert_eq!(
            input.first_token_of(&["Tokens", "Token"]),
            Ok((empty, "Tokens"))
        );
        assert_eq!(
            input.first_token_of(&["Token", "Tokens"]),
            Ok((s_input, "Token"))
        );
        assert_eq!(input.first_token_of(&[]), Ok((input, "")));

        // Error messages should be correct
        assert_eq!(
            input.first_token_of(&["Meow"]).unwrap_err(),
            input.expected("\"Meow\"")
        );
        assert_eq!(
            input.first_token_of(&["Meow", "Bark"]).unwrap_err(),
            input.expected("either \"Meow\" or \"Bark\"")
        );
        assert_eq!(
            input.first_token_of(&["Meow", "Bark", "Moo"]).unwrap_err(),
            input.expected("one of \"Meow\", \"Bark\", or \"Moo\"")
        );
    }

    #[test]
    fn test_starts_with() {
        let input = Buffer {
            row: 0,
            col: 0,
            text: "Starts with",
        };

        assert!(input.starts_with("Starts"));
        assert!(!input.starts_with("starts"));
        assert!(input.starts_with("Starts with"));
        assert!(input.starts_with("Star"));
        assert!(!input.starts_with("X"));
    }

    #[test]
    fn test_starts_with_multibyte() {
        const HEART: &'static str = "\u{1F49C}";
        let heart_emoji = Buffer {
            row: 0,
            col: 0,
            text: HEART,
        };

        assert!(heart_emoji.starts_with(HEART));
        assert!(!heart_emoji.starts_with("x"));

        let heart = Buffer {
            row: 0,
            col: 0,
            text: "heart",
        };

        assert!(!heart.starts_with(HEART));
    }
}
