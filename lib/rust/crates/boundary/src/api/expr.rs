//! A member's native call as Kotlin: on one line if it fits, else split as the Kotlin style
//! splits it.

/// A native call and what surrounds it: `before`, `call(args)`, then `after`. With `lend`, the
/// call is in that lambda, such as `LatinSession.lend(session) {`, inside `before` and `after`.
pub(super) struct Expr {
    pub(super) before: String,
    pub(super) call: String,
    pub(super) args: Vec<String>,
    pub(super) after: String,
    pub(super) lend: Option<String>,
}

impl Expr {
    /// The native call alone, without what surrounds it.
    fn bare(&self) -> Expr {
        Expr {
            before: String::new(),
            call: self.call.clone(),
            args: self.args.clone(),
            after: String::new(),
            lend: None,
        }
    }

    /// The expression on one line.
    pub(super) fn line(&self) -> String {
        let call = format!("{}({})", self.call, self.args.join(", "));
        match &self.lend {
            Some(lend) => format!("{}{lend} {call} }}{}", self.before, self.after),
            None => format!("{}{call}{}", self.before, self.after),
        }
    }

    /// The expression at `indent`: if it doesn't fit on one line, a lent call on lines of its
    /// own, else the body of the lambda after the call on a line of its own, else the arguments
    /// one to a line.
    pub(super) fn lines(&self, indent: &str) -> String {
        let one = format!("{indent}{}", self.line());
        if one.len() <= 120 {
            return one + "\n";
        }
        if let Some(lend) = &self.lend {
            let inner = self.bare().lines(&format!("{indent}    "));
            let close = format!("{indent}}}{}", self.after);
            assert!(close.len() <= 120, "too long: {close}");
            return format!("{indent}{}{lend}\n{inner}{close}\n", self.before);
        }
        if let Some((open, body, close)) = lambda(&self.after) {
            let args = self.args.join(", ");
            let head = format!("{indent}{}{}({args}){open}", self.before, self.call);
            let body = format!("{indent}    {body}");
            let close = format!("{indent}}}{close}");
            if [&head, &body, &close].iter().all(|line| line.len() <= 120) {
                return format!("{head}\n{body}\n{close}\n");
            }
        }
        let args: String = self.args.iter().map(|arg| argument(arg, indent)).collect();
        let mut close = format!("{indent}){}", self.after);
        if close.len() > 120 {
            // Each call chained after the first on a line of its own.
            let calls = chain(&self.after);
            close = format!("{indent}){}", calls[0]);
            for call in &calls[1..] {
                close.push_str(&format!("\n{indent}    {call}"));
            }
        }
        format!("{indent}{}{}(\n{args}{close}\n", self.before, self.call)
    }
}

/// One argument on its own line at `indent`, its lambda's body on a line of its own if the line
/// is too long and it has one.
fn argument(arg: &str, indent: &str) -> String {
    let one = format!("{indent}    {arg},\n");
    match lambda(arg) {
        Some((open, body, close)) if one.len() > 121 => {
            format!("{indent}    {open}\n{indent}        {body}\n{indent}    }}{close},\n")
        }
        _ => one,
    }
}

/// The calls chained in `after`, such as `.let { … }` and `?.also { … }`, split where a `.` or
/// `?.` starts one outside any braces or parentheses.
pub(super) fn chain(after: &str) -> Vec<&str> {
    let bytes = after.as_bytes();
    let mut calls = Vec::new();
    let (mut depth, mut start) = (0, 0);
    for (at, &c) in bytes.iter().enumerate() {
        let starts = match c {
            b'{' | b'(' | b'[' => {
                depth += 1;
                false
            }
            b'}' | b')' | b']' => {
                depth -= 1;
                false
            }
            b'?' => bytes.get(at + 1) == Some(&b'.'),
            b'.' => at == 0 || bytes[at - 1] != b'?',
            _ => false,
        };
        if starts && depth == 0 && at > start {
            calls.push(&after[start..at]);
            start = at;
        }
    }
    calls.push(&after[start..]);
    calls
}

/// The first lambda in `after`, such as `.map { Entry(it[0]) }.also { … }`: what opens it
/// (`.map {`), its body, and what follows its closing brace.
fn lambda(after: &str) -> Option<(&str, &str, &str)> {
    let open = after.find(" { ")?;
    let mut depth = 0;
    for (at, c) in after.char_indices().skip(open + 1) {
        match c {
            '{' => depth += 1,
            '}' if depth == 1 => {
                let body = after[open + 3..at].trim_end();
                return Some((&after[..open + 2], body, &after[at + 1..]));
            }
            '}' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// `head = expr`, on one line if it fits, else with the expression on the next.
pub(super) fn expression(head: &str, expr: &Expr) -> String {
    let one = format!("{head} = {}", expr.line());
    if one.len() <= 120 && !head.contains('\n') {
        return one + "\n";
    }
    let last = head.lines().last().unwrap_or_default();
    let indent = " ".repeat(last.len() - last.trim_start().len() + 4);
    format!("{head} =\n{}", expr.lines(&indent))
}
