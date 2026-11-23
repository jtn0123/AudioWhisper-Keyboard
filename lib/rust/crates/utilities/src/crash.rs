//! What a crash leaves behind and what is shown of it, decided here for Kotlin: whether the last
//! crash's report is checked for a repeat, whether crashes come so fast the keyboard should step
//! aside ([`after`], [`same`]), and the report the crash screen shows and copies ([`report`]).
//! Android writes and reads the report files, posts the notification and draws the screen; the
//! file names stay there too, since a crash's report is written before anything else is asked.

use wire::crash::{after as steps, fact};

/// Crashes closer together than this, in ms, are a loop.
pub const LOOP_MS: i64 = 5_000;

/// What to do after a crash at `now`, the last at `last` (0 for none): [`steps::CHECK_LAST`] if
/// there was one, whose report is dropped if this one is the same, plus [`steps::LOOP`] if
/// crashes are coming too fast.
pub fn after(last: i64, now: i64) -> i32 {
    let mut flags = 0;
    if last > 0 {
        flags |= steps::CHECK_LAST;
    }
    if now.wrapping_sub(last) < LOOP_MS {
        flags |= steps::LOOP;
    }
    flags
}

/// Whether the last crash's report, as read back, is this crash's.
pub fn same(last: &[u16], this: &[u16]) -> bool {
    last == this
}

/// A report as text: the build and the phone, the debug log's header, then each stack trace
/// (named by `names`, with `details`), each in a collapsible section, as GitHub shows them. The
/// version links to its commit for a `debug` build, else to its changelog. `facts` is a
/// `CrashFact` row. Nothing is added but these inputs and the report's fixed headings.
pub fn report(
    facts: &[Vec<u16>],
    debug: bool,
    names: &[Vec<u16>],
    details: &[Vec<u16>],
) -> Option<Vec<u16>> {
    if facts.len() != fact::SIZE as usize || names.len() != details.len() {
        return None;
    }
    let at = |index: i32| facts[index as usize].as_slice();
    let link = at(if debug {
        fact::COMMIT_URL
    } else {
        fact::CHANGELOG_URL
    });
    let mut out = Text::default();
    out.put("#### Environment information").end();
    out.put("- FlorisBoard [")
        .units(at(fact::VERSION_NAME))
        .put("](")
        .units(link);
    out.put(") (").units(at(fact::VERSION_CODE)).put(")").end();
    out.put("- Device: ").units(at(fact::DEVICE)).end();
    out.put("- Android: ").units(at(fact::ANDROID)).end();
    out.end();
    out.put("#### Attached logs and stacktrace files").end();
    let header: Vec<u16> = "Detailed info (Debug log header)".encode_utf16().collect();
    out.section(&header, at(fact::DEBUG_LOG));
    out.end();
    for (name, detail) in names.iter().zip(details) {
        out.section(name, detail);
        out.end();
    }
    Some(out.0)
}

/// UTF-16 text being written; lines end with `\n`, as Kotlin's `appendLine` ends them.
#[derive(Default)]
struct Text(Vec<u16>);

impl Text {
    fn put(&mut self, text: &str) -> &mut Text {
        self.0.extend(text.encode_utf16());
        self
    }

    fn units(&mut self, text: &[u16]) -> &mut Text {
        self.0.extend_from_slice(text);
        self
    }

    fn end(&mut self) {
        self.put("\n");
    }

    /// A collapsible section, as GitHub's markdown has them.
    fn section(&mut self, summary: &[u16], details: &[u16]) {
        self.put("<details>").end();
        self.put("<summary>").units(summary).put("</summary>").end();
        self.end();
        self.put("```").end();
        self.units(details).end();
        self.put("```").end();
        self.put("</details>").end();
    }
}

#[cfg(test)]
mod tests;
