package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SwipeRules

/** The [SwipeWords] of a plain word list, its frequencies from [probability], with no user words. */
fun SwipeWords.Companion.fromDictionary(words: List<String>, probability: (String) -> Int): SwipeWords =
    SwipeWords(SwipeRules.swipeable(words), probability, emptyMap()) { values ->
        values.filter { probability(it) == NOT_A_PROBABILITY }
    }
