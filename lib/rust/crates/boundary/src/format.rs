//! Laying out generated code as the repo formats it by hand: the Rust as rustfmt's defaults would
//! (lines of at most 100 columns, a call's arguments on one line only up to 60), the Kotlin in
//! 120 columns, and doc comments wrapped at 100. The tests check the Rust against rustfmt itself.

/// rustfmt's `max_width`.
const RUST_WIDTH: usize = 100;
/// rustfmt's `fn_call_width`: arguments wider than this go one to a line.
const RUST_CALL_ARGS: usize = 60;
/// The Kotlin line length in .editorconfig.
const KOTLIN_WIDTH: usize = 120;
/// Where hand-written doc comments wrap.
const DOC_WIDTH: usize = 100;

/// `text` as `indent`ed lines of `prefix` and words, wrapped at the doc width, with a line of
/// `prefix` alone between paragraphs, which `text` separates with a blank line.
fn wrap(text: &str, indent: &str, prefix: &str) -> String {
    let blank = format!("{indent}{prefix}\n");
    let paragraphs: Vec<String> = text
        .split("\n\n")
        .map(|paragraph| wrap_paragraph(paragraph, indent, prefix))
        .collect();
    paragraphs.join(&blank)
}

/// One paragraph of `text`, as [`wrap`] lays it out.
fn wrap_paragraph(text: &str, indent: &str, prefix: &str) -> String {
    let start = format!("{indent}{prefix} ");
    let mut lines = String::new();
    let mut line = start.clone();
    for word in text.split_whitespace() {
        if line.len() > start.len() && line.len() + 1 + word.len() > DOC_WIDTH {
            lines.push_str(&line);
            lines.push('\n');
            line.clone_from(&start);
        }
        if line.len() > start.len() {
            line.push(' ');
        }
        line.push_str(word);
    }
    lines.push_str(&line);
    lines.push('\n');
    lines
}

/// A Rust doc comment of `text`, `///` lines at `indent`.
pub fn rust_doc(text: &str, indent: &str) -> String {
    wrap(text, indent, "///")
}

/// A Rust comment of `text`, `//` lines at `indent`.
pub fn rust_comment(text: &str, indent: &str) -> String {
    wrap(text, indent, "//")
}

/// A KDoc of `text` at `indent`: one line if it fits, else a block.
pub fn kdoc(text: &str, indent: &str) -> String {
    let one = format!(
        "{indent}/** {} */",
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    );
    if one.len() <= DOC_WIDTH && !text.contains("\n\n") {
        return one + "\n";
    }
    format!("{indent}/**\n{}{indent} */\n", wrap(text, indent, " *"))
}

/// A Kotlin declaration `head(params)tail` at `indent`: on one line if it fits, else one
/// parameter to a line with a trailing comma, as the hand-written Kotlin does.
pub fn kotlin_declaration(indent: &str, head: &str, params: &[String], tail: &str) -> String {
    let one = format!("{indent}{head}({}){tail}", params.join(", "));
    if one.len() <= KOTLIN_WIDTH {
        return one + "\n";
    }
    let params: String = params
        .iter()
        .map(|it| format!("{indent}    {it},\n"))
        .collect();
    format!("{indent}{head}(\n{params}{indent}){tail}\n")
}

/// A Rust function signature `head(params) -> returns {`, as rustfmt lays it out: on one line if
/// it fits, else one parameter to a line. A `returns` of `()` is left out.
pub fn rust_signature(head: &str, params: &[String], returns: &str) -> String {
    let tail = if returns == "()" {
        " {".to_owned()
    } else {
        format!(" -> {returns} {{")
    };
    let one = format!("{head}({}){tail}", params.join(", "));
    if one.len() <= RUST_WIDTH {
        return one + "\n";
    }
    let params: String = params.iter().map(|it| format!("    {it},\n")).collect();
    format!("{head}(\n{params}){tail}\n")
}

/// An entry point's body: in an `unsafe` block, `call::<returns>(env, |env| { lets;
/// callee(args).result() })`, as rustfmt lays it out. `result` takes the function's value, or its
/// `jni::Result`. With nothing to read first and a closure narrow enough to be a call's argument
/// on one line, it is all one line; otherwise the closure is a block.
pub fn rust_body(
    returns: &str,
    lets: &[(String, String)],
    callee: &str,
    args: &[String],
) -> String {
    let uses_env = !lets.is_empty() || args.first().is_some_and(|arg| arg == "env");
    let close = if uses_env { "env" } else { "_" };
    let call = format!("call::<{returns}>");
    let one = format!("env, |{close}| {callee}({}).result()", args.join(", "));
    let line = format!("    unsafe {{ {call}({one}) }}");
    if lets.is_empty() && one.len() <= RUST_CALL_ARGS && line.len() <= RUST_WIDTH {
        return line + "\n";
    }
    let mut body = format!("    unsafe {{\n        {call}(env, |{close}| {{\n");
    for (name, value) in lets {
        body.push_str(&rust_let(12, name, value));
    }
    body.push_str(&chain_result(12, rust_call(12, callee, args)));
    body.push_str("        })\n    }\n");
    body
}

