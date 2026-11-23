use wire::flag;
use wire::gif::{self as ops, effect, reply, results as status};

use super::Session;
use crate::chips::{RECENTS, TRENDING};
use crate::giphy::Gif;
use crate::rows::{self, text};
use crate::utf16;

/// One effect, read back.
#[derive(Debug, PartialEq)]
enum Effect {
    Results(i32, String, Vec<String>),
    Chips(Vec<String>),
    Fetch(i32, String),
    Cancel(i32),
    Read(i32, String),
    Save(i32),
    Forget,
}

struct Plan {
    showing: bool,
    typing: bool,
    query: String,
    shown: String,
    effects: Vec<Effect>,
}

fn string(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

fn number(units: &[u16]) -> i32 {
    string(units).parse().unwrap()
}

fn effects(rest: &[Vec<u16>]) -> Vec<Effect> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < rest.len() {
        let code = string(&rest[at]);
        let field = |i: usize| &rest[at + i];
        let (effect, used) = match code.as_str() {
            effect::RESULTS => {
                let count = number(field(3)) as usize;
                let ids = (0..count).map(|i| string(field(4 + i * 6))).collect();
                (
                    Effect::Results(number(field(1)), string(field(2)), ids),
                    4 + count * 6,
                )
            }
            effect::CHIPS => {
                let count = number(field(1)) as usize;
                (
                    Effect::Chips((0..count).map(|i| string(field(2 + i))).collect()),
                    2 + count,
                )
            }
            effect::FETCH => (Effect::Fetch(number(field(1)), string(field(2))), 6),
            effect::CANCEL => (Effect::Cancel(number(field(1))), 2),
            effect::READ => (Effect::Read(number(field(1)), string(field(2))), 3),
            effect::SAVE => (Effect::Save(number(field(1))), 2),
            effect::FORGET_PREVIEWS => (Effect::Forget, 1),
            other => panic!("no effect {other}"),
        };
        out.push(effect);
        at += used;
    }
    out
}

fn call(session: &mut Session, op: &str, now: i64, fields: &[&str]) -> Plan {
    let fields: Vec<Vec<u16>> = fields.iter().map(|it| utf16(it)).collect();
    let answer = session.call(op, now, &fields).unwrap();
    Plan {
        showing: flag::decode(&answer[0]),
        typing: flag::decode(&answer[1]),
        query: string(&answer[2]),
        shown: string(&answer[3]),
        effects: effects(&answer[4..]),
    }
}

fn gif(id: &str) -> Gif {
    Gif {
        id: utf16(id),
        title: utf16(&format!("title {id}")),
        preview_url: utf16(&format!("p/{id}")),
        aspect_ratio: 1.5,
        send_url: utf16(&format!("s/{id}")),
        page_url: utf16(&format!("u/{id}")),
    }
}

