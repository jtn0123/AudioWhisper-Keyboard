//! The public Kotlin over the `external fun`s: the code every binding used to write by hand
//! around its native object. An [`Api`] is an `object` of members that each forward to one
//! native method, or a class that owns a native handle and frees it through `RustLibrary`'s
//! cleaner, with the records and enums its members use. Each member converts its arguments and
//! result with a fixed vocabulary; anything more is hand-written, beside it, as an extension.

use crate::format::{kdoc, kotlin_declaration};
use crate::kotlin::{KotlinFile, kotlin_header, simple_type};
use crate::{Class, Module, Param, Return};

mod handle;
mod records;
#[cfg(test)]
mod tests;

pub use records::{BIT, Bit, BitKind};

/// A public Kotlin type over native methods, in its own file.
#[derive(Debug)]
pub struct Api {
    /// The Kotlin name, such as `SpacingRules`.
    pub name: &'static str,
    pub doc: &'static str,
    /// The declarations file it is in, `src/declarations/<module>.rs`.
    pub module: &'static str,
    pub kotlin: Module,
    pub shape: Shape,
    /// The native object its members call, unless a member names another.
    pub native: &'static str,
    pub members: &'static [Member],
    /// Records and enums declared inside it.
    pub nested: &'static [Type],
    /// `const val`s, or `val`s when a type is given: (name, type, value, doc).
    pub constants: &'static [(&'static str, &'static str, &'static str, &'static str)],
    /// Whether a class has a `companion object` even with nothing generated in it, for
    /// hand-written extensions of it.
    pub companion: bool,
}

/// An api's defaults, for the fields a declaration leaves out with `..API`.
pub const API: Api = Api {
    name: "",
    doc: "",
    module: "",
    kotlin: Module::Rust,
    shape: Shape::Object,
    native: "",
    members: &[],
    nested: &[],
    constants: &[],
    companion: false,
};

/// What kind of Kotlin type an [`Api`] is.
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    /// An `object` whose members call static native methods.
    Object,
    /// A class that owns a native handle; its members pass the handle as their method's first
    /// argument.
    Handle(Handle),
}

/// How a handle class gets, guards and frees its handle.
#[derive(Clone, Copy, Debug)]
pub struct Handle {
    /// The constructor's parameters, which `create` takes, with any defaults.
    pub params: &'static [Arg],
    /// The native method that makes the handle; empty for a class an `internal constructor`
    /// gives the handle to.
    pub create: &'static str,
    /// The native method that frees it.
    pub free: &'static str,
    /// Whether the class is `AutoCloseable`, freeing the handle on `close()` and leaving 0 for
    /// later calls, which the native side refuses.
    pub close: bool,
    /// Whether every member is `@Synchronized`; otherwise each keeps the object reachable until
    /// its call returns.
    pub synchronized: bool,
}

/// A handle class's defaults, for `..HANDLE`.
pub const HANDLE: Handle = Handle {
    params: &[],
    create: "",
    free: "",
    close: false,
    synchronized: false,
};

/// One member, calling one native method.
#[derive(Debug)]
pub struct Member {
    /// The Kotlin name; the native method's too unless `call` names it.
    pub name: &'static str,
    pub doc: &'static str,
    pub kind: Kind,
    /// The native method, when its name differs.
    pub call: &'static str,
    /// The native object, when not the api's.
    pub native: &'static str,
    /// How parameters differ from the native method's, by name; the rest pass as they are.
    pub args: &'static [Arg],
    pub result: Out,
    /// Whether a handle class's member is static, in its `companion object`.
    pub companion: bool,
    /// Whether it is an `operator fun`.
    pub operator: bool,
}

/// A member's defaults, for `..MEMBER`.
pub const MEMBER: Member = Member {
    name: "",
    doc: "",
    kind: Kind::Fun,
    call: "",
    native: "",
    args: &[],
    result: Out::Same,
    companion: false,
    operator: false,
};

/// Whether a member is a function or a property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Fun,
    /// A `val` read once, when the object is made.
    Val,
    /// A `val` read on first use.
    Lazy,
}

