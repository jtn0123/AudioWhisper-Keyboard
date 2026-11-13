//! Domain dispatch over canonical JSON from the platform syntax decoder. UTF-16 is retained,
//! including lone surrogates; scanning unrelated values does not convert them through UTF-8.
use crate::Text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Keyboard,
    Theme,
    LanguagePack,
}
impl Kind {
    pub fn of(value: &[u16]) -> Option<Self> {
        [Self::Keyboard, Self::Theme, Self::LanguagePack]
            .into_iter()
            .find(|kind| value.iter().copied().eq(kind.serial().encode_utf16()))
    }
    pub fn serial(self) -> &'static str {
        match self {
            Self::Keyboard => "ime.extension.keyboard",
            Self::Theme => "ime.extension.theme",
            Self::LanguagePack => "ime.extension.languagepack",
        }
    }
    pub fn group(self) -> &'static str {
        match self {
            Self::Keyboard => "ime/keyboard",
            Self::Theme => "ime/theme",
            Self::LanguagePack => "ime/languagepack",
        }
    }
    pub fn component_field(self) -> &'static str {
        match self {
            Self::Keyboard => "",
            Self::Theme => "themes",
            Self::LanguagePack => "items",
        }
    }
}

fn string(input: &[u16], at: &mut usize) -> Option<Text> {
    if input.get(*at) != Some(&34) {
        return None;
    }
    *at += 1;
    let mut value = vec![];
    while let Some(&u) = input.get(*at) {
        *at += 1;
        match u {
            34 => return Some(value),
            92 => {
                let &escaped = input.get(*at)?;
                *at += 1;
                let decoded = match escaped {
                    34 | 47 | 92 => escaped,
                    98 => 8,
                    102 => 12,
                    110 => 10,
                    114 => 13,
                    116 => 9,
                    117 => {
                        let mut unit = 0u16;
                        for _ in 0..4 {
                            let &hex = input.get(*at)?;
                            *at += 1;
                            let digit = match hex {
                                48..=57 => hex - 48,
                                65..=70 => hex - 55,
                                97..=102 => hex - 87,
                                _ => return None,
                            };
                            unit = unit * 16 + digit;
                        }
                        unit
                    }
                    _ => return None,
                };
                value.push(decoded);
            }
            _ => value.push(u),
        }
    }
    None
}
fn whitespace(input: &[u16], at: &mut usize) {
    while input.get(*at).is_some_and(|u| [9, 10, 13, 32].contains(u)) {
        *at += 1;
    }
}
fn skip(input: &[u16], at: &mut usize) -> Option<()> {
    let mut depth = 0usize;
    while let Some(&u) = input.get(*at) {
        match u {
            34 => {
                string(input, at)?;
            }
            91 | 123 => {
                depth += 1;
                *at += 1;
            }
            93 | 125 => {
                if depth == 0 {
                    return Some(());
                }
                depth -= 1;
                *at += 1;
            }
            44 if depth == 0 => return Some(()),
            _ => *at += 1,
        }
    }
    Some(())
}
pub fn kind(input: &[u16]) -> Option<Kind> {
    let mut at = 0;
    whitespace(input, &mut at);
    if input.get(at) != Some(&123) {
        return None;
    }
    at += 1;
    let mut selected = None;
    loop {
        whitespace(input, &mut at);
        if input.get(at) == Some(&125) {
            return selected;
        }
        let key = string(input, &mut at)?;
        whitespace(input, &mut at);
        if input.get(at) != Some(&58) {
            return None;
        }
        at += 1;
        whitespace(input, &mut at);
        if key == [36] {
            selected = string(input, &mut at).and_then(|value| Kind::of(&value));
            // Unknown and non-string discriminators use the platform serializer's exact diagnostic.
            if input
                .get(at)
                .is_some_and(|&u| u != 44 && u != 125 && ![9, 10, 13, 32].contains(&u))
            {
                skip(input, &mut at)?;
            }
        } else {
            skip(input, &mut at)?;
        }
        whitespace(input, &mut at);
        match input.get(at) {
            Some(44) => at += 1,
            Some(125) => return selected,
            _ => return None,
        }
    }
}

/// The streaming Kotlin serializer uses the first discriminator before it falls back
/// to a JSON object. Preserve that choice even when a later duplicate overwrites the object key.
pub fn first_discriminator(input: &[u16]) -> Option<Text> {
    fn atom(input: &[u16], at: &mut usize) -> Option<Text> {
        if input.get(*at) == Some(&34) {
            return string(input, at);
        }
        let start = *at;
        while input
            .get(*at)
            .is_some_and(|u| ![9, 10, 13, 32, 44, 58, 91, 93, 123, 125].contains(u))
        {
            *at += 1;
        }
        (*at > start).then(|| input[start..*at].to_vec())
    }
    let mut at = 0;
    whitespace(input, &mut at);
    if input.get(at) != Some(&123) {
        return None;
    }
    at += 1;
    whitespace(input, &mut at);
    if atom(input, &mut at)? != [36] {
        return None;
    }
    whitespace(input, &mut at);
    if input.get(at) != Some(&58) {
        return None;
    }
    at += 1;
    whitespace(input, &mut at);
    Some(atom(input, &mut at).unwrap_or_default())
}

/// The old importer validates neither semantic versions nor a manifest schema URI. Domain
/// editing validates a nonblank version; import preserves serializer-required fields only.
pub fn accepts(requested: &[u16], kind: Kind) -> bool {
    requested.iter().copied().eq("ext-any".encode_utf16())
        || requested.iter().copied().eq(match kind {
            Kind::Keyboard => "ext-keyboard",
            Kind::Theme => "ext-theme",
            Kind::LanguagePack => "ext-languagepack",
        }
        .encode_utf16())
}
