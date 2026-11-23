use wire::crash::{after as steps, fact};

use super::*;

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn facts() -> Vec<Vec<u16>> {
    ["1.2", "12", "commit", "changelog", "Pixel", "15", "log"]
        .iter()
        .map(|it| utf16(it))
        .collect()
}

#[test]
fn a_crash_soon_after_the_last_is_a_loop_and_the_last_is_checked() {
    assert_eq!(after(0, 10_000), 0);
    assert_eq!(after(0, 4_999), steps::LOOP);
    assert_eq!(after(1_000, 5_999), steps::CHECK_LAST | steps::LOOP);
    assert_eq!(after(1_000, 6_000), steps::CHECK_LAST);
    assert_eq!(after(9_000, 1_000), steps::CHECK_LAST | steps::LOOP);
    assert!(same(&utf16("trace\n"), &utf16("trace\n")));
    assert!(!same(&utf16("trace"), &utf16("trace\n")));
}

#[test]
fn a_report_has_the_build_the_phone_the_log_and_each_trace() {
    let report = report(&facts(), false, &[utf16("1.stacktrace")], &[utf16("boom")]).unwrap();
    let expected = "#### Environment information\n- FlorisBoard [1.2](changelog) (12)\n\
        - Device: Pixel\n- Android: 15\n\n#### Attached logs and stacktrace files\n\
        <details>\n<summary>Detailed info (Debug log header)</summary>\n\n```\nlog\n```\n</details>\n\n\
        <details>\n<summary>1.stacktrace</summary>\n\n```\nboom\n```\n</details>\n\n";
    assert_eq!(String::from_utf16(&report).unwrap(), expected);
    let debug = report_text(true);
    assert!(debug.contains("[1.2](commit)") && !debug.contains("changelog"));
}

fn report_text(debug: bool) -> String {
    String::from_utf16(&report(&facts(), debug, &[], &[]).unwrap()).unwrap()
}

#[test]
fn a_report_without_traces_ends_after_the_log() {
    assert!(report_text(false).ends_with("```\n</details>\n\n"));
}

#[test]
fn rows_of_the_wrong_length_are_refused() {
    assert_eq!(report(&facts()[1..], false, &[], &[]), None);
    let mut long = facts();
    long.push(utf16("extra"));
    assert_eq!(report(&long, false, &[], &[]), None);
    assert_eq!(report(&facts(), false, &[utf16("a")], &[]), None);
    assert_eq!(fact::SIZE as usize, facts().len());
}
