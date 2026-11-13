use super::*;

#[test]
fn missing_and_empty_layouts_differ() {
    assert_eq!(merge(&[2], &[-1], &[-1], &[]), Ok(vec![2, 0, 1]));
    assert_eq!(merge(&[2], &[], &[-1], &[]), Ok(vec![0]));
    assert_eq!(
        merge(&[], &[1, 2], &[1], &[0, 8, 9]),
        Ok(vec![1, 3, 2, 1, 2])
    );
}

#[test]
fn every_placeholder_inserts_the_last_main_row() {
    assert_eq!(
        merge(&[1, 2], &[3, 1], &[1], &[0, 4, 0, 5]),
        Ok(vec![1, 7, 1, 0, 5, 1, 2, 4, 1, 2, 1, 6])
    );
}

#[test]
fn invalid_metadata_is_an_error() {
    assert!(merge(&[-2], &[], &[], &[]).is_err());
    assert!(merge(&[], &[1], &[], &[]).is_err());
    assert!(hints(0, true, &[], &[1], &[]).is_err());
    assert!(hints(0, true, &[-1], &[], &[]).is_err());
}

#[test]
fn cache_hashes_keep_all_signed_bits() {
    let hashes = [i32::MIN, -1, 0, 1, i32::MAX];
    let keys: Vec<_> = (-1..=10)
        .flat_map(|mode| hashes.map(|hash| cache_key(mode, hash)))
        .collect();
    assert_eq!(invalidated(&keys, None, Some(-1)).len(), 12);
    assert_eq!(invalidated(&keys, Some(-1), None).len(), 5);
    assert_eq!(
        invalidated(&keys, Some(-1), Some(i32::MIN)),
        vec![cache_key(-1, i32::MIN)]
    );
}
