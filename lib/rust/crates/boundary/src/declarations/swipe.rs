//! `SwipeNative`, swipe typing's rules, answered by awnative's `suggest::swipe` module and the
//! suggest crate, and `SwipeRules`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, Kind, MEMBER, METHOD, Member, Method, Out, Param,
    Return, Type, arg, default,
};

const PROBABILITY: (&str, Param) = ("probability", Param::Int);
const USER_PROBABILITY: (&str, Param) = ("userProbability", Param::Int);
const INTS: Param = Param::Array(Element::Int);

/// A method of `SwipeNative`, answered by `rust` if it isn't empty.
const fn method(
    name: &'static str,
    params: &'static [(&'static str, Param)],
    returns: Return,
    rust: &'static str,
) -> Method {
    Method {
        name,
        params,
        returns,
        rust,
        ..METHOD
    }
}

pub const SWIPE_NATIVE: Class = Class {
    name: "SwipeNative",
    doc: "Swipe typing's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/suggest/swipe.rs and the suggest crate. [SwipeRules] wraps \
          them.",
    module: "suggest::swipe",
    file: "swipe",
    methods: &[
        method("maxGlideBonus", &[], Return::Float, ""),
        method(
            "glideBonus",
            &[
                ("pairCount", Param::Int),
                ("ngram", Param::Int),
                PROBABILITY,
            ],
            Return::Float,
            "",
        ),
        method(
            "rankGlide",
            &[
                ("costs", Param::Array(Element::Float)),
                ("pairCounts", INTS),
                ("ngrams", INTS),
                ("probabilities", INTS),
            ],
            Return::Array(Element::Int),
            "",
        ),
        method("maxProbability", &[], Return::Int, ""),
        method("common", &[], Return::Int, ""),
        method("costPerStep", &[], Return::Double, ""),
        method("learnedProbability", &[], Return::Int, ""),
        method(
            "isSwipeable",
            &[("word", Param::String)],
            Return::Boolean,
            "::suggest::swipe_words::is_swipeable",
        ),
        method(
            "swipeable",
            &[("text", Param::String), ("lengths", INTS)],
            Return::Array(Element::Int),
            "",
        ),
        method(
            "frequencyOf",
            &[PROBABILITY],
            Return::Double,
            "::suggest::swipe_words::frequency_of",
        ),
        method(
            "frequency",
            &[PROBABILITY, USER_PROBABILITY],
            Return::Double,
            "::suggest::swipe_words::frequency",
        ),
        method(
            "tunedFrequency",
            &[PROBABILITY, USER_PROBABILITY, ("boost", Param::Int)],
            Return::Double,
            "::suggest::swipe_words::tuned_frequency",
        ),
        method(
            "userWords",
            &[
                ("learned", Param::Strings),
                ("personalWords", Param::Strings),
                ("personalFrequencies", INTS),
            ],
            Return::Array(Element::Int),
            "",
        ),
    ],
    ..CLASS
};

/// A value of `SwipeRules`.
const fn value(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        kind: Kind::Val,
        ..MEMBER
    }
}

/// A function of `SwipeRules`.
const fn function(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        ..MEMBER
    }
}

pub const SWIPE_RULES: Api = Api {
    name: "SwipeRules",
    doc: "Swipe typing's rules, in Rust (lib/rust/crates/suggest's glide_ranker.rs and \
          swipe_words.rs): which words a swipe can spell, how strongly the glide classifier \
          favours each, and how the words before a swipe re-rank its guesses. The keyboard's \
          `GlideRanker` and `SwipeWords` call these and document the rules; they ask the \
          dictionary and the learned words, and pass what those say in.",
    module: "swipe",
    native: "SwipeNative",
    nested: &[Type::Record {
        name: "Context",
        doc: "What the words before a guess say for it; see [rankGlide].",
        data: false,
        constants: &[],
        fields: &[
            ("pairCount", "Int"),
            ("ngram", "Int"),
            ("probability", "Int"),
        ],
    }],
    members: &[
        value("maxGlideBonus", "The most [glideBonus] gives."),
        value("maxProbability", ""),
        value("common", "Words this common or more count the same."),
        value(
            "costPerStep",
            "What each step of probability below [common] costs a word, in nats.",
        ),
        value(
            "learnedProbability",
            "The probability every learned word counts as.",
        ),
        function(
            "glideBonus",
            "How much the words before a guess favour it, in nats, given how many times the user \
             typed it after them ([pairCount]) and the dictionary's [ngram] for it there and \
             [probability] alone.",
        ),
        Member {
            args: &[
                arg(
                    "pairCounts",
                    ArgKind::As(
                        "contexts: List<Context>",
                        "IntArray(contexts.size) { contexts[it].pairCount }",
                    ),
                ),
                arg(
                    "ngrams",
                    ArgKind::Fixed("IntArray(contexts.size) { contexts[it].ngram }"),
                ),
                arg(
                    "probabilities",
                    ArgKind::Fixed("IntArray(contexts.size) { contexts[it].probability }"),
                ),
            ],
            ..function(
                "rankGlide",
                "The guesses with these [costs] and [contexts], best first, as their indexes.",
            )
        },
        function("isSwipeable", ""),
        Member {
            args: &[
                arg(
                    "text",
                    ArgKind::As("words: List<String>", "words.joinToString(\"\")"),
                ),
                arg(
                    "lengths",
                    ArgKind::Fixed("IntArray(words.size) { words[it].length }"),
                ),
            ],
            result: Out::Then("List<String>", ".map { words[it] }"),
            ..function(
                "swipeable",
                "The [words] a swipe can spell, in order. They cross to Rust as one string and \
                 their lengths: a dictionary's 160,000 words one by one took longer in JNI calls \
                 than the Kotlin it replaced.",
            )
        },
        function(
            "frequencyOf",
            "The glide classifier's frequency for a word of [probability] (0-255).",
        ),
        function(
            "frequency",
            "The frequency of a word the dictionary gives [probability] and the user's words \
             [userProbability].",
        ),
        Member {
            args: &[default("boost", "0")],
            ..function(
                "tunedFrequency",
                "As [frequency], as the keyboard swipes: on one slope all the way up, where \
                 [frequency] counts every word from [common] up the same, and [boost] steps \
                 higher for the user's use of the word (LearnedWordStore.swipeBoosts; \
                 lib/rust/crates/suggest/src/swipe_words.rs).",
            )
        },
        Member {
            args: &[
                arg("learned", ArgKind::List),
                arg("personalWords", ArgKind::List),
            ],
            result: Out::Then(
                "Map<String, Int>",
                ".let { userWordMap(learned, personalWords, it) }",
            ),
            ..function(
                "userWords",
                "The user's words with their probabilities, from their [learned] spellings and \
                 their personal dictionary's [personalWords] at [personalFrequencies].",
            )
        },
    ],
    ..API
};
