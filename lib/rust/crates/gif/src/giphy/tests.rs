use super::*;

fn parsed(body: &str) -> Result<Vec<Gif>> {
    parse(&utf16(body))
}

fn text(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

const ITEM: &str = r#"{"id":"abc","title":"Cat","url":"https://giphy.com/gifs/abc",
    "images":{"fixed_width":{"url":"https://p/abc.gif","webp":"https://p/abc.webp",
    "width":"200","height":"100"},"downsized":{"url":"https://d/abc.gif"}}}"#;

#[test]
fn a_gif_reads_its_preview_and_what_to_send() {
    let gifs = parsed(&format!(r#"{{"data":[{ITEM}],"meta":{{"status":200}}}}"#)).unwrap();
    assert_eq!(gifs.len(), 1);
    let gif = &gifs[0];
    assert_eq!(text(&gif.id), "abc");
    assert_eq!(text(&gif.title), "Cat");
    assert_eq!(text(&gif.preview_url), "https://p/abc.webp");
    assert_eq!(gif.aspect_ratio, 2.0);
    assert_eq!(text(&gif.send_url), "https://d/abc.gif");
    assert_eq!(text(&gif.page_url), "https://giphy.com/gifs/abc");
}

#[test]
fn missing_renditions_fall_back_or_drop_the_gif() {
    let gif = |images: &str| {
        let body = format!(r#"{{"data":[{{"id":"a","images":{images}}}]}}"#);
        parsed(&body).unwrap().pop()
    };
    let only = gif(r#"{"fixed_width":{"url":"u","width":"3","height":"2"}}"#).unwrap();
    assert_eq!(
        (text(&only.preview_url), text(&only.send_url)),
        ("u".into(), "u".into())
    );
    assert_eq!(only.aspect_ratio, 1.5);
    assert!(text(&only.title).is_empty() && text(&only.page_url).is_empty());
    let webp =
        gif(r#"{"fixed_width":{"webp":"w","width":"1","height":"1"},"downsized":{"url":"d"}}"#);
    assert_eq!(text(&webp.unwrap().preview_url), "w");
    assert!(gif(r#"{"fixed_width":{"webp":"w","width":"1","height":"1"}}"#).is_none());
    assert!(gif(r#"{"fixed_width":{"width":"1","height":"1"},"downsized":{"url":"d"}}"#).is_none());
    assert!(gif(r#"{"fixed_width":null,"downsized":{"url":"d"}}"#).is_none());
    assert!(gif(r#"{"fixed_width":{"url":"u","width":"x","height":"1"}}"#).is_none());
    assert!(gif(r#"{"fixed_width":{"url":"u","width":"1","height":"0"}}"#).is_none());
    assert!(gif(r#"{"fixed_width":{"url":"u","width":"1","height":"-1"}}"#).is_none());
    assert!(gif(r#"{"fixed_width":{"url":"u","width":"1","height":"NaN"}}"#).is_none());
    assert!(gif(r#"{"fixed_width":{"url":"u","width":"1"}}"#).is_none());
    assert!(gif("{}").is_none());
    let downsized_null =
        gif(r#"{"fixed_width":{"url":"u","width":"1","height":"1"},"downsized":null}"#);
    assert_eq!(text(&downsized_null.unwrap().send_url), "u");
}

#[test]
fn blank_and_repeated_ids_are_skipped() {
    let body = r#"{"data":[
        {"id":" ","images":{"fixed_width":{"url":"u","width":"1","height":"1"}}},
        {"id":"a","title":"first","images":{"fixed_width":{"url":"u","width":"1","height":"1"}}},
        {"id":"a","title":"second","images":{"fixed_width":{"url":"u","width":"1","height":"1"}}},
        {"id":"b","images":{}},
        {"id":"b","title":"kept","images":{"fixed_width":{"url":"u","width":"1","height":"1"}}}
    ]}"#;
    let titles: Vec<String> = parsed(body)
        .unwrap()
        .iter()
        .map(|it| text(&it.title))
        .collect();
    assert_eq!(titles, ["first", "kept"]);
}

#[test]
fn an_answer_without_data_has_no_gifs() {
    assert_eq!(parsed("{}"), Ok(vec![]));
    assert_eq!(parsed(r#"{"data":[]}"#), Ok(vec![]));
    assert_eq!(parsed(r#"{"data":[{"id":"x"}],"data":[]}"#), Ok(vec![]));
}

#[test]
fn a_body_that_isnt_the_json_expected_is_unreadable() {
    for body in [
        "",
        "not json",
        "[]",
        r#"{"data":null}"#,
        r#"{"data":{}}"#,
        r#"{"data":[1]}"#,
        r#"{"data":[{"id":null}]}"#,
        r#"{"data":[{"id":5}]}"#,
        r#"{"data":[{"images":[]}]}"#,
        r#"{"data":[{"images":{"fixed_width":{"width":200}}}]}"#,
        r#"{"data":[]"#,
        r#"{"data":[]} {}"#,
    ] {
        assert_eq!(parsed(body), Err(crate::json::Unreadable), "{body}");
    }
}

#[test]
fn values_are_encoded_as_url_encoder_does() {
    let encoded = |value: &[u16]| text(&encode(value));
    assert_eq!(encoded(&utf16("az AZ09.-*_")), "az+AZ09.-*_");
    assert_eq!(
        encoded(&utf16("a&b=c/d?~'!()")),
        "a%26b%3Dc%2Fd%3F%7E%27%21%28%29"
    );
    assert_eq!(encoded(&utf16("café 😀")), "caf%C3%A9+%F0%9F%98%80");
    assert_eq!(encoded(&[0xD800, 0x61, 0xDC00]), "%3Fa%3F");
    assert_eq!(encoded(&[0x00, 0x7F]), "%00%7F");
}

#[test]
fn urls_add_the_key_and_settings_after_the_params() {
    let key = utf16("k y");
    assert_eq!(
        text(&trending_url(&key)),
        "https://api.giphy.com/v1/gifs/trending?api_key=k+y&limit=30&rating=pg-13&lang=en"
    );
    assert_eq!(
        text(&search_url(&key, &utf16("happy cat"))),
        "https://api.giphy.com/v1/gifs/search?q=happy+cat&api_key=k+y&limit=30&rating=pg-13&lang=en"
    );
    let params = [
        (utf16("lang"), utf16("fr")),
        (utf16("x"), utf16("1")),
        (utf16("x"), utf16("2")),
    ];
    assert_eq!(
        text(&url(&key, &utf16("e"), &params)),
        "https://api.giphy.com/v1/gifs/e?lang=en&x=1&x=2&api_key=k+y&limit=30&rating=pg-13"
    );
}

#[test]
fn a_search_is_cut_to_fifty_units() {
    let long = "x".repeat(49) + "😀";
    let url = text(&search_url(&utf16(""), &utf16(&long)));
    assert!(url.contains(&format!("q={}%3F&", "x".repeat(49))), "{url}");
}
