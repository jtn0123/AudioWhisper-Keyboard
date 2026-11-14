use super::*;
use std::sync::{Arc, atomic::AtomicUsize};
#[derive(Debug)]
struct Owner(Arc<AtomicUsize>);
impl Drop for Owner {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}
#[test]
fn stop_precedes_free_and_callback_keeps_buffer_live() {
    let freed = Arc::new(AtomicUsize::new(0));
    let g = Generator::new(Owner(freed.clone()));
    let synthesis = g.lease().unwrap();
    assert!(synthesis.current());
    g.close();
    assert!(!synthesis.current());
    assert!(g.stopped());
    assert_eq!(
        freed.load(Ordering::Acquire),
        0,
        "active synthesis/playback still borrows buffer"
    );
    assert!(g.lease().is_err());
    drop(synthesis);
    assert_eq!(freed.load(Ordering::Acquire), 1);
}
#[test]
fn cancellation_then_immediate_respeak_uses_a_new_generation() {
    let freed = Arc::new(AtomicUsize::new(0));
    let g = Generator::new(Owner(freed.clone()));
    for _ in 0..200 {
        let synthesis = g.lease().unwrap();
        g.cancel();
        assert!(!synthesis.current());
        drop(synthesis);
        let next = g.lease().unwrap();
        assert!(next.current());
        drop(next);
        assert_eq!(freed.load(Ordering::Acquire), 0);
    }
    g.close();
    assert_eq!(freed.load(Ordering::Acquire), 1);
}
