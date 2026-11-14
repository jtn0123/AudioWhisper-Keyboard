use super::*;
fn fields(items: &[&str]) -> Vec<Vec<u16>> {
    items.iter().map(|s| s.encode_utf16().collect()).collect()
}
fn call(s: &mut Session, op: &str, id: i32, args: &[&str]) -> Vec<Vec<u16>> {
    s.call(op, 1000, id, &fields(args)).unwrap()
}
#[test]
fn detailed_phases_and_cancel_generation() {
    let mut s = Session::new();
    call(&mut s, "ready", 0, &["true", "true", "ready"]);
    let id = s.generation;
    call(&mut s, "opened", id, &[]);
    for (op, phase) in [
        ("warming", "warming"),
        ("warmed", "listening"),
        ("vadOn", "speech"),
        ("vadOff", "trailing_silence"),
        ("stop", "flushing"),
        ("finalizing", "finalizing"),
        ("finished", "committed"),
    ] {
        call(&mut s, op, id, &[]);
        assert_eq!(s.phase, phase);
    }
    call(&mut s, "ready", 0, &["true", "true", "ready"]);
    let id = s.generation;
    call(&mut s, "opened", id, &[]);
    call(&mut s, "cancel", id, &[]);
    assert_eq!(s.phase, "cancelled");
    assert_eq!(call(&mut s, "final", id, &["true", "fixture"]).len(), 6);
    assert_eq!(call(&mut s, "preview", id, &["fixture"]).len(), 6);
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
    call(&mut s, "release", 0, &[]);
    for op in [
        "press",
        "releaseMic",
        "ready",
        "opened",
        "final",
        "preview",
        "modelError",
        "free",
        "unload",
    ] {
        assert!(call(&mut s, op, 0, &[]).is_empty());
    }
}

#[test]
fn cancel_and_stop_at_every_phase() {
    for target in [
        "idle",
        "warming",
        "listening",
        "speech",
        "trailing_silence",
        "flushing",
        "finalizing",
        "committed",
        "cancelled",
        "error",
    ] {
        for operation in ["cancel", "stop"] {
            let mut s = Session::new();
            if target != "idle" {
                call(&mut s, "ready", 0, &["true", "true", "ready"]);
                let generation = s.generation;
                call(&mut s, "opened", generation, &[]);
                let id = s.generation;
                match target {
                    "warming" => {
                        call(&mut s, "warming", id, &[]);
                    }
                    "speech" => {
                        call(&mut s, "vadOn", id, &[]);
                    }
                    "trailing_silence" => {
                        call(&mut s, "vadOn", id, &[]);
                        call(&mut s, "vadOff", id, &[]);
                    }
                    "flushing" => {
                        call(&mut s, "stop", id, &[]);
                    }
                    "finalizing" => {
                        call(&mut s, "stop", id, &[]);
                        call(&mut s, "finalizing", id, &[]);
                    }
                    "committed" => {
                        call(&mut s, "stop", id, &[]);
                        call(&mut s, "finished", id, &[]);
                    }
                    "cancelled" => {
                        call(&mut s, "cancel", id, &[]);
                    }
                    "error" => {
                        call(&mut s, "modelError", id, &[]);
                    }
                    _ => {}
                }
            }
            assert_eq!(s.phase, target);
            let id = s.generation;
            call(&mut s, operation, id, &[]);
            if operation == "cancel" {
                assert_eq!(s.status, 0);
            } else {
                assert!(s.status == 0 || s.status == 2);
            }
        }
    }
}
