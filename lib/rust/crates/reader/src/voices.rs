//! Which of the phone's own text-to-speech voices may read a page aloud. Read-aloud promises that
//! nothing it reads is sent anywhere, so a voice that synthesises on a server, or one whose data
//! isn't installed yet (the engine would fetch it), never reads. The engine's default voice is
//! kept when it works offline; otherwise the best offline voice for its language reads; with none,
//! nothing is read.

/// One of the engine's voices, as Android describes it (`android.speech.tts.Voice`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts<'a> {
    /// Its locale's language and country (`Locale.getLanguage`, `getCountry`), as UTF-16.
    pub language: &'a [u16],
    pub country: &'a [u16],
    /// `isNetworkConnectionRequired`.
    pub needs_network: bool,
    /// Without the engine's "notInstalled" feature: its data is on the phone.
    pub installed: bool,
    /// `getQuality`, higher is better (`QUALITY_VERY_LOW` 100 to `QUALITY_VERY_HIGH` 500).
    pub quality: i32,
    /// `getLatency`, lower is better (`LATENCY_VERY_LOW` 100 to `LATENCY_VERY_HIGH` 500).
    pub latency: i32,
}

impl Facts<'_> {
    fn offline(&self) -> bool {
        !self.needs_network && self.installed
    }
}

fn same(a: &[u16], b: &[u16]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(&x, &y)| {
            let lower =
                |unit: u16| u8::try_from(unit).map_or(unit, |c| c.to_ascii_lowercase().into());
            lower(x) == lower(y)
        })
}

/// The voice to read with, as an index into `voices`: `preferred` (the engine's default, when the
/// list has it) if it works offline and speaks `language`; otherwise the offline voice for
/// `language` that speaks `country` too, then has the best quality, then the lowest latency, then
/// comes first. None when every voice for `language` needs the network or isn't installed.
pub fn offline_voice(
    voices: &[Facts],
    language: &[u16],
    country: &[u16],
    preferred: Option<usize>,
) -> Option<usize> {
    let speaks = |facts: &Facts| facts.offline() && same(facts.language, language);
    if let Some(index) = preferred
        && voices.get(index).is_some_and(speaks)
    {
        return Some(index);
    }
    (voices.iter().enumerate())
        .filter(|(_, facts)| speaks(facts))
        .min_by_key(|&(index, facts)| {
            let elsewhere = !same(facts.country, country);
            (elsewhere, -facts.quality, facts.latency, index)
        })
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Owned(Vec<u16>, Vec<u16>, bool, bool, i32, i32);

    fn voice(locale: &str, needs_network: bool, installed: bool, quality: i32) -> Owned {
        let (language, country) = locale.split_once('_').unwrap_or((locale, ""));
        let utf16 = |text: &str| text.encode_utf16().collect();
        Owned(
            utf16(language),
            utf16(country),
            needs_network,
            installed,
            quality,
            300,
        )
    }

    fn facts(owned: &[Owned]) -> Vec<Facts<'_>> {
        (owned.iter())
            .map(|o| Facts {
                language: &o.0,
                country: &o.1,
                needs_network: o.2,
                installed: o.3,
                quality: o.4,
                latency: o.5,
            })
            .collect()
    }

    fn choose(owned: &[Owned], locale: &str, preferred: Option<usize>) -> Option<usize> {
        let (language, country) = locale.split_once('_').unwrap_or((locale, ""));
        let language: Vec<u16> = language.encode_utf16().collect();
        let country: Vec<u16> = country.encode_utf16().collect();
        offline_voice(&facts(owned), &language, &country, preferred)
    }

    #[test]
    fn an_offline_default_voice_is_kept() {
        let voices = [
            voice("en_US", false, true, 300),
            voice("en_US", false, true, 500),
        ];
        assert_eq!(choose(&voices, "en_US", Some(0)), Some(0));
    }

    #[test]
    fn a_default_voice_that_needs_the_network_is_replaced_by_an_offline_one() {
        let voices = [
            voice("en_US", true, true, 500),
            voice("en_US", false, true, 300),
        ];
        assert_eq!(choose(&voices, "en_US", Some(0)), Some(1));
    }

    #[test]
    fn a_voice_not_yet_installed_never_reads() {
        let voices = [
            voice("en_US", false, false, 500),
            voice("en_US", false, true, 100),
        ];
        assert_eq!(choose(&voices, "en_US", Some(0)), Some(1));
        assert_eq!(choose(&voices[..1], "en_US", Some(0)), None);
    }

    #[test]
    fn with_only_network_voices_nothing_reads() {
        let voices = [
            voice("en_US", true, true, 500),
            voice("en_GB", true, true, 400),
        ];
        assert_eq!(choose(&voices, "en_US", Some(0)), None);
        assert_eq!(choose(&voices, "en_US", None), None);
    }

    #[test]
    fn another_language_never_reads_in_its_place() {
        let voices = [
            voice("en_US", true, true, 500),
            voice("de_DE", false, true, 500),
        ];
        assert_eq!(choose(&voices, "en_US", Some(0)), None);
    }

    #[test]
    fn the_same_country_comes_first_then_quality_then_latency() {
        let mut voices = vec![
            voice("en_GB", false, true, 500),
            voice("en_US", false, true, 300),
            voice("en_US", false, true, 400),
            voice("en_IN", false, true, 400),
        ];
        assert_eq!(choose(&voices, "en_US", None), Some(2));
        assert_eq!(choose(&voices, "en_AU", None), Some(0));
        voices[1].4 = 400;
        voices[1].5 = 200;
        assert_eq!(choose(&voices, "en_US", None), Some(1));
        voices[1].5 = 300;
        assert_eq!(
            choose(&voices, "en_US", None),
            Some(1),
            "a tie goes to the first"
        );
    }

    #[test]
    fn languages_and_countries_match_in_either_case() {
        let voices = [voice("EN_us", false, true, 300)];
        assert_eq!(choose(&voices, "en_US", None), Some(0));
    }

    #[test]
    fn a_default_the_list_lacks_is_ignored() {
        let voices = [voice("en_US", false, true, 300)];
        assert_eq!(choose(&voices, "en_US", Some(5)), Some(0));
    }
}
