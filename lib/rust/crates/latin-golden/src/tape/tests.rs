use super::*;

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("latin-golden-{}-{name}.txt", std::process::id()))
}

/// Records `answers` on a tape for `path`, writing or checking it.
fn run(path: &Path, writing: bool, answers: &[(i32, Vec<i32>)]) {
    let mut tape = Tape::at(path.to_path_buf(), "test", writing);
    for answer in answers {
        tape.record("answer", answer);
    }
    tape.finish();
}

fn answers(count: i32) -> Vec<(i32, Vec<i32>)> {
    (0..count)
        .map(|it| (it, vec![it; (it % 5) as usize]))
        .collect()
}

fn panics(run: impl FnOnce() + std::panic::UnwindSafe) -> String {
    let error = std::panic::catch_unwind(run).expect_err("a panic");
    error.downcast_ref::<String>().cloned().unwrap_or_else(|| {
        error
            .downcast_ref::<&str>()
            .map(|it| it.to_string())
            .unwrap_or_default()
    })
}

#[test]
fn a_written_tape_checks_the_same_answers() {
    let path = scratch("same");
    run(&path, true, &answers(100));
    run(&path, false, &answers(100));
    let text = std::fs::read_to_string(&path).expect("written");
    assert!(text.contains("count 100\n"));
    assert!(text.contains("\nat 64 "));
    assert!(text.contains("\nsample 63 "));
    assert!(!text.contains("\nsample 4 "));
    std::fs::remove_file(path).expect("removed");
}

#[test]
fn a_different_answer_fails_at_its_checkpoint_or_sample() {
    let path = scratch("different");
    run(&path, true, &answers(100));
    let mut changed = answers(100);
    changed[40].1.push(7);
    let message = panics(|| run(&path, false, &changed));
    assert!(message.contains("answers 32 to 63 differ"), "{message}");
    let mut changed = answers(100);
    changed[2].0 = -1;
    let message = panics(|| run(&path, false, &changed));
    assert!(message.contains("answer 2 is (-1,[2,2])"), "{message}");
    let mut changed = answers(100);
    changed[99].0 = 0;
    let message = panics(|| run(&path, false, &changed));
    assert!(message.contains("the answers differ"), "{message}");
    std::fs::remove_file(path).expect("removed");
}

#[test]
fn more_or_fewer_answers_fail() {
    let path = scratch("count");
    run(&path, true, &answers(10));
    let message = panics(|| run(&path, false, &answers(11)));
    assert!(
        message.contains("more answers than the golden 10"),
        "{message}"
    );
    let message = panics(|| run(&path, false, &answers(9)));
    assert!(
        message.contains("9 answers, the golden has 10"),
        "{message}"
    );
    std::fs::remove_file(path).expect("removed");
}

#[test]
fn a_missing_golden_file_says_where_to_look() {
    let message = panics(|| run(&scratch("missing"), false, &answers(1)));
    assert!(
        message.contains("docs/phase6-oracle-goldens.md"),
        "{message}"
    );
}

#[test]
fn floats_are_their_bits() {
    let hash = |value: f32| {
        let mut fnv = Fnv::default();
        value.feed(&mut fnv);
        fnv.finish()
    };
    assert_ne!(hash(0.0), hash(-0.0));
    assert_eq!(hash(f32::NAN), hash(f32::NAN));
    assert_ne!(hash(f32::NAN), hash(f32::from_bits(f32::NAN.to_bits() | 1)));
}
