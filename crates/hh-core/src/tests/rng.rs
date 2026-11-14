use crate::rng::Stream;

fn draws(s: &mut Stream, n: usize) -> Vec<u32> {
    (0..n).map(|_| s.below(1000)).collect()
}

#[test]
fn a_purpose_is_its_own_stream() {
    let a = draws(&mut Stream::new(7, "sunlight"), 20);
    assert_eq!(a, draws(&mut Stream::new(7, "sunlight"), 20));
    assert_ne!(a, draws(&mut Stream::new(7, "events"), 20));
    assert_ne!(a, draws(&mut Stream::new(8, "sunlight"), 20));
}

#[test]
fn a_serialized_stream_continues_identically() {
    let mut s = Stream::new(u64::MAX, "sunlight");
    draws(&mut s, 13);
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("\"word_pos\":\""), "positions are strings: {json}");
    let mut t: Stream = serde_json::from_str(&json).unwrap();
    assert_eq!(draws(&mut s, 50), draws(&mut t, 50));
}

#[test]
fn below_stays_in_range() {
    let mut s = Stream::new(1, "x");
    assert!((0..1000).all(|_| s.below(3) < 3));
    assert!((0..100).all(|_| s.below(1) == 0));
}
