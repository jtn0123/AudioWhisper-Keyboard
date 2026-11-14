//! Reading JSON exactly as kotlinx.serialization 1.12's `Json { ignoreUnknownKeys = true }` reads
//! it into the panel's classes, so a GIPHY answer one accepts the other accepts, with the same
//! values. That isn't quite standard JSON:
//!
//! * only space, tab, `\n` and `\r` are whitespace;
//! * the members of an object needn't be separated by commas, though an array's must;
//! * an unknown key's value is skipped by matching brackets alone, so anything between them
//!   passes, and a bare value is any run of characters that aren't JSON punctuation;
//! * a string may hold raw control characters, and `\u` takes any four hex digits;
//! * a repeated key's last value wins.
//!
//! The reading follows `AbstractJsonLexer` and `StreamingJsonDecoder` step by step, so it fails
//! wherever they would throw. Where it fails doesn't matter: either way GIPHY's answer is
//! unreadable.

/// Why the JSON couldn't be read. kotlinx's message isn't kept: the panel only says the answer
/// was unreadable.
#[derive(Debug, PartialEq, Eq)]
pub struct Unreadable;

pub type Result<T> = std::result::Result<T, Unreadable>;

/// What kind of token a UTF-16 unit starts (kotlinx's `charToTokenClass`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    /// Anything else: part of a bare value.
    Other,
    String,
    Escape,
    Whitespace,
    Comma,
    Colon,
    BeginObject,
    EndObject,
    BeginList,
    EndList,
    Eof,
    /// A control character other than whitespace.
    Invalid,
}

const QUOTE: u16 = b'"' as u16;
const BACKSLASH: u16 = b'\\' as u16;

fn token(unit: u16) -> Token {
    match unit {
        0x09 | 0x0A | 0x0D | 0x20 => Token::Whitespace,
        0x00..=0x20 => Token::Invalid,
        0x2C => Token::Comma,
        0x3A => Token::Colon,
        0x7B => Token::BeginObject,
        0x7D => Token::EndObject,
        0x5B => Token::BeginList,
        0x5D => Token::EndList,
        QUOTE => Token::String,
        BACKSLASH => Token::Escape,
        _ => Token::Other,
    }
}

/// The unit an escape `\c` stands for, or `None` if it isn't one (`u` is handled apart).
fn escaped(unit: u16) -> Option<u16> {
    Some(match unit {
        0x62 => 0x08, // b
        0x74 => 0x09, // t
        0x6E => 0x0A, // n
        0x66 => 0x0C, // f
        0x72 => 0x0D, // r
        0x2F => 0x2F, // /
        QUOTE => QUOTE,
        BACKSLASH => BACKSLASH,
        _ => return None,
    })
}

fn hex(unit: u16) -> Result<u16> {
    match unit {
        0x30..=0x39 => Ok(unit - 0x30),
        0x61..=0x66 => Ok(unit - 0x61 + 10),
        0x41..=0x46 => Ok(unit - 0x41 + 10),
        _ => Err(Unreadable),
    }
}

