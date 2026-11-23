"""ThirdPartyLicensesScreen (SettingsLicencesMatchTest): the cases the frozen AboutLibraries and
Html.fromHtml reference used to catch, over the licence data, the row text and the dialog text."""
LICENCES = 'lib/rust/crates/render/src/settings_ui/licences/'
DATA = LICENCES + 'data.rs'
LIST = LICENCES + 'list.rs'
HTML = LICENCES + 'html.rs'
MODULE = ':app'
TARGETS = {'licences': ['*SettingsLicencesMatchTest']}
CASES = [
    # The libraries: licence set order, the author line, the fallback name and the sort.
    ('licences-in-json-order', DATA, '    order.sort();\n    order.into_iter().map(|(_, _, i)| i).collect()',
     '    order.into_iter().map(|(_, _, i)| i).collect()', 'licences'),
    ('bucket-without-spread', DATA, 'let bucket = ((h ^ (h >> 16)) as usize) & (table - 1);',
     'let bucket = (h as usize) & (table - 1);', 'licences'),
    ('duplicate-licences-kept', DATA, 'if seen.contains(&hash.as_str()) {', 'if seen.is_empty() && seen.contains(&hash.as_str()) {',
     'licences'),
    ('organisation-before-developers', DATA, 'let author = if developers.is_empty() {',
     'let author = if organisation.is_some() {', 'licences'),
    ('developers-joined-by-semicolons', DATA, '                .join(", ")\n', '                .join("; ")\n', 'licences'),
    ('nameless-shows-blank', DATA, 'name: opt_string_or(o, "name", &id),', 'name: opt_string_or(o, "name", ""),', 'licences'),
    ('order-by-case', DATA, 'libraries.iter().map(|l| l.name.to_lowercase()).collect();',
     'libraries.iter().map(|l| l.name.clone()).collect();', 'licences'),
    ('ties-reversed', DATA, 'order.sort_by(|&a, &b| utf16_cmp(&keys[a], &keys[b]));',
     'order.sort_by(|&a, &b| utf16_cmp(&keys[a], &keys[b]).then(b.cmp(&a)));', 'licences'),
    ('order-by-code-point', DATA, 'a.encode_utf16().cmp(b.encode_utf16())', 'a.cmp(b)', 'licences'),
    # The row's line: a blank version or author is left out.
    ('blank-version-shown', LIST, 'library.version.as_deref().filter(|v| !blank(v))', 'library.version.as_deref()', 'licences'),
    ('empty-author-shown', LIST, 'Some(library.author.as_str()).filter(|a| !blank(a))', 'Some(library.author.as_str())',
     'licences'),
    # The dialog: the licence contents joined, then Html.fromHtml's text.
    ('contents-two-breaks-apart', DATA, '.join("<br /><br /><br /><br />")', '.join("<br /><br />")', 'licences'),
    ('content-newline-a-space', DATA, ".replace('\\n', \"<br />\")", ".replace('\\n', \" \")", 'licences'),
    ('br-a-space', HTML, 'if local.eq_ignore_ascii_case("br") {\n            self.out.push(NL);',
     'if local.eq_ignore_ascii_case("br") {\n            self.out.push(SP);', 'licences'),
    ('img-a-question-mark', HTML, 'self.out.push(0xFFFC);', 'self.out.push(0x3F);', 'licences'),
    ('blocks-two-lines-apart', HTML, '            self.newlines(1);\n            self.marks += 1;',
     '            self.newlines(2);\n            self.marks += 1;', 'licences'),
    ('div-not-a-block', HTML, '"p" | "ul" | "li" | "div" | "blockquote"', '"p" | "ul" | "li" | "blockquote"', 'licences'),
    ('spaces-after-a-space-kept', HTML, 'if pred != SP && pred != NL {', 'if pred != NL {', 'licences'),
    ('break-before-text-ignored', HTML, 'self.out.last().copied().unwrap_or(NL)', 'self.out.first().copied().unwrap_or(NL)',
     'licences'),
    ('missing-content-as-null', DATA, '.unwrap_or("")', '.unwrap_or("null")', 'licences'),
]
