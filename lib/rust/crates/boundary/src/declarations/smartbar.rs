//! `SmartbarNative`, the smartbar's font-independent rules, answered by awnative's
//! `render_smartbar` module, and `SmartbarRules`, the Kotlin over it, with `TextFit`, how a word
//! draws.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, MEMBER, METHOD, Member, Method, Module, Out, Param,
    Return, Type, arg,
};

const FLOATS: Param = Param::Array(Element::Float);
const CONFIG: (&str, Param) = ("config", FLOATS);
const FIT: Return = Return::Array(Element::Float);

/// A method of `SmartbarNative`.
const fn method(
    name: &'static str,
    params: &'static [(&'static str, Param)],
    returns: Return,
) -> Method {
    Method {
        name,
        params,
        returns,
        ..METHOD
    }
}

pub const SMARTBAR_NATIVE: Class = Class {
    name: "SmartbarNative",
    doc: "The smartbar's rules' entry points in libawnative: lib/rust/crates/awnative/src/\
          render_smartbar.rs. A text fit crosses as its squeeze and then its scale. \
          [SmartbarRules] wraps them.",
    module: "render_smartbar",
    file: "smartbar",
    methods: &[
        method(
            "layout",
            &[CONFIG, ("width", Param::Float), ("intrinsic", FLOATS)],
            Return::Array(Element::Float),
        ),
        method(
            "ellipsis",
            &[("advances", FLOATS), ("width", Param::Float)],
            Return::Int,
        ),
        method("roomy", &[CONFIG], Return::Boolean),
        method(
            "actions",
            &[CONFIG, ("graph", Param::Long), ("fields", Param::Strings)],
            Return::Strings,
        ),
        method("toggleWidth", &[CONFIG], Return::Float),
        method(
            "fitStep",
            &[
                ("squeeze", Param::Float),
                ("scale", Param::Float),
                ("measured", Param::Float),
                ("available", Param::Float),
            ],
            FIT,
        ),
        method(
            "fitText",
            &[
                ("available", Param::Float),
                ("measure", Param::Object("SmartbarRules.TextMeasure")),
            ],
            FIT,
        ),
    ],
    ..CLASS
};

pub const TEXT_FIT: (&str, Module, Type) = (
    "smartbar",
    Module::Rust,
    Type::Record {
        name: "TextFit",
        doc: "How a word draws, `smartbar_fit::TextFit`: [squeeze] narrows its glyphs only (text \
              scale X), [scale] sizes its font.",
        data: true,
        fields: &[("squeeze", "Float"), ("scale", "Float")],
        constants: &[("NATURAL", "TextFit", "TextFit(1f, 1f)", "")],
    },
);

pub const SMARTBAR_RULES: Api = Api {
    name: "SmartbarRules",
    doc: "The smartbar's layout and how its words fit, in Rust (lib/rust/crates/render/src/\
          smartbar.rs and smartbar_fit.rs), for the views and the renderer to agree.",
    module: "smartbar",
    native: "SmartbarNative",
    nested: &[Type::Interface {
        name: "TextMeasure",
        doc: "A word's unsqueezed width at a font scale, with the renderer's own text layout.",
        members: &[("fun measure(scale: Float): Float", "")],
    }],
    constants: &[(
        "TEXT_FIT_STEPS",
        "",
        "3",
        "`smartbar_fit::TEXT_FIT_STEPS`: most measurements made while fitting one word.",
    )],
    members: &[
        Member {
            name: "layout",
            doc: "Where the smartbar's slots go in a row [width] wide, for [config] and the \
                  words' [intrinsic] widths, packed as `smartbar::Layout::packed`.",
            ..MEMBER
        },
        Member {
            name: "ellipsis",
            doc: "How many characters of these [advances] fit in [width] with an ellipsis after.",
            ..MEMBER
        },
        Member {
            name: "roomy",
            doc: "Words show in Gboard's row: the mic, the words and the arrow; see \
                  `smartbar::Config::roomy`.",
            ..MEMBER
        },
        Member {
            name: "actions",
            doc: "What each action shows, `smartbar_actions::call`, for [config] and [fields]: a \
                  [SmartbarBar] row, then a [SmartbarAction] row for each action, whose keys \
                  [graph] answers for. The answer is a [SmartbarPlan] row, then a \
                  [SmartbarActionPlan] row for each action.",
            args: &[arg("graph", ArgKind::Handle("KeyGraph"))],
            ..MEMBER
        },
        Member {
            name: "toggleWidth",
            doc: "The arrow's slot width in pixels: three quarters of the row while words show, \
                  else the row.",
            ..MEMBER
        },
        Member {
            name: "fitStep",
            doc: "One step fitting a word, `smartbar_fit::fit_step`: squeeze first, then shrink.",
            args: &[
                arg("squeeze", ArgKind::As("fit: TextFit", "fit.squeeze")),
                arg("scale", ArgKind::Fixed("fit.scale")),
            ],
            result: Out::Record("TextFit"),
            ..MEMBER
        },
        Member {
            name: "fitText",
            doc: "How a word draws in [available], `smartbar_fit::fit_text`: [measure] gives its \
                  unsqueezed width at a font scale, with the renderer's own text layout. Measures \
                  once at full size, then again only when the font shrinks.",
            args: &[arg(
                "measure",
                ArgKind::As("measure: (Float) -> Float", "TextMeasure { measure(it) }"),
            )],
            result: Out::Record("TextFit"),
            ..MEMBER
        },
    ],
    ..API
};