/// `call`, laid out at `indent`, with `.result()` after it as rustfmt chains it: on the call's
/// line if the call is one line and there is room, else on a line of its own.
fn chain_result(indent: usize, call: String) -> String {
    const RESULT: &str = ".result()";
    let pad = " ".repeat(indent);
    if call.lines().count() > 1 {
        return format!("{call}{pad}{RESULT}\n");
    }
    let line = call.trim_end();
    if line.len() + RESULT.len() <= RUST_WIDTH {
        return format!("{line}{RESULT}\n");
    }
    format!("{call}{pad}    {RESULT}\n")
}

/// An entry point's body whose call passes a struct of its arguments, `name { fields }`, after
/// the environment if `env`; a field whose value is its own name is written once, as clippy wants.
pub fn rust_struct_body(
    returns: &str,
    lets: &[(String, String)],
    callee: &str,
    env: bool,
    name: &str,
    fields: &[(String, String)],
) -> String {
    let close = if env || !lets.is_empty() { "env" } else { "_" };
    let mut body = format!("    unsafe {{\n        call::<{returns}>(env, |{close}| {{\n");
    for (field, value) in lets {
        body.push_str(&rust_let(12, field, value));
    }
    // rustfmt lets a struct overflow its call's line only as the call's one argument.
    let (head, pad, tail) = if env {
        let head =
            format!("            {callee}(\n                env,\n                {name} {{");
        (head, " ".repeat(20), "                },\n            )\n")
    } else {
        let head = format!("            {callee}({name} {{");
        (head, " ".repeat(16), "            })\n")
    };
    assert!(
        head.lines().all(|line| line.len() <= RUST_WIDTH),
        "a call too long: {callee}"
    );
    body.push_str(&head);
    body.push('\n');
    for (field, value) in fields {
        if field == value {
            body.push_str(&format!("{pad}{field},\n"));
        } else {
            body.push_str(&format!("{pad}{field}: {value},\n"));
        }
    }
    body.push_str(tail);
    body.push_str("            .result()\n");
    body.push_str("        })\n    }\n");
    body
}

/// `use path::{names};`, as rustfmt lays it out: braces only for more than one name, and the
/// names filling indented lines when they don't fit on one.
pub fn rust_use(path: &str, names: &[String]) -> String {
    if let [name] = names {
        return format!("use {path}::{name};\n");
    }
    let one = format!("use {path}::{{{}}};", names.join(", "));
    if one.len() <= RUST_WIDTH {
        return one + "\n";
    }
    let mut text = format!("use {path}::{{\n");
    let mut line = String::new();
    for name in names {
        let item = format!("{name},");
        if !line.is_empty() && line.len() + 1 + item.len() > RUST_WIDTH {
            text.push_str(&line);
            text.push('\n');
            line.clear();
        }
        line.push_str(if line.is_empty() { "    " } else { " " });
        line.push_str(&item);
    }
    text.push_str(&line);
    text.push_str("\n};\n");
    text
}

/// A call `callee(args)` at `indent`, as rustfmt lays it out: on one line if it fits and the
/// arguments are narrow enough; else, on the next line, the arguments together if they are narrow
/// enough or one to a line if not.
fn rust_call(indent: usize, callee: &str, args: &[String]) -> String {
    let pad = " ".repeat(indent);
    let joined = args.join(", ");
    let narrow = joined.len() <= RUST_CALL_ARGS;
    if narrow && indent + callee.len() + joined.len() + 2 <= RUST_WIDTH {
        return format!("{pad}{callee}({joined})\n");
    }
    let inner = " ".repeat(indent + 4);
    let args = if narrow {
        format!("{inner}{joined},\n")
    } else {
        args.iter().map(|it| format!("{inner}{it},\n")).collect()
    };
    format!("{pad}{callee}(\n{args}{pad})\n")
}

/// `let name = value;` at `indent`, as rustfmt lays it out: on one line if it fits, else with the
/// value on the next line, which must fit there.
fn rust_let(indent: usize, name: &str, value: &str) -> String {
    let pad = " ".repeat(indent);
    let one = format!("{pad}let {name} = {value};");
    if one.len() <= RUST_WIDTH {
        return one + "\n";
    }
    let next = format!("{pad}    {value};");
    assert!(
        next.len() <= RUST_WIDTH,
        "a name too long to lay out: {name}"
    );
    format!("{pad}let {name} =\n{next}\n")
}
