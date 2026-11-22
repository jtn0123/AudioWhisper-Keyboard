//! Import and export: reading lines as `readLine` does, each line as `importCombinedList` read
//! it, the bad line's number, old files, the export's exact bytes, and the picker's round trip.
use crate::common::*;
use render::settings_ui::{
    catalogue::words::{Act, Part},
    words::{
        FLORIS, SYSTEM,
        transfer::{EXPORT_MIME, EXPORT_NAME, IMPORT_MIME, entry, export, import, lines},
    },
};
use wire::settings_ui::files::{
    document, done, effect as files, job, mode, outcome as picked_how, pick, picked, report,
    result, work,
};
use wire::settings_ui::words::{dictionary, effect, outcome};

/// A file as `exportCombinedList` wrote it, the dictionary's three words in `queryAll()` order.
const EXPORTED: &str = "dictionary=document:1234;date=1800000000000;generated-by=com.jtn0123.audiowhisper.keyboard;version=1\n w=zorblat;f=200;l=en_US\n w=Quexor;f=120;l=en-us;s=qx\n w=blivet;f=255;l=null\n";

fn three() -> Vec<render::settings_ui::words::Word> {
    sample()[..3].to_vec()
}
fn picked_report(how: &str, handles: &[&str]) -> Vec<String> {
    let mut p = vec![String::new(); picked::SIZE];
    p[picked::REPORT] = report::PICKED.into();
    p[picked::OUTCOME] = how.into();
    for h in handles {
        let mut d = vec![String::new(); document::SIZE];
        d[document::HANDLE] = (*h).into();
        p.extend(d);
    }
    p
}
fn done_report(kind: &str, how: &str, text: &str) -> Vec<String> {
    let mut p = vec![String::new(); done::SIZE];
    p[done::REPORT] = report::DONE.into();
    p[done::JOB] = kind.into();
    p[done::RESULT] = how.into();
    p[done::TEXT] = text.into();
    p
}
fn pick_packet(how: &str, mime: &str, name: &str) -> Vec<String> {
    let mut p = vec![String::new(); pick::SIZE];
    p[pick::EFFECT] = files::PICK.into();
    p[pick::MODE] = how.into();
    p[pick::MIME] = mime.into();
    p[pick::NAME] = name.into();
    p
}
fn work_packet(kind: &str, text: &str) -> Vec<String> {
    let mut p = vec![String::new(); work::SIZE];
    p[work::EFFECT] = files::WORK.into();
    p[work::JOB] = kind.into();
    p[work::DOCUMENT] = "0".into();
    p[work::TEXT] = text.into();
    p
}
/// Taps action `act` on the first screen.
fn act(a: &mut render::settings_ui::App, text: &mut render::text::Text, act: Act) {
    let id = part_id(a, Part::Action(act));
    assert!(tap(a, text, id));
}

#[test]
fn lines_end_as_buffered_reader_read_line_ends_them() {
    assert_eq!(lines("a\nb\r\nc\rd"), ["a", "b", "c", "d"]);
    assert_eq!(lines("a\n"), ["a"]);
    assert_eq!(lines("a\n\nb"), ["a", "", "b"]);
    assert_eq!(lines("\n"), [""]);
    assert_eq!(lines("\r\r\n"), ["", ""]);
    assert!(lines("").is_empty());
}

#[test]
fn a_line_is_read_as_import_combined_list_read_it() {
    let w = |t: &str, f: i32, l: Option<&str>, s: Option<&str>| Some(word(0, t, f, l, s));
    let cases = [
        (
            " w=blorp;f=12;l=en_US;s=bp",
            w("blorp", 12, Some("en_US"), Some("bp")),
        ),
        (" WORD = blorp ; Freq= 3 ", w("blorp", 3, None, None)),
        ("locale=all;w=a;f=1", w("a", 1, None, None)),
        ("w=a;f=1;l=null", w("a", 1, None, None)),
        ("w=a;f=1;l=;s=", w("a", 1, None, None)),
        ("w=a;f=1;l=all ", w("a", 1, None, None)),
        ("w=a;f=1;x=y", w("a", 1, None, None)),
        ("w=a;f=1;shortcut=q;l=de", w("a", 1, Some("de"), Some("q"))),
        ("w=a;f=7;f=255", w("a", 255, None, None)),
        ("w=a;f=0", None),
        ("w=a;f=256", None),
        ("w=a;f=many", None),
        ("w=a;f=", None),
        ("w=;f=1", None),
        ("f=1", None),
        ("w=a", None),
        ("w=a=b;f=1", None),
        ("w=a;f=1;", None),
        ("w=a;f=1;key", None),
        ("", None),
    ];
    for (line, expected) in cases {
        assert_eq!(entry(line), expected, "case {line:?}");
    }
}

#[test]
fn an_import_skips_the_first_line_and_names_the_first_it_cannot_read() {
    let ok = import("anything at all\n w=a;f=1\r\n w=b;f=2;l=de\n").unwrap();
    assert_eq!(
        ok,
        [
            word(0, "a", 1, None, None),
            word(0, "b", 2, Some("de"), None)
        ]
    );
    assert_eq!(import("header\n w=a;f=1\n w=b;f=999\n w=c"), Err(3));
    assert_eq!(
        import("header\n w=a;f=1\n\n w=b;f=2"),
        Err(3),
        "an empty line"
    );
    assert_eq!(import("header only"), Ok(vec![]));
    assert_eq!(import(""), Ok(vec![]));
}

