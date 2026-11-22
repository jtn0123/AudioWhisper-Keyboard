//! Import and export through the file picker (`files.rs`). An import reads the file as
//! `importCombinedList` read it, line by line after the first, but checks every line before
//! anything is written: a line it cannot read names its number and imports nothing, where
//! Compose wrote the lines before it and showed the line itself. An export writes what
//! `exportCombinedList` wrote, byte for byte; Kotlin puts the document's last path segment in
//! front, as only it sees the document's address.
use super::Word;
use crate::settings_ui::{App, files::Picked};
use wire::settings_ui::files::{done, job, mode, result, work};
use wire::settings_ui::words::effect;

/// The picker purposes: an import's, and an export's.
pub const IMPORT: &str = "wordsImport";
pub const EXPORT: &str = "wordsExport";
/// CreateDocument's suggested name and type, and GetContent's type.
pub const EXPORT_NAME: &str = "my-personal-dictionary.clb";
pub const EXPORT_MIME: &str = "application/octet-stream";
pub const IMPORT_MIME: &str = "*/*";
/// `exportCombinedList`'s `version`.
pub const VERSION: &str = "1";

/// An import or export on its way: the dictionary it is for.
#[derive(Clone, Debug, PartialEq)]
pub struct Transfer {
    pub dictionary: String,
    pub import: bool,
}

/// `text`'s lines as `BufferedReader.readLine` gives them: ended by `\n`, `\r` or `\r\n`, the
/// last one only when it holds something.
pub fn lines(text: &str) -> Vec<&str> {
    let mut out = vec![];
    let mut start = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' || bytes[i] == b'\r' {
            out.push(&text[start..i]);
            if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
                i += 1;
            }
            start = i + 1;
        }
        i += 1;
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

/// `value` as Kotlin's `trim()` leaves it.
fn trimmed(value: &str) -> String {
    String::from_utf16_lossy(chars::trim(&settings::text(value)))
}

/// One line after the first as `importCombinedList` read it: `key=value` pairs split by `;`,
/// the keys trimmed and in any case, the values trimmed. A word and a frequency from 1 to 255
/// are needed; `all`, `null` or nothing as the locale is none; other keys are passed over.
pub fn entry(line: &str) -> Option<Word> {
    let (mut word, mut freq, mut locale, mut shortcut) = (None, None, None, None);
    for property in line.split(';') {
        let pair: Vec<&str> = property.split('=').collect();
        let [key, value] = pair[..] else {
            return None;
        };
        let units = settings::text(key);
        let is = |name: &str| chars::lower_equals_ascii(chars::trim(&units), name);
        let value = trimmed(value);
        let some = || Some(value.clone()).filter(|v| !v.is_empty());
        if is("w") || is("word") {
            word = some();
        } else if is("f") || is("freq") {
            let n = chars::text::to_int_or_null(&settings::text(&value))?;
            let [min, max] = super::editor::FREQUENCY_RANGE;
            freq = Some(n).filter(|n| (min..=max).contains(n));
            freq?;
        } else if is("l") || is("locale") {
            locale = some().filter(|v| v != "all" && v != "null");
        } else if is("s") || is("shortcut") {
            shortcut = some();
        }
    }
    Some(Word {
        id: 0,
        word: word?,
        freq: freq?,
        locale,
        shortcut,
    })
}

/// An import's words, in the file's order; the number of the first line it cannot read, the
/// first line being 1, when there is one.
pub fn import(text: &str) -> Result<Vec<Word>, usize> {
    lines(text)
        .iter()
        .enumerate()
        .skip(1)
        .map(|(n, line)| entry(line).ok_or(n + 1))
        .collect()
}

/// What an export writes after `dictionary=` and the document's last path segment: the date,
/// the package and the version, then each word as ` w=…;f=…;l=…` with `;s=…` when it has a
/// shortcut, a null locale written `null`, every line ended by `\n`.
pub fn export(now: i64, package: &str, words: &[Word]) -> String {
    let mut out = format!(";date={now};generated-by={package};version={VERSION}\n");
    for w in words {
        let locale = w.locale.as_deref().unwrap_or("null");
        out.push_str(&format!(" w={};f={};l={locale}", w.word, w.freq));
        if let Some(s) = &w.shortcut {
            out.push_str(";s=");
            out.push_str(s);
        }
        out.push('\n');
    }
    out
}

impl App {
    /// Import's picker, as GetContent opened it.
    pub(super) fn start_import(&mut self, d: &str) {
        self.words.transfer = Some(Transfer {
            dictionary: d.into(),
            import: true,
        });
        self.pick(IMPORT, mode::OPEN, &[IMPORT_MIME], "");
    }
    /// Export's picker, as CreateDocument opened it. The dictionary is read again meanwhile, as
    /// Compose read it when the document was picked.
    pub(super) fn start_export(&mut self, d: &str) {
        self.words.transfer = Some(Transfer {
            dictionary: d.into(),
            import: false,
        });
        self.words.requested = None;
        self.request_words(d);
        self.pick(EXPORT, mode::CREATE, &[EXPORT_MIME], EXPORT_NAME);
    }
    /// The picker closed for an import or export; false when `purpose` is not theirs.
    pub(in crate::settings_ui) fn words_picked(&mut self, purpose: &str, answer: &Picked) -> bool {
        if purpose != IMPORT && purpose != EXPORT {
            return false;
        }
        let Some(t) = self.words.transfer.clone() else {
            return true;
        };
        match answer {
            Picked::Documents(d) if t.import => {
                self.work(job::WORDS_READ, &[(work::DOCUMENT, d[0].handle.clone())]);
            }
            Picked::Documents(d) => {
                let words = self
                    .words
                    .report
                    .as_ref()
                    .filter(|r| r.dictionary == t.dictionary)
                    .map(|r| r.words.clone())
                    .unwrap_or_default();
                let text = export((self.now)(), self.fact("package"), &words);
                let fields = [(work::DOCUMENT, d[0].handle.clone()), (work::TEXT, text)];
                self.work(job::WORDS_WRITE, &fields);
            }
            // Compose said nothing when the picker was dismissed.
            Picked::Cancelled => self.words.transfer = None,
            Picked::NoPicker => {
                self.words.transfer = None;
                self.toast(self.label("settings__udm__no_picker"));
            }
        }
        true
    }
    /// File work ended; false when it is not an import's or export's.
    pub(in crate::settings_ui) fn words_done(&mut self, p: &[String]) -> bool {
        let read = p[done::JOB] == job::WORDS_READ;
        if !read && p[done::JOB] != job::WORDS_WRITE {
            return false;
        }
        let transfer = self.words.transfer.take();
        if p[done::RESULT] != result::OK {
            self.toast(self.label("settings__udm__file_error"));
        } else if !read {
            self.toast(self.label("settings__udm__dictionary_export_success"));
        } else if let Some(t) = transfer {
            match import(&p[done::TEXT]) {
                Ok(words) => {
                    let mut packet = vec![effect::IMPORT.to_owned(), t.dictionary];
                    packet.extend(words.iter().flat_map(super::encode));
                    self.effects.push(packet);
                }
                Err(line) => {
                    let message = self.label("settings__udm__import_line_error");
                    self.toast(message.replace("{line}", &line.to_string()));
                }
            }
        }
        true
    }
    /// Work Rust waited for was lost with the activity; false when it was not words'.
    pub(in crate::settings_ui) fn words_lost(&mut self, lost: &str) -> bool {
        if lost != job::WORDS_READ && lost != job::WORDS_WRITE {
            return false;
        }
        self.words.transfer = None;
        self.toast(self.label("settings__files__interrupted"));
        true
    }
}
