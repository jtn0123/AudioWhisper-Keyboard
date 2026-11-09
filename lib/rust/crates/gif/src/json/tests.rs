use super::*;
use crate::utf16;

/// Reads `text` as an object of string fields, giving each key and value, or `Err`.
fn fields(text: &str) -> Result<Vec<(String, Option<String>)>> {
    let source = utf16(text);
    let mut lexer = Lexer::new(&source);
    let mut read = Vec::new();
    lexer.object(|lexer, key| {
        if key == utf16("skip") {
            return Ok(false);
        }
        let value = lexer.nullable_string()?;
        read.push((
            String::from_utf16_lossy(key),
            value.map(|it| String::from_utf16_lossy(&it)),
        ));
        Ok(true)
    })?;
    lexer.expect_eof()?;
    Ok(read)
}

fn pair(key: &str, value: &str) -> (String, Option<String>) {
    (key.to_owned(), Some(value.to_owned()))
}

#[test]
fn whitespace_is_only_space_tab_and_line_ends() {
    assert_eq!(
        fields(" \t\r\n{ \"a\" : \"b\" }\n"),
        Ok(vec![pair("a", "b")])
    );
    assert_eq!(fields("\u{0B}{}"), Err(Unreadable));
    assert_eq!(fields("{}\u{A0}"), Err(Unreadable));
}

#[test]
fn members_need_no_commas_but_may_not_lead_or_trail() {
    assert_eq!(
        fields(r#"{"a":"1" "b":"2"}"#),
        Ok(vec![pair("a", "1"), pair("b", "2")])
    );
    assert_eq!(
        fields(r#"{"a":"1","b":"2"}"#),
        Ok(vec![pair("a", "1"), pair("b", "2")])
    );
    assert_eq!(fields(r#"{,"a":"1"}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"a":"1",}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"a":"1",,"b":"2"}"#), Err(Unreadable));
    assert_eq!(fields("{,}"), Err(Unreadable));
}

#[test]
fn unknown_values_are_skipped_by_brackets_alone() {
    assert_eq!(fields(r#"{"skip":[1,{"x":]}],"a":"b"}"#), Err(Unreadable));
    assert_eq!(
        fields(r#"{"skip":[1 2 {:,} "]"],"a":"b"}"#),
        Ok(vec![pair("a", "b")])
    );
    assert_eq!(
        fields(r#"{"skip":tru-e "a":"b"}"#),
        Ok(vec![pair("a", "b")])
    );
    assert_eq!(fields(r#"{"skip":[}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"skip":["#), Err(Unreadable));
    assert_eq!(fields(r#"{"skip":,"a":"b"}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"skip":"x\q"}"#), Err(Unreadable));
}

#[test]
fn strings_take_escapes_and_raw_control_characters() {
    assert_eq!(
        fields(r#"{"a":"\b\t\n\f\r\/\"\\éÉ"}"#),
        Ok(vec![pair("a", "\u{8}\t\n\u{C}\r/\"\\éÉ")])
    );
    assert_eq!(
        fields("{\"a\":\"\u{1}\n\"}"),
        Ok(vec![pair("a", "\u{1}\n")])
    );
    assert_eq!(fields(r#"{"a":"\x"}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"a":"\u00g0"}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"a":"unclosed}"#), Err(Unreadable));
    // kotlinx wants a unit after the four digits, which the closing quote is.
    assert_eq!(fields(r#"{"a":"A"}"#), Ok(vec![pair("a", "A")]));
    assert_eq!(fields(r#"{"a":"A"#), Err(Unreadable));
    // A surrogate half is kept as it is.
    let source = utf16(r#"{"a":"\ud800"}"#);
    let mut lexer = Lexer::new(&source);
    lexer
        .object(|lexer, _| {
            assert_eq!(lexer.string()?, [0xD800]);
            Ok(true)
        })
        .unwrap();
}

#[test]
fn null_is_null_only_alone() {
    assert_eq!(fields(r#"{"a":null}"#), Ok(vec![("a".to_owned(), None)]));
    assert_eq!(
        fields(r#"{"a":null "b":"c"}"#),
        Ok(vec![("a".to_owned(), None), pair("b", "c")])
    );
    assert_eq!(fields(r#"{"a":nullx}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"a":nul}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"a":NULL}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"a":"null"}"#), Ok(vec![pair("a", "null")]));
}

#[test]
fn keys_must_be_strings_followed_by_colons() {
    assert_eq!(fields(r#"{a:"b"}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"a" "b"}"#), Err(Unreadable));
    assert_eq!(fields(r#"{"a":"b"} x"#), Err(Unreadable));
    assert_eq!(fields(r#"["a"]"#), Err(Unreadable));
    assert_eq!(fields(""), Err(Unreadable));
}

/// Reads `text` as an array of strings.
fn strings(text: &str) -> Result<usize> {
    let source = utf16(text);
    let mut lexer = Lexer::new(&source);
    let mut count = 0;
    lexer.list(|lexer| {
        lexer.string()?;
        count += 1;
        Ok(())
    })?;
    lexer.expect_eof()?;
    Ok(count)
}

#[test]
fn items_need_commas_between_them_only() {
    assert_eq!(strings("[]"), Ok(0));
    assert_eq!(strings(r#"[ "a" , "b" ]"#), Ok(2));
    assert_eq!(strings(r#"["a" "b"]"#), Err(Unreadable));
    assert_eq!(strings(r#"[,"a"]"#), Err(Unreadable));
    assert_eq!(strings(r#"["a",]"#), Err(Unreadable));
    assert_eq!(strings(r#"["a",,"b"]"#), Err(Unreadable));
    assert_eq!(strings(r#"["a""#), Err(Unreadable));
    assert_eq!(strings(r#"["a"}"#), Err(Unreadable));
}