#[test]
fn old_export_files_import_and_a_new_export_is_byte_for_byte_the_old() {
    let read = import(EXPORTED).unwrap();
    let mut expected = three();
    for w in &mut expected {
        w.id = 0;
    }
    // `null` was written for no locale and reads back as none.
    expected[2].locale = None;
    assert_eq!(read, expected);
    let written = format!("dictionary=document:1234{}", export(NOW, PACKAGE, &three()));
    assert_eq!(written, EXPORTED);
    assert_eq!(
        export(NOW, PACKAGE, &[]),
        ";date=1800000000000;generated-by=com.jtn0123.audiowhisper.keyboard;version=1\n"
    );
}

#[test]
fn import_picks_a_file_reads_it_and_sends_every_word_in_one_effect() {
    let (mut a, mut text) = app(&[FLORIS], &sample());
    act(&mut a, &mut text, Act::Import);
    assert_eq!(a.effects, [pick_packet(mode::OPEN, IMPORT_MIME, "")]);
    a.effects.clear();
    a.file_report(&picked_report(picked_how::PICKED, &["0"]))
        .unwrap();
    assert_eq!(a.effects, [work_packet(job::WORDS_READ, "")]);
    a.scene(&mut text);
    for busy in [Act::Import, Act::Export] {
        assert!(
            !control(&a, part_id(&a, Part::Action(busy)))
                .unwrap()
                .enabled
        );
    }
    a.effects.clear();
    a.file_report(&done_report(job::WORDS_READ, result::OK, EXPORTED))
        .unwrap();
    let mut words = import(EXPORTED).unwrap();
    assert_eq!(
        a.effects,
        [packet(effect::IMPORT, dictionary::FLORIS, &words)]
    );
    assert!(a.words.transfer.is_none() && a.files.busy.is_none());
    a.effects.clear();
    a.strings.insert(
        "settings__udm__dictionary_import_success".into(),
        "Imported".into(),
    );
    words.push(word(9, "new", 1, None, None));
    a.set_words(&report(dictionary::FLORIS, true, outcome::IMPORTED, &words))
        .unwrap();
    assert_eq!(toasts(&a), ["Imported"]);
}

#[test]
fn a_bad_line_imports_nothing_and_says_which() {
    let (mut a, mut text) = app(&[SYSTEM], &sample());
    act(&mut a, &mut text, Act::Import);
    a.file_report(&picked_report(picked_how::PICKED, &["0"]))
        .unwrap();
    a.effects.clear();
    let file = format!("{EXPORTED} w=bad;f=0\n");
    a.file_report(&done_report(job::WORDS_READ, result::OK, &file))
        .unwrap();
    assert!(word_effects(&a).is_empty());
    assert_eq!(
        toasts(&a),
        ["Line 5 of that file is not a word. Nothing was imported."]
    );
}

#[test]
fn export_reads_the_dictionary_again_and_writes_what_compose_wrote() {
    let (mut a, mut text) = app(&[FLORIS], &sample());
    act(&mut a, &mut text, Act::Export);
    assert_eq!(
        a.effects,
        [
            vec![effect::LOAD.to_string(), dictionary::FLORIS.into()],
            pick_packet(mode::CREATE, EXPORT_MIME, EXPORT_NAME),
        ]
    );
    a.set_words(&report(dictionary::FLORIS, true, outcome::NONE, &three()))
        .unwrap();
    a.effects.clear();
    a.file_report(&picked_report(picked_how::PICKED, &["0"]))
        .unwrap();
    let body = export(NOW, PACKAGE, &three());
    assert_eq!(a.effects, [work_packet(job::WORDS_WRITE, &body)]);
    a.effects.clear();
    a.strings.insert(
        "settings__udm__dictionary_export_success".into(),
        "Exported".into(),
    );
    a.file_report(&done_report(job::WORDS_WRITE, result::OK, ""))
        .unwrap();
    assert_eq!(toasts(&a), ["Exported"]);
}

#[test]
fn a_dismissed_or_missing_picker_and_failed_or_lost_file_work_are_plainly_told() {
    let (mut a, mut text) = app(&[FLORIS], &sample());
    act(&mut a, &mut text, Act::Import);
    a.effects.clear();
    a.file_report(&picked_report(picked_how::CANCELLED, &[]))
        .unwrap();
    assert!(a.effects.is_empty() && a.words.transfer.is_none());
    act(&mut a, &mut text, Act::Export);
    a.effects.clear();
    a.file_report(&picked_report(picked_how::NO_PICKER, &[]))
        .unwrap();
    assert_eq!(toasts(&a), ["settings__udm__no_picker"]);
    for (kind, how) in [
        (job::WORDS_READ, result::UNREADABLE),
        (job::WORDS_WRITE, result::GONE),
    ] {
        a.effects.clear();
        act(
            &mut a,
            &mut text,
            if kind == job::WORDS_READ {
                Act::Import
            } else {
                Act::Export
            },
        );
        a.file_report(&picked_report(picked_how::PICKED, &["0"]))
            .unwrap();
        a.effects.clear();
        a.file_report(&done_report(kind, how, "")).unwrap();
        assert_eq!(toasts(&a), ["settings__udm__file_error"], "{kind}");
    }
    a.effects.clear();
    act(&mut a, &mut text, Act::Import);
    a.file_report(&picked_report(picked_how::PICKED, &["0"]))
        .unwrap();
    a.effects.clear();
    let mut idle = vec![String::new(); 1];
    idle[0] = report::IDLE.into();
    a.file_report(&idle).unwrap();
    assert_eq!(toasts(&a), ["settings__files__interrupted"]);
    assert!(a.words.transfer.is_none() && a.files.busy.is_none());
}