/// How a parameter differs from the native method's, which it is matched to by name.
#[derive(Clone, Copy, Debug)]
pub struct Arg {
    pub name: &'static str,
    pub kind: ArgKind,
    /// Its default value, if any, as Kotlin.
    pub default: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgKind {
    /// As the native method takes it.
    Same,
    /// A crates/wire enum of this name, passing its `code`.
    Code(&'static str),
    /// A Kotlin enum of this name, passing its `ordinal`.
    Ordinal(&'static str),
    /// A `List` for a string array, passing `toTypedArray()`.
    List,
    /// A `CharSequence` for a string, or a `CharSequence?` for a nullable one, passing
    /// `toString()`.
    Text,
    /// An api of this name that owns a handle, passing its handle and keeping it reachable.
    Handle(&'static str),
    /// A nullable api of this name, passing its handle or 0.
    NullableHandle(&'static str),
    /// No parameter: the native method gets this Kotlin expression.
    Fixed(&'static str),
    /// Declared as this Kotlin parameter instead, such as `params: Map<String, String>`, the
    /// native method getting this one-line expression of it. Another native parameter can take
    /// the same Kotlin one through [`ArgKind::Fixed`].
    As(&'static str, &'static str),
}

/// A parameter that only changes its default.
pub const fn default(name: &'static str, default: &'static str) -> Arg {
    Arg {
        name,
        kind: ArgKind::Same,
        default,
    }
}

/// A parameter of another kind.
pub const fn arg(name: &'static str, kind: ArgKind) -> Arg {
    Arg {
        name,
        kind,
        default: "",
    }
}

/// How a member converts the native result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Out {
    Same,
    /// An array as a `List`, with `asList()`.
    List,
    /// An `Int` as the entry of a Kotlin enum of this name, by ordinal.
    Entry(&'static str),
    /// An `Int` as the entry of a Kotlin enum of this name, or `null` for a negative one.
    NullableEntry(&'static str),
    /// The result passed to the constructor of this type.
    Wrap(&'static str),
    /// A row read as the record of this name, a field an element; a nullable row reads as null.
    Record(&'static str),
    /// A row read as a list of the record of this name, each in as many elements as it has
    /// fields.
    Records(&'static str),
    /// A row of two elements read as a `Pair` of these types, as a record's fields are read.
    Pair(&'static str, &'static str),
}

/// A record or enum that an api's members take or return.
#[derive(Debug)]
pub enum Type {
    /// `class Name(val field: Type, …)`, a `data class` if `data`.
    Record {
        name: &'static str,
        doc: &'static str,
        data: bool,
        /// (name, type).
        fields: &'static [(&'static str, &'static str)],
    },
    /// `enum class Name { A, B }`.
    Enum {
        name: &'static str,
        doc: &'static str,
        entries: &'static [&'static str],
    },
    /// `value class Name(private val bits: Int)`, a `Long` if `long`, whose properties read its
    /// bits; its constructor is `internal` if `internal`.
    Bits {
        name: &'static str,
        doc: &'static str,
        long: bool,
        internal: bool,
        fields: &'static [Bit],
    },
}

/// The Kotlin files for `apis`, whose members call methods of `classes`, each with its module.
pub fn api_files(apis: &[Api], classes: &[Class]) -> Vec<(Module, KotlinFile)> {
    apis.iter()
        .map(|api| {
            let path = format!("{}/{}.kt", api.kotlin.package().replace('.', "/"), api.name);
            let text = api_file(api, apis, classes);
            (api.kotlin, KotlinFile { path, text })
        })
        .collect()
}

/// The native method `name` of the class `native`.
pub(super) fn method<'a>(classes: &'a [Class], native: &str, name: &str) -> &'a crate::Method {
    let class = classes.iter().find(|class| class.name == native);
    let class = class.unwrap_or_else(|| panic!("no native class {native}"));
    let method = class.methods.iter().find(|method| method.name == name);
    method.unwrap_or_else(|| panic!("{native} has no method {name}"))
}

fn api_file(api: &Api, apis: &[Api], classes: &[Class]) -> String {
    let mut body = String::new();
    for nested in api.nested {
        push_block(&mut body, &type_text(nested, "    "));
    }
    if !api.constants.is_empty() && matches!(api.shape, Shape::Object) {
        push_block(&mut body, &constants(api.constants, "    "));
    }
    let head = match api.shape {
        Shape::Object => {
            for member in api.members {
                push_block(
                    &mut body,
                    &member_text(api, member, None, classes, apis, "    "),
                );
            }
            format!("object {} {{\n", api.name)
        }
        Shape::Handle(handle) => handle::handle_class(api, handle, classes, apis, &mut body),
    };
    let mut imports = Vec::new();
    if body.contains("Reference.reachabilityFence") {
        imports.push("java.lang.ref.Reference".to_owned());
    }
    let source = format!("src/declarations/{}.rs", api.module);
    let mut text = kotlin_header(&source, api.kotlin, &imports);
    text.push_str(&kdoc(api.doc, ""));
    text.push_str(&head);
    if matches!(api.shape, Shape::Handle(_)) && !body.is_empty() {
        text.push('\n');
    }
    text.push_str(&body);
    text.push_str("}\n");
    text
}

/// Adds `block` to a type's body, after a blank line if it already has something.
pub(super) fn push_block(body: &mut String, block: &str) {
    if !body.is_empty() {
        body.push('\n');
    }
    body.push_str(block);
}

pub(super) fn constants(constants: &[(&str, &str, &str, &str)], indent: &str) -> String {
    let mut text = String::new();
    for (name, ty, value, doc) in constants {
        if !doc.is_empty() {
            text.push_str(&kdoc(doc, indent));
        }
        if ty.is_empty() {
            text.push_str(&format!("{indent}const val {name} = {value}\n"));
        } else {
            text.push_str(&format!("{indent}val {name}: {ty} = {value}\n"));
        }
    }
    text
}

/// A parameter as Kotlin declares it, with `native`'s type unless the arg says otherwise.
fn param_text(arg: &Arg, native: Param) -> String {
    let ty = match arg.kind {
        ArgKind::Same => simple_type(&native.kotlin()).0,
        ArgKind::Code(ty) | ArgKind::Ordinal(ty) | ArgKind::Handle(ty) => ty.to_owned(),
        ArgKind::NullableHandle(ty) => format!("{ty}?"),
        ArgKind::List => match native {
            Param::NullableStrings => "List<String?>".to_owned(),
            _ => "List<String>".to_owned(),
        },
        ArgKind::Text => match native {
            Param::String => "CharSequence".to_owned(),
            Param::NullableString => "CharSequence?".to_owned(),
            other => panic!("{}: no text for {other:?}", arg.name),
        },
        ArgKind::Fixed(_) => unreachable!("a fixed argument has no parameter"),
        ArgKind::As(declared, _) => return with_default(declared.to_owned(), arg.default),
    };
    with_default(format!("{}: {ty}", arg.name), arg.default)
}

/// `param`, with ` = default` after it if there is one.
fn with_default(param: String, default: &str) -> String {
    if default.is_empty() {
        param
    } else {
        format!("{param} = {default}")
    }
}

/// How each of a native method's parameters is declared publicly, and what it is passed.
pub(super) fn params_and_args(
    native: &crate::Method,
    args: &[Arg],
    skip_handle: bool,
) -> (Vec<String>, Vec<String>) {
    let mut params = Vec::new();
    let mut passed = Vec::new();
    for (i, (name, param)) in native.params.iter().enumerate() {
        if skip_handle && i == 0 {
            passed.push("handle".to_owned());
            continue;
        }
        let arg = args
            .iter()
            .find(|arg| arg.name == *name)
            .copied()
            .unwrap_or(Arg {
                name,
                kind: ArgKind::Same,
                default: "",
            });
        passed.push(match arg.kind {
            ArgKind::Same => name.to_string(),
            ArgKind::Code(_) => format!("{name}.code"),
            ArgKind::Ordinal(_) => format!("{name}.ordinal"),
            ArgKind::List => format!("{name}.toTypedArray()"),
            ArgKind::Text if *param == Param::NullableString => format!("{name}?.toString()"),
            ArgKind::Text => format!("{name}.toString()"),
            ArgKind::Handle(_) => format!("{name}.handle"),
            ArgKind::NullableHandle(_) => format!("{name}?.handle ?: 0L"),
            ArgKind::Fixed(value) | ArgKind::As(_, value) => value.to_owned(),
        });
        if !matches!(arg.kind, ArgKind::Fixed(_)) {
            params.push(param_text(&arg, *param));
        }
    }
    for arg in args {
        assert!(
            native.params.iter().any(|(name, _)| *name == arg.name),
            "{} has no parameter {}",
            native.name,
            arg.name
        );
    }
    (params, passed)
}

/// The public type of `returns` once `out` converts it, or `None` for nothing.
fn result_type(returns: Return, out: Out) -> Option<String> {
    match out {
        _ if returns == Return::Unit => None,
        Out::Same => Some(returns.kotlin()),
        Out::List => Some(match returns {
            Return::Strings => "List<String>".to_owned(),
            Return::Array(_) => format!("List<{}>", returns.kotlin().trim_end_matches("Array")),
            Return::IntArrays => "List<IntArray>".to_owned(),
            Return::StringArrays => "List<Array<String>>".to_owned(),
            other => panic!("no list of {other:?}"),
        }),
        Out::Entry(ty) | Out::Wrap(ty) => Some(ty.to_owned()),
        Out::NullableEntry(ty) => Some(format!("{ty}?")),
        Out::Record(ty) => Some(records::record_type(ty, false, returns)),
        Out::Records(ty) => Some(records::record_type(ty, true, returns)),
        Out::Pair(first, second) => Some(format!("Pair<{first}, {second}>")),
    }
}

/// A native call and what surrounds it: `before`, `call(args)`, then `after`.
struct Expr {
    before: String,
    call: String,
    args: Vec<String>,
    after: String,
}

impl Expr {
    /// The expression on one line.
    fn line(&self) -> String {
        format!(
            "{}{}({}){}",
            self.before,
            self.call,
            self.args.join(", "),
            self.after
        )
    }

    /// The expression at `indent`, its arguments one to a line if it doesn't fit on one.
    fn lines(&self, indent: &str) -> String {
        let one = format!("{indent}{}", self.line());
        if one.len() <= 120 {
            return one + "\n";
        }
        let args: String = self
            .args
            .iter()
            .map(|arg| format!("{indent}    {arg},\n"))
            .collect();
        format!(
            "{indent}{}{}(\n{args}{indent}){}\n",
            self.before, self.call, self.after
        )
    }
}

/// The text around a native call that returns `returns` and `out` converts, for `api`.
fn converted(out: Out, returns: Return, api: &Api, apis: &[Api]) -> (String, String) {
    match out {
        Out::Same => (String::new(), String::new()),
        Out::List => (String::new(), ".asList()".to_owned()),
        Out::Entry(ty) => (format!("{ty}.entries["), "]".to_owned()),
        Out::NullableEntry(ty) => (
            String::new(),
            format!(".let {{ if (it < 0) null else {ty}.entries[it] }}"),
        ),
        Out::Wrap(ty) => (format!("{ty}("), ")".to_owned()),
        Out::Record(ty) => (
            String::new(),
            records::record(ty, false, returns, api, apis),
        ),
        Out::Records(ty) => (String::new(), records::record(ty, true, returns, api, apis)),
        Out::Pair(first, second) => (
            String::new(),
            records::pair([first, second], returns, api, apis),
        ),
    }
}

/// One member: its signature and the native call it makes.
pub(super) fn member_text(
    api: &Api,
    member: &Member,
    handle: Option<Handle>,
    classes: &[Class],
    apis: &[Api],
    indent: &str,
) -> String {
    let native_name = if member.native.is_empty() {
        api.native
    } else {
        member.native
    };
    let call = if member.call.is_empty() {
        member.name
    } else {
        member.call
    };
    let native = method(classes, native_name, call);
    let own = handle.is_some() && !member.companion;
    let (params, passed) = params_and_args(native, member.args, own);
    let (before, after) = converted(member.result, native.returns, api, apis);
    let mut expr = Expr {
        before,
        call: format!("{native_name}.{call}"),
        args: passed,
        after,
    };
    let mut fences: Vec<String> = Vec::new();
    for arg in member.args {
        if let ArgKind::Handle(ty) | ArgKind::NullableHandle(ty) = arg.kind {
            assert!(apis.iter().any(|api| api.name == ty), "no api {ty}");
            fences.push(arg.name.to_owned());
        }
    }
    let synchronized = handle.is_some_and(|handle| handle.synchronized) && own;
    if own && !synchronized {
        fences.push("this".to_owned());
    }
    if !fences.is_empty() {
        let fences: Vec<String> = fences
            .iter()
            .map(|it| format!("Reference.reachabilityFence({it})"))
            .collect();
        expr.after
            .push_str(&format!(".also {{ {} }}", fences.join("; ")));
    }
    let returns = result_type(native.returns, member.result);
    let mut text = String::new();
    if !member.doc.is_empty() {
        text.push_str(&kdoc(member.doc, indent));
    }
    let typed = returns.map_or_else(String::new, |ty| format!(": {ty}"));
    match member.kind {
        Kind::Fun => {
            if synchronized {
                text.push_str(&format!("{indent}@Synchronized\n"));
            }
            let operator = if member.operator { "operator " } else { "" };
            let head = format!("{operator}fun {}", member.name);
            let signature = kotlin_declaration(indent, &head, &params, &typed);
            text.push_str(&expression(signature.trim_end(), &expr));
        }
        Kind::Val | Kind::Lazy => {
            assert!(
                params.is_empty(),
                "{}: a property takes nothing",
                member.name
            );
            let head = format!("{indent}val {}{typed}", member.name);
            if member.kind == Kind::Val {
                text.push_str(&expression(&head, &expr));
            } else {
                text.push_str(&format!("{head} by lazy {{ {} }}\n", expr.line()));
            }
        }
    }
    text
}

/// `head = expr`, on one line if it fits, else with the expression on the next.
fn expression(head: &str, expr: &Expr) -> String {
    let one = format!("{head} = {}", expr.line());
    if one.len() <= 120 && !head.contains('\n') {
        return one + "\n";
    }
    let last = head.lines().last().unwrap_or_default();
    let indent = " ".repeat(last.len() - last.trim_start().len() + 4);
    format!("{head} =\n{}", expr.lines(&indent))
}

/// A record or enum at `indent`.
fn type_text(ty: &Type, indent: &str) -> String {
    match ty {
        Type::Record {
            name,
            doc,
            data,
            fields,
        } => {
            let mut text = kdoc(doc, indent);
            let fields: Vec<String> = fields
                .iter()
                .map(|(name, ty)| format!("val {name}: {ty}"))
                .collect();
            let head = format!("{}class {name}", if *data { "data " } else { "" });
            text.push_str(&kotlin_declaration(indent, &head, &fields, ""));
            text
        }
        Type::Enum { name, doc, entries } => {
            format!(
                "{}{indent}enum class {name} {{ {} }}\n",
                kdoc(doc, indent),
                entries.join(", ")
            )
        }
        Type::Bits { .. } => records::bits_text(ty, indent),
    }
}

/// The Kotlin files for top-level `types`, one each, generated from `module`'s declarations.
pub fn type_files(types: &[(&str, Module, Type)]) -> Vec<(Module, KotlinFile)> {
    types
        .iter()
        .map(|(module, kotlin, ty)| {
            let name = match ty {
                Type::Record { name, .. } | Type::Enum { name, .. } | Type::Bits { name, .. } => {
                    *name
                }
            };
            let source = format!("src/declarations/{module}.rs");
            let mut text = kotlin_header(&source, *kotlin, &[]);
            text.push_str(&type_text(ty, ""));
            let path = format!("{}/{name}.kt", kotlin.package().replace('.', "/"));
            (*kotlin, KotlinFile { path, text })
        })
        .collect()
}
