"""MIME filters (the lib family's MimePolicyMatchTest, in :lib:kotlin, whose JVM tests run as
`test`). These cases are the ones the frozen Kotlin reference used to catch."""
MIME = 'lib/rust/crates/utilities/src/mime.rs'
MODULE = ':lib:kotlin'
TASK = 'test'
TARGETS = {'mime': ['*MimePolicyMatchTest']}
CASES = [
    ('star-matches-spaces', MIME, r'pattern.extend(text(r"[^\s]+"));', r'pattern.extend(text(r".+"));', 'mime'),
    ('three-part-filter-accepted', MIME, 'let reason = if pieces.len() != 2 {', 'let reason = if pieces.len() > 3 || pieces.len() < 2 {',
     'mime'),
    ('empty-subtype-accepted', MIME, '} else if pieces.iter().any(|s| s.is_empty()) {', '} else if pieces[0].is_empty() {', 'mime'),
    ('three-part-value-admitted', MIME, '(!a.is_empty() && !b.is_empty() && parts.next().is_none())',
     '(!a.is_empty() && !b.is_empty())', 'mime'),
    ('one-means-any', MIME, 'ops::ONE => matched == 1,', 'ops::ONE => matched > 0,', 'mime'),
    ('all-of-none', MIME, 'ops::ALL => count > 0 && matched == count,', 'ops::ALL => matched == count,', 'mime'),
]
