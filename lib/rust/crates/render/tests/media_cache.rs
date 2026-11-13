use render::media_cache::{Frame, MediaCache};
fn frame(delay: i64) -> Frame {
    Frame::argb(2, 2, &[0xffff0000u32 as i32; 4], delay).unwrap()
}
#[test]
fn only_visible_animations_have_deadlines_and_close_is_idle() {
    let mut c = MediaCache::default();
    assert!(c.upload(1, 0, frame(100_000_000), false));
    assert!(c.upload(1, 1, frame(200_000_000), true));
    c.visibility([1].into(), 1000);
    assert_eq!(c.deadline(), Some(100_001_000));
    assert!(!c.tick(100_000_999));
    assert!(c.tick(100_001_000));
    assert_eq!(c.entries[&1].frame, 1);
    assert_eq!(c.deadline(), Some(300_001_000));
    c.visibility(Default::default(), 200_000_000);
    assert_eq!(c.deadline(), None);
    assert!(!c.tick(i64::MAX));
    c = MediaCache::default();
    assert_eq!(c.bytes, 0);
    assert_eq!(c.deadline(), None);
}
#[test]
fn admission_counts_both_backends_and_evicts_oldest_invisible() {
    let mut c = MediaCache::default();
    c.budget = 96;
    c.per_item = 64;
    c.upload(1, 0, frame(1), true);
    c.upload(2, 0, frame(1), true);
    c.upload(3, 0, frame(1), true);
    c.visibility([2, 3].into(), 0);
    assert!(c.upload(4, 0, frame(1), true));
    assert!(!c.entries.contains_key(&1));
    assert_eq!(c.bytes, 96);
    assert!(!c.upload(2, 2, frame(1), true));
    assert!(c.upload(2, 1, frame(1), true));
    assert!(!c.upload(2, 2, frame(1), true));
    assert!(c.bytes <= c.budget);
}
#[test]
fn malformed_upload_and_frame_cap_are_rejected() {
    assert!(Frame::argb(2, 2, &[0; 3], 1).is_none());
    assert!(Frame::argb(0, 1, &[], 1).is_none());
    let mut c = MediaCache::default();
    assert!(!c.upload(1, 64, frame(1), true));
}

#[test]fn argb_is_premultiplied_once_for_both_backends(){let frame=Frame::argb(1,1,&[0x80ff0000u32 as i32],1).unwrap();assert_eq!(frame.cpu.data_as_u8_slice(),&[128,0,0,128]);}
