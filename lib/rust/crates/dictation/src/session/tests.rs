use super::*;
use wire::dictation::{self as events, effect, model, model_state, phase, problem, ready, status};

const READY: [&str; 3] = [flag::TRUE, flag::TRUE, ready::READY];
fn fields(items: &[&str]) -> Vec<Vec<u16>> {
    items.iter().map(|s| s.encode_utf16().collect()).collect()
}
fn call(s: &mut Session, op: &str, id: i32, args: &[&str]) -> Vec<Vec<u16>> {
    s.call(op, 1000, id, &fields(args)).unwrap()
}
#[test]
fn detailed_phases_and_cancel_generation() {
    let mut s = Session::new();
    call(&mut s, events::READY, 0, &READY);
    let id = s.generation;
    call(&mut s, events::OPENED, id, &[]);
    for (op, phase) in [
        (events::WARMING, phase::WARMING),
        (events::WARMED, phase::LISTENING),
        (events::VAD_ON, phase::SPEECH),
        (events::VAD_OFF, phase::TRAILING_SILENCE),
        (events::STOP, phase::FLUSHING),
        (events::FINALIZING, phase::FINALIZING),
        (events::FINISHED, phase::COMMITTED),
    ] {
        call(&mut s, op, id, &[]);
        assert_eq!(s.phase, phase);
    }
    call(&mut s, events::READY, 0, &READY);
    let id = s.generation;
    call(&mut s, events::OPENED, id, &[]);
    call(&mut s, events::CANCEL, id, &[]);
    assert_eq!(s.phase, phase::CANCELLED);
    assert_eq!(
        call(&mut s, events::FINAL, id, &[flag::TRUE, "fixture"]).len(),
        plan::SIZE
    );
    assert_eq!(
        call(&mut s, events::PREVIEW, id, &["fixture"]).len(),
        plan::SIZE
    );
    assert!(!s.typed);
}
#[test]
fn exact_timeout_edges_and_preview_gate() {
    assert_eq!(release_at(1000, 2000, true), 301000);
    assert_eq!(release_at(1000, 2000, false), 62000);
    assert_eq!(release_at(1000, 300999, false), 301000);
    let mut s = Session::new();
    assert!(s.claim_preview());
    assert!(!s.claim_preview());
    s.complete_preview();
    assert!(s.claim_preview());
}
#[test]
fn released_owner_ignores_all_late_events() {
    let mut s = Session::new();
    call(&mut s, events::RELEASE, 0, &[]);
    for op in [
        events::PRESS,
        events::RELEASE_MIC,
        events::READY,
        events::OPENED,
        events::FINAL,
        events::PREVIEW,
        events::MODEL_ERROR,
        events::FREE,
        events::UNLOAD,
    ] {
        assert!(call(&mut s, op, 0, &[]).is_empty());
    }
}

#[test]
fn cancel_and_stop_at_every_phase() {
    for target in [
        phase::IDLE,
        phase::WARMING,
        phase::LISTENING,
        phase::SPEECH,
        phase::TRAILING_SILENCE,
        phase::FLUSHING,
        phase::FINALIZING,
        phase::COMMITTED,
        phase::CANCELLED,
        phase::ERROR,
    ] {
        for operation in [events::CANCEL, events::STOP] {
            let mut s = Session::new();
            if target != phase::IDLE {
                call(&mut s, events::READY, 0, &READY);
                let generation = s.generation;
                call(&mut s, events::OPENED, generation, &[]);
                let id = s.generation;
                match target {
                    phase::WARMING => {
                        call(&mut s, events::WARMING, id, &[]);
                    }
                    phase::SPEECH => {
                        call(&mut s, events::VAD_ON, id, &[]);
                    }
                    phase::TRAILING_SILENCE => {
                        call(&mut s, events::VAD_ON, id, &[]);
                        call(&mut s, events::VAD_OFF, id, &[]);
                    }
                    phase::FLUSHING => {
                        call(&mut s, events::STOP, id, &[]);
                    }
                    phase::FINALIZING => {
                        call(&mut s, events::STOP, id, &[]);
                        call(&mut s, events::FINALIZING, id, &[]);
                    }
                    phase::COMMITTED => {
                        call(&mut s, events::STOP, id, &[]);
                        call(&mut s, events::FINISHED, id, &[]);
                    }
                    phase::CANCELLED => {
                        call(&mut s, events::CANCEL, id, &[]);
                    }
                    phase::ERROR => {
                        call(&mut s, events::MODEL_ERROR, id, &[]);
                    }
                    _ => {}
                }
            }
            assert_eq!(s.phase, target);
            let id = s.generation;
            call(&mut s, operation, id, &[]);
            if operation == events::CANCEL {
                assert_eq!(s.status, status::IDLE);
            } else {
                assert!(s.status == status::IDLE || s.status == status::FINISHING);
            }
        }
    }
}

#[test]
fn a_ready_row_names_why_dictation_cannot_start() {
    for (row, problem, settings) in [
        (
            [flag::FALSE, flag::TRUE, ready::READY],
            problem::FIELD_OFF,
            None,
        ),
        (
            [flag::TRUE, "true", ready::READY],
            problem::ALLOW_MIC,
            Some(effect::APP_SETTINGS),
        ),
        (
            [flag::TRUE, flag::TRUE, ready::MISSING],
            problem::NOT_INSTALLED,
            Some(effect::MODEL_SETTINGS),
        ),
        (
            [flag::TRUE, flag::TRUE, ready::DAMAGED],
            problem::DAMAGED,
            Some(effect::MODEL_SETTINGS),
        ),
    ] {
        let mut s = Session::new();
        let plan = call(&mut s, events::READY, 0, &row);
        let mut expected = fields(&[effect::PROBLEM, problem]);
        expected.extend(settings.map(|it| it.encode_utf16().collect()));
        assert_eq!(plan[plan::SIZE..], expected[..]);
        assert_eq!(s.phase, phase::ERROR);
    }
    let mut s = Session::new();
    let plan = call(&mut s, events::READY, 0, &READY);
    assert_eq!(plan[plan::SIZE..], fields(&[effect::LISTEN, "1"])[..]);
    assert!(s.call(events::READY, 0, 0, &fields(&[flag::TRUE])).is_err());
    assert!(s.call("readied", 0, 0, &[]).is_err());
}

#[test]
fn the_model_moves_through_its_states() {
    let mut s = Session::new();
    assert_eq!(s.model(model::GET), Ok(model_state::NONE));
    assert_eq!(s.model(model::FAILED), Ok(0));
    assert_eq!(s.model(model::GET), Ok(model_state::FAILED));
    assert_eq!(s.model(model::RETRY), Ok(0));
    assert_eq!(s.model(model::GET), Ok(model_state::NONE));
    assert_eq!(s.model(model::LOADED), Ok(0));
    assert_eq!(s.model(model::IS_LOADED), Ok(1));
    assert_eq!(s.model(model::CLOSE), Ok(1));
    assert_eq!(s.model(model::CLOSE), Ok(0));
    assert!(s.model("open").is_err());
}