fn body(ids: &[&str]) -> String {
    let gifs: Vec<String> = ids
        .iter()
        .map(|id| {
            format!(
                r#"{{"id":"{id}","title":"t","url":"u/{id}","images":{{"fixed_width":{{"url":"p/{id}","width":"200","height":"100"}},"original":{{"url":"s/{id}"}}}}}}"#
            )
        })
        .collect();
    format!(r#"{{"data":[{}]}}"#, gifs.join(","))
}

fn available() -> Session {
    let mut session = Session::new();
    call(&mut session, ops::SOURCE, 0, &[flag::TRUE, "KEY"]);
    session
}

fn fetched(session: &mut Session, generation: i32, now: i64, ids: &[&str]) -> Plan {
    let generation = generation.to_string();
    call(
        session,
        ops::FETCHED,
        now,
        &[&generation, reply::OK, "200", &body(ids)],
    )
}

fn loaded(ids: &[&str]) -> Effect {
    Effect::Results(
        status::LOADED,
        String::new(),
        ids.iter().map(|it| it.to_string()).collect(),
    )
}

fn loading() -> Effect {
    Effect::Results(status::LOADING, String::new(), Vec::new())
}

#[test]
fn a_search_is_fetched_once_and_kept_ten_minutes() {
    let mut session = available();
    let plan = call(&mut session, ops::SHOW, 0, &[flag::TRUE]);
    assert!(plan.showing);
    assert_eq!(plan.effects[0], loading());
    let Effect::Fetch(1, url) = &plan.effects[1] else {
        panic!("{:?}", plan.effects)
    };
    assert!(url.contains("/trending?") && url.contains("api_key=KEY"));
    assert_eq!(
        fetched(&mut session, 1, 5, &["a", "b"]).effects,
        vec![loaded(&["a", "b"])]
    );
    assert!(
        call(&mut session, ops::LOAD, 6, &[TRENDING])
            .effects
            .is_empty()
    );
    let plan = call(&mut session, ops::LOAD, 7, &["cats"]);
    assert_eq!(plan.shown, "cats");
    assert!(matches!(&plan.effects[1], Effect::Fetch(2, url) if url.contains("q=cats")));
    let plan = call(&mut session, ops::LOAD, 8, &[TRENDING]);
    assert_eq!(plan.effects, vec![Effect::Cancel(2), loaded(&["a", "b"])]);
    assert!(fetched(&mut session, 2, 9, &["late"]).effects.is_empty());
    call(&mut session, ops::LOAD, 10, &["dogs"]);
    let plan = call(&mut session, ops::LOAD, 5 + 600_000, &[TRENDING]);
    assert_eq!(plan.effects[0], Effect::Cancel(3));
    assert!(matches!(plan.effects[2], Effect::Fetch(4, _)));
}

#[test]
fn a_failure_is_shown_and_tried_again() {
    let mut session = available();
    call(&mut session, ops::LOAD, 0, &["cats"]);
    let plan = call(
        &mut session,
        ops::FETCHED,
        1,
        &["1", reply::STATUS, "429", ""],
    );
    let Effect::Results(status::FAILED, problem, _) = &plan.effects[0] else {
        panic!()
    };
    assert!(problem.contains("hourly limit"));
    let plan = call(&mut session, ops::LOAD, 2, &["cats"]);
    assert!(matches!(plan.effects[1], Effect::Fetch(2, _)));
    let plan = call(
        &mut session,
        ops::FETCHED,
        3,
        &["2", reply::OK, "200", "nope"],
    );
    assert!(
        matches!(&plan.effects[0], Effect::Results(status::FAILED, p, _) if p.contains("unreadable"))
    );
    call(&mut session, ops::LOAD, 4, &["cats"]);
    let plan = call(
        &mut session,
        ops::FETCHED,
        5,
        &["3", reply::UNREACHABLE, "0", ""],
    );
    assert!(
        matches!(&plan.effects[0], Effect::Results(status::FAILED, p, _) if p.contains("reach"))
    );
    assert!(
        session
            .call(
                ops::FETCHED,
                6,
                &[text(3), utf16("odd"), text(0), Vec::new()]
            )
            .is_err()
    );
}

#[test]
fn without_a_source_nothing_is_fetched() {
    let mut session = Session::new();
    let plan = call(&mut session, ops::SHOW, 0, &[flag::TRUE]);
    assert!(!plan.showing && plan.effects.is_empty());
    assert!(
        call(&mut session, ops::LOAD, 0, &[RECENTS])
            .effects
            .is_empty()
    );
    let plan = call(&mut session, ops::SNAPSHOT, 0, &[]);
    assert_eq!(plan.effects[0], loading());
    let Effect::Chips(chips) = &plan.effects[1] else {
        panic!()
    };
    assert_eq!(chips[0], TRENDING);
}

#[test]
fn recents_read_keep_what_was_sent_meanwhile_and_open_first() {
    let mut session = available();
    let plan = call(&mut session, ops::ATTACH, 0, &["/files/gif-recents.json"]);
    assert_eq!(
        plan.effects,
        vec![Effect::Read(1, "/files/gif-recents.json".into())]
    );
    assert!(
        call(&mut session, ops::ATTACH, 0, &["/files/gif-recents.json"])
            .effects
            .is_empty()
    );
    let sent: Vec<String> = [flag::FALSE.to_owned()]
        .into_iter()
        .chain(rows::gif(&gif("new")).iter().map(|it| string(it)))
        .collect();
    let sent: Vec<&str> = sent.iter().map(String::as_str).collect();
    let plan = call(&mut session, ops::SENT, 0, &sent);
    assert_eq!(plan.effects[1], Effect::Save(1));
    session.read(1, Some(vec![gif("old"), gif("new")]));
    session.read(0, Some(vec![gif("stale")]));
    let plan = call(&mut session, ops::MERGED, 0, &[]);
    let Effect::Chips(chips) = &plan.effects[0] else {
        panic!()
    };
    assert_eq!(chips[0], RECENTS);
    let (path, gifs) = session.rows(1).unwrap();
    assert_eq!(string(&path), "/files/gif-recents.json");
    assert_eq!(
        gifs.iter().map(|it| string(&it.id)).collect::<Vec<_>>(),
        ["new", "old"]
    );
    assert!(session.rows(0).is_none());
    let plan = call(&mut session, ops::SHOW, 0, &[flag::TRUE]);
    assert_eq!(plan.shown, RECENTS);
    assert_eq!(plan.effects, vec![loaded(&["new", "old"])]);
}

#[test]
fn incognito_sends_are_not_kept() {
    let mut session = available();
    let mut fields = vec![utf16(flag::TRUE)];
    fields.extend(rows::gif(&gif("a")));
    let answer = session.call(ops::SENT, 0, &fields).unwrap();
    assert_eq!(answer.len(), 4);
    assert!(session.rows(0).is_none());
}

#[test]
fn the_search_field_edits_and_submits() {
    let mut session = available();
    let plan = call(&mut session, ops::START_TYPING, 0, &[]);
    assert!(plan.typing && plan.query.is_empty());
    call(&mut session, ops::TYPE, 0, &["happy dance"]);
    call(&mut session, ops::DELETE_LAST, 0, &[]);
    assert_eq!(call(&mut session, ops::DELETE_WORD, 0, &[]).query, "happy ");
    call(&mut session, ops::TYPE_WORD, 0, &["cat"]);
    assert_eq!(
        call(&mut session, ops::SWIPE, 0, &[flag::FALSE, "2", flag::TRUE]).query,
        "happy c"
    );
    assert_eq!(
        call(&mut session, ops::SWIPE, 0, &[flag::TRUE, "1", flag::FALSE]).query,
        "happy "
    );
    assert_eq!(
        call(
            &mut session,
            ops::SWIPE,
            0,
            &[flag::FALSE, "0", flag::FALSE]
        )
        .query,
        "happy cat"
    );
    let plan = call(&mut session, ops::SUBMIT, 0, &[]);
    assert!(!plan.typing);
    assert_eq!(plan.shown, "happy cat");
    assert_eq!(
        call(&mut session, ops::START_TYPING, 0, &[]).query,
        "happy cat"
    );
    call(&mut session, ops::SWIPE, 0, &[flag::TRUE, "9", flag::TRUE]);
    assert_eq!(call(&mut session, ops::SUBMIT, 0, &[]).shown, TRENDING);
    let plan = call(&mut session, ops::RESET, 0, &[]);
    assert!(!plan.typing && !plan.showing);
    assert_eq!(plan.effects, vec![Effect::Forget]);
    assert!(session.call("nothing", 0, &[]).is_err());
}

#[test]
fn the_last_word_is_offered_unless_private() {
    let mut session = available();
    let plan = call(
        &mut session,
        ops::OFFER,
        0,
        &[flag::TRUE, "well done, congrats", flag::FALSE],
    );
    let Effect::Chips(chips) = &plan.effects[0] else {
        panic!()
    };
    assert_eq!(chips[0], "Congrats");
    let plan = call(
        &mut session,
        ops::OFFER,
        0,
        &[flag::TRUE, "pizza", flag::TRUE],
    );
    assert_eq!(
        plan.effects,
        call(
            &mut Session::new(),
            ops::OFFER,
            0,
            &[flag::FALSE, "", flag::FALSE]
        )
        .effects
    );
}