/// JSON text being read, and how far.
#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a [u16],
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [u16]) -> Self {
        Lexer {
            source,
            position: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self
            .source
            .get(self.position)
            .is_some_and(|&it| token(it) == Token::Whitespace)
        {
            self.position += 1;
        }
    }

    /// The next token, past any whitespace, without taking it.
    pub fn peek(&mut self) -> Token {
        self.skip_whitespace();
        self.source
            .get(self.position)
            .map_or(Token::Eof, |&it| token(it))
    }

    /// Takes the next token, past any whitespace: one unit.
    pub fn take(&mut self) -> Token {
        let next = self.peek();
        if next != Token::Eof {
            self.position += 1;
        }
        next
    }

    /// Takes the next token, which must be `expected`.
    pub fn expect(&mut self, expected: Token) -> Result<()> {
        if self.take() == expected {
            Ok(())
        } else {
            Err(Unreadable)
        }
    }

    /// Fails unless only whitespace is left.
    pub fn expect_eof(&mut self) -> Result<()> {
        self.expect(Token::Eof)
    }

    /// Takes a comma if one is next.
    pub fn try_comma(&mut self) -> bool {
        let comma = self.peek() == Token::Comma;
        if comma {
            self.position += 1;
        }
        comma
    }

    /// Whether a value (or a key) could start next: anything but the end, `}`, `]`, `:` or `,`.
    pub fn can_take_value(&mut self) -> bool {
        !matches!(
            self.peek(),
            Token::Eof | Token::EndObject | Token::EndList | Token::Colon | Token::Comma
        )
    }

    /// Takes `null` if it is next. kotlinx doesn't when it starts a longer bare value (`nullx`),
    /// but a null is only ever a member's value here, and what follows one must be a key, a comma
    /// or a `}`; so the read fails at the next step instead, as kotlinx's does at this one.
    pub fn try_null(&mut self) -> bool {
        self.skip_whitespace();
        let null = self.source[self.position..].starts_with(&[0x6E, 0x75, 0x6C, 0x6C]);
        if null {
            self.position += 4;
        }
        null
    }

    /// Takes a quoted string and gives its value.
    pub fn string(&mut self) -> Result<Vec<u16>> {
        self.expect(Token::String)?;
        let mut value = Vec::new();
        loop {
            let unit = *self.source.get(self.position).ok_or(Unreadable)?;
            self.position += 1;
            match unit {
                QUOTE => return Ok(value),
                BACKSLASH => value.push(self.escape()?),
                _ => value.push(unit),
            }
        }
    }

    /// The rest of an escape, after its backslash.
    fn escape(&mut self) -> Result<u16> {
        let unit = *self.source.get(self.position).ok_or(Unreadable)?;
        self.position += 1;
        if unit != 0x75 {
            return escaped(unit).ok_or(Unreadable);
        }
        // kotlinx wants a fifth unit after the four digits; the closing quote must follow anyway.
        let digits = self
            .source
            .get(self.position..self.position + 5)
            .ok_or(Unreadable)?;
        let value = digits[..4]
            .iter()
            .try_fold(0, |value, &it| Ok(value << 4 | hex(it)?))?;
        self.position += 4;
        Ok(value)
    }

    /// Takes a string, or a bare value: a run of units that are `Token::Other`.
    fn lenient_string(&mut self) -> Result<()> {
        match self.peek() {
            Token::String => self.string().map(drop),
            Token::Other => {
                while self
                    .source
                    .get(self.position)
                    .is_some_and(|&it| token(it) == Token::Other)
                {
                    self.position += 1;
                }
                Ok(())
            }
            _ => Err(Unreadable),
        }
    }

    /// Skips an unknown key's value. An object or array is skipped by matching its brackets
    /// alone, taking strings whole and anything else a unit at a time.
    pub fn skip_value(&mut self) -> Result<()> {
        if !matches!(self.peek(), Token::BeginObject | Token::BeginList) {
            return self.lenient_string();
        }
        let mut open = Vec::new();
        loop {
            match self.peek() {
                Token::String => {
                    self.string()?;
                    continue;
                }
                Token::BeginObject | Token::BeginList => open.push(self.peek()),
                Token::EndList if open.pop() != Some(Token::BeginList) => return Err(Unreadable),
                Token::EndObject if open.pop() != Some(Token::BeginObject) => {
                    return Err(Unreadable);
                }
                Token::Eof => return Err(Unreadable),
                _ => {}
            }
            self.take();
            if open.is_empty() {
                return Ok(());
            }
        }
    }

    /// Reads an object, handing each key to `field`, which reads the value and returns `true` if
    /// it knows the key; an unknown key's value is skipped. As a class kotlinx decodes reads it.
    pub fn object(
        &mut self,
        mut field: impl FnMut(&mut Self, &[u16]) -> Result<bool>,
    ) -> Result<()> {
        self.expect(Token::BeginObject)?;
        if self.peek() == Token::Comma {
            return Err(Unreadable);
        }
        'members: loop {
            // Whether the last thing taken was a comma, which a key must follow.
            let mut comma = self.try_comma();
            while self.can_take_value() {
                let key = self.string()?;
                self.expect(Token::Colon)?;
                if field(self, &key)? {
                    continue 'members;
                }
                self.skip_value()?;
                comma = self.try_comma();
            }
            if comma {
                return Err(Unreadable);
            }
            // kotlinx's endStructure also fails on a comma here, but none can be left: the last
            // `try_comma` took any there was.
            return self.expect(Token::EndObject);
        }
    }

    /// Reads an array, `item` reading each value. Unlike an object's members, its items must be
    /// separated by commas.
    pub fn list(&mut self, mut item: impl FnMut(&mut Self) -> Result<()>) -> Result<()> {
        self.expect(Token::BeginList)?;
        if self.peek() == Token::Comma {
            return Err(Unreadable);
        }
        let mut first = true;
        loop {
            let comma = self.try_comma();
            if !self.can_take_value() {
                if comma {
                    return Err(Unreadable);
                }
                break;
            }
            if !first && !comma {
                return Err(Unreadable);
            }
            first = false;
            item(self)?;
        }
        // As in `object`, no comma can be left for endStructure to fail on.
        self.expect(Token::EndList)
    }

    /// Captures a raw value for a schema-directed consumer. Unknown members retain
    /// kotlinx's bracket-only skip behavior; the consumer validates known fields.
    pub fn raw_value(&mut self) -> Result<Vec<u16>> {
        self.skip_whitespace();
        let start = self.position;
        self.skip_value()?;
        Ok(self.source[start..self.position].to_vec())
    }

    /// Reads a `String?`: `null`, or a quoted string.
    pub fn nullable_string(&mut self) -> Result<Option<Vec<u16>>> {
        if self.try_null() {
            Ok(None)
        } else {
            self.string().map(Some)
        }
    }
}

#[cfg(test)]
mod tests;
