//! `ScreenNative`, the decisions a walk of the screen makes, answered by awnative's `screen`
//! module and the reader crate, and `ScreenWalkRules`, the Kotlin over it.

use crate::{
    API, Api, CLASS, Class, Element, MEMBER, METHOD, Member, Method, Out, Param, Return, Type,
};

const INTS: Param = Param::Array(Element::Int);

pub const SCREEN_NATIVE: Class = Class {
    name: "ScreenNative",
    doc: "The screen walk's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/screen.rs. [ScreenWalkRules] wraps them.",
    module: "screen",
    methods: &[
        Method {
            name: "isSideways",
            params: &[
                ("rows", Param::Int),
                ("columns", Param::Int),
                ("upDown", Param::Boolean),
                ("across", Param::Boolean),
                ("className", Param::String),
            ],
            returns: Return::Boolean,
            rust: "::reader::screen::is_sideways",
            ..METHOD
        },
        Method {
            name: "visibleArea",
            params: &[("boxes", INTS)],
            returns: Return::Long,
            ..METHOD
        },
        Method {
            name: "visiblePart",
            params: &[("boxes", INTS)],
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        Method {
            name: "mainScroller",
            params: &[
                ("pieceScrollers", INTS),
                ("areas", Param::Array(Element::Long)),
            ],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "inList",
            params: &[
                ("pieceScrollers", INTS),
                ("parents", INTS),
                ("main", Param::Int),
            ],
            returns: Return::Array(Element::Boolean),
            ..METHOD
        },
        Method {
            name: "moveIntoView",
            params: &[
                ("first", Param::Int),
                ("last", Param::Int),
                ("top", Param::Int),
                ("bottom", Param::Int),
                ("canTurn", Param::Boolean),
            ],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "afterTurn",
            params: &[
                ("first", Param::Int),
                ("last", Param::Int),
                ("top", Param::Int),
                ("bottom", Param::Int),
            ],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "isInside",
            params: &[
                ("top", Param::Int),
                ("bottom", Param::Int),
                ("viewTop", Param::Int),
                ("viewBottom", Param::Int),
            ],
            returns: Return::Boolean,
            rust: "::reader::screen::is_inside",
            ..METHOD
        },
    ],
    ..CLASS
};

pub const SCREEN_WALK_RULES: Api = Api {
    name: "ScreenWalkRules",
    doc: "The decisions a walk of the screen and following along make, in Rust \
          (lib/rust/crates/reader/src/screen.rs): whether a scrollable view goes sideways, how \
          much of a box shows, which of the screen's lists is the one that scrolls and what is \
          inside it, and how to keep the words being read in view. The keyboard's \
          `reader.ScreenRules` wraps these.\n\n\
          A scroller is its index in the walk's list of them, and [NONE] is none. Two boxes go \
          in as one array of eight: a box's left, top, right and bottom, then the visible box's.",
    module: "screen",
    native: "ScreenNative",
    constants: &[("NONE", "", "-1", "No scroller.")],
    nested: &[
        Type::Enum {
            name: "Move",
            doc: "How to bring a web page's piece into view.",
            entries: &["STAY", "SHOW_ON_SCREEN", "TURN_PAGES"],
        },
        Type::Enum {
            name: "AfterTurn",
            doc: "What to do once the page has turned.",
            entries: &["SHOW_ON_SCREEN", "DONE", "TURN_AGAIN"],
        },
    ],
    members: &[
        Member {
            name: "isSideways",
            doc: "Whether a scrollable view goes sideways (a carousel or a pager): a collection \
                  of one row and several [columns], a view that only scrolls [across], or a \
                  horizontal scroll view or pager, by its [className], that can't go [upDown]. \
                  [rows] and [columns] are 0 for a view that isn't a collection.",
            ..MEMBER
        },
        Member {
            name: "visibleArea",
            doc: "How much of the first of [boxes] shows in the second, in square pixels.",
            ..MEMBER
        },
        Member {
            name: "visiblePart",
            doc: "The part of the first of [boxes] that shows in the second, or all of the \
                  second: its four sides.",
            ..MEMBER
        },
        Member {
            name: "mainScroller",
            doc: "The list, among scrollers with these visible [areas], given which scroller \
                  each piece of text is directly in ([pieceScrollers]): the one in sight that \
                  holds the most text itself, the bigger on a tie, the first of equals; [NONE] \
                  if none is in sight.",
            ..MEMBER
        },
        Member {
            name: "inList",
            doc: "For each piece of text, whether the scroller it is directly in \
                  ([pieceScrollers]) is [main] or inside it, given each scroller's [parents]; a \
                  parent comes before its child.",
            ..MEMBER
        },
        Member {
            name: "move",
            doc: "How to bring a web page's piece spanning [first] to [last] into a view from \
                  [top] to [bottom].",
            call: "moveIntoView",
            result: Out::Entry("Move"),
            ..MEMBER
        },
        Member {
            name: "afterTurn",
            doc: "What to do once the page has turned, the piece spanning [first] to [last].",
            result: Out::Entry("AfterTurn"),
            ..MEMBER
        },
        Member {
            name: "isInside",
            doc: "Whether an app's own view from [top] to [bottom] is well inside a view from \
                  [viewTop] to [viewBottom].",
            ..MEMBER
        },
    ],
    ..API
};
