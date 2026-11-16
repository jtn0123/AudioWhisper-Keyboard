//! TagSoup 1.2's `HTMLScanner.scan` loop: reads UTF-16 units, folds CR LF and C1 controls,
//! drops other controls, and runs each transition's action against the Parser.
use super::{
    scanner_table::{A, S, transition},
    tagsoup::{Element, Handler, Parser, is_digit, is_letter, schema},
};
use std::collections::HashMap;
/// HTMLScanner's map of C1 controls to the Windows-1252 characters they usually mean.
const WIN: [u16; 32] = [
    0x20AC, 0xFFFD, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160, 0x2039,
    0x0152, 0xFFFD, 0x017D, 0xFFFD, 0xFFFD, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014,
    0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0xFFFD, 0x017E, 0x0178,
];

/// Runs `HTMLScanner.scan` over `input` with TagSoup's Parser as the scan handler.
pub(super) fn parse<H: Handler>(input: &[u16], h: &mut H) {
    let root = schema().types["<root>"].clone();
    let mut p = Parser {
        h,
        bogons: HashMap::new(),
        stack: vec![Element {
            t: root,
            preclosed: false,
        }],
        saved: vec![],
        new_element: None,
        virgin: true,
        cdata: false,
        entity: 0,
    };
    let mut state = S::Pcdata;
    let mut buf: Vec<u16> = Vec::with_capacity(200);
    let mut capacity = 200usize;
    let mut at = 0usize;
    let mut pushed: Vec<i32> = vec![];
    if input.first() == Some(&0xFEFF) {
        at = 1;
    }
    let read = |at: &mut usize, pushed: &mut Vec<i32>| -> i32 {
        if let Some(c) = pushed.pop() {
            return c;
        }
        match input.get(*at) {
            Some(&c) => {
                *at += 1;
                i32::from(c)
            }
            None => -1,
        }
    };
    while state != S::Done {
        let mut ch = read(&mut at, &mut pushed);
        if (0x80..=0x9F).contains(&ch) {
            ch = i32::from(WIN[(ch - 0x80) as usize]);
        }
        if ch == i32::from(b'\r') {
            ch = read(&mut at, &mut pushed);
            if ch != i32::from(b'\n') {
                pushed.push(ch);
                ch = i32::from(b'\n');
            }
        }
        if !(ch >= 0x20 || ch == 0x0A || ch == 0x09 || ch == -1) {
            continue;
        }
        let Some((action, next)) = transition(state, ch) else {
            // HTMLScanner throws "can't cope"; Html.fromHtml would surface that as an error.
            break;
        };
        let mut next = next;
        let save = |buf: &mut Vec<u16>, capacity: &mut usize, p: &mut Parser<H>, c: i32| {
            if buf.len() >= *capacity - 20 {
                if state == S::Pcdata || state == S::Cdata {
                    p.pcdata(buf);
                    buf.clear();
                } else {
                    *capacity *= 2;
                }
            }
            buf.push(c as u16);
        };
        match action {
            A::AdupSave => {
                buf.clear();
                save(&mut buf, &mut capacity, &mut p, ch);
            }
            A::Aname | A::AnameAdup | A::Aval => buf.clear(),
            A::AdupStagc | A::AnameAdupStagc | A::AvalStagc | A::Stagc => {
                buf.clear();
                p.stagc();
            }
            A::Cdata => {
                let n = buf.len().saturating_sub(if buf.len() > 1 { 2 } else { 0 });
                p.pcdata(&buf[..n]);
                buf.clear();
            }
            A::EntityStart => {
                p.pcdata(&buf);
                buf.clear();
                save(&mut buf, &mut capacity, &mut p, ch);
            }
            A::Entity => {
                let unit = ch as u16;
                let alnum = is_letter(unit) || is_digit(unit);
                let hex = is_digit(unit) || (unit < 0x80 && (unit as u8).is_ascii_hexdigit());
                if state == S::Ent && ch == i32::from(b'#') {
                    next = S::Ncr;
                    save(&mut buf, &mut capacity, &mut p, ch);
                } else if state == S::Ncr && (ch == i32::from(b'x') || ch == i32::from(b'X')) {
                    next = S::Xncr;
                    save(&mut buf, &mut capacity, &mut p, ch);
                } else if (state == S::Ent && ch >= 0 && alnum)
                    || (state == S::Ncr && ch >= 0 && is_digit(ch as u16))
                    || (state == S::Xncr && ch >= 0 && hex)
                {
                    save(&mut buf, &mut capacity, &mut p, ch);
                } else {
                    p.entity(&buf[1..]);
                    let mut ent = p.entity;
                    if ent != 0 {
                        buf.clear();
                        if (0x80..=0x9F).contains(&ent) {
                            ent = u32::from(WIN[(ent - 0x80) as usize]);
                        }
                        if ent < 0x20 || (0xD800..=0xDFFF).contains(&ent) {
                            // Controls become a space that is never saved; surrogates drop.
                        } else if ent <= 0xFFFF {
                            save(&mut buf, &mut capacity, &mut p, ent as i32);
                        } else {
                            let e = ent.wrapping_sub(0x10000);
                            save(&mut buf, &mut capacity, &mut p, ((e >> 10) + 0xD800) as i32);
                            save(
                                &mut buf,
                                &mut capacity,
                                &mut p,
                                ((e & 0x3FF) + 0xDC00) as i32,
                            );
                        }
                        if ch != i32::from(b';') {
                            pushed.push(ch);
                        }
                    } else {
                        pushed.push(ch);
                    }
                    next = S::Pcdata;
                }
            }
            A::Etag => {
                let name = std::mem::take(&mut buf);
                p.etag(&name);
            }
            A::Decl | A::Cmnt | A::Pi | A::Pitarget | A::PitargetPi => buf.clear(),
            A::Gi => {
                p.gi(&buf);
                buf.clear();
            }
            A::GiStagc => {
                p.gi(&buf);
                buf.clear();
                p.stagc();
            }
            A::Lt => {
                save(&mut buf, &mut capacity, &mut p, i32::from(b'<'));
                save(&mut buf, &mut capacity, &mut p, ch);
            }
            A::LtPcdata => {
                save(&mut buf, &mut capacity, &mut p, i32::from(b'<'));
                p.pcdata(&buf);
                buf.clear();
            }
            A::Pcdata => {
                p.pcdata(&buf);
                buf.clear();
            }
            A::Minus3 => {
                save(&mut buf, &mut capacity, &mut p, i32::from(b'-'));
                save(&mut buf, &mut capacity, &mut p, i32::from(b' '));
            }
            A::Minus2 | A::Minus => {
                if action == A::Minus2 {
                    save(&mut buf, &mut capacity, &mut p, i32::from(b'-'));
                    save(&mut buf, &mut capacity, &mut p, i32::from(b' '));
                }
                save(&mut buf, &mut capacity, &mut p, i32::from(b'-'));
                save(&mut buf, &mut capacity, &mut p, ch);
            }
            A::Save => save(&mut buf, &mut capacity, &mut p, ch),
            A::Skip => {}
            A::Sp => save(&mut buf, &mut capacity, &mut p, i32::from(b' ')),
            A::Emptytag => {
                if !buf.is_empty() {
                    p.gi(&buf);
                }
                buf.clear();
                if p.new_element.is_some() {
                    let e = p.new_element.clone().expect("checked");
                    p.rectify(e.t);
                    p.etag_basic(&[]);
                }
            }
            A::UnsavePcdata => {
                buf.pop();
                p.pcdata(&buf);
                buf.clear();
            }
        }
        if std::mem::take(&mut p.cdata) {
            next = S::Cdata;
        }
        state = next;
    }
    p.eof();
}
