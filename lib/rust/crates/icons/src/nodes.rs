//! A path's nodes, Compose's `PathNode`s, read from the canonical path syntax `data/sources.tsv`
//! holds: a command letter, then that command's numbers, every token apart by a space.

/// One node: its SVG command letter, as `PathNode` spells it, and its numbers. An arc's flags are
/// 0 or 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Node {
    pub command: u8,
    pub args: [f32; 7],
}

/// How many numbers `command` takes, or `None` for a letter that is not a path command.
fn arity(command: u8) -> Option<usize> {
    Some(match command.to_ascii_lowercase() {
        b'z' => 0,
        b'h' | b'v' => 1,
        b'm' | b'l' | b't' => 2,
        b's' | b'q' => 4,
        b'c' => 6,
        b'a' => 7,
        _ => return None,
    })
}

/// The nodes of `svg`, or `None` if it is not canonical path syntax. A command followed by more
/// numbers than it takes repeats, and a move's extra pairs are lines, as in SVG.
pub(crate) fn parse(svg: &str) -> Option<Vec<Node>> {
    let mut nodes = vec![];
    let mut tokens = svg.split_ascii_whitespace().peekable();
    while let Some(token) = tokens.next() {
        let &[mut command] = token.as_bytes() else {
            return None;
        };
        let count = arity(command)?;
        let mut first = true;
        loop {
            let mut args = [0f32; 7];
            for arg in args.iter_mut().take(count) {
                *arg = tokens.next()?.parse().ok()?;
            }
            nodes.push(Node { command, args });
            let more = tokens.peek().is_some_and(|t| t.parse::<f32>().is_ok());
            if count == 0 || !more {
                break;
            }
            if first && command.eq_ignore_ascii_case(&b'm') {
                command = if command == b'M' { b'L' } else { b'l' };
            }
            first = false;
        }
    }
    Some(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(command: u8, args: &[f32]) -> Node {
        let mut all = [0f32; 7];
        all[..args.len()].copy_from_slice(args);
        Node { command, args: all }
    }

    #[test]
    fn reads_every_command_and_repeats() {
        let nodes = parse("M 1.0 2.0 3.0 4.0 h 5.0 a 1.0 2.0 0.0 1 0 3.0 4.0 Z").unwrap();
        assert_eq!(
            nodes,
            [
                node(b'M', &[1.0, 2.0]),
                node(b'L', &[3.0, 4.0]),
                node(b'h', &[5.0]),
                node(b'a', &[1.0, 2.0, 0.0, 1.0, 0.0, 3.0, 4.0]),
                node(b'Z', &[]),
            ]
        );
        assert_eq!(parse("m 1.0 1.0 2.0 2.0").unwrap()[1].command, b'l');
        assert_eq!(parse("L 1.0E-4 2.5").unwrap()[0].args[0], 1.0e-4);
    }

    #[test]
    fn refuses_what_is_not_canonical() {
        assert_eq!(parse("M 1.0"), None);
        assert_eq!(parse("X 1.0 2.0"), None);
        assert_eq!(parse("M1.0 2.0"), None);
        assert_eq!(parse("").unwrap(), []);
    }
}
