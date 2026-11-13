package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SwipeRules

/** Synthetic lookup adapter retained for the pre-existing reference tests only. */
fun SwipeWords.Companion.fromDictionary(words: List<String>, probability: (String) -> Int): SwipeWords =
    SwipeWords(SwipeRules.swipeable(words), probability, emptyMap()) { values ->
        values.filter { probability(it) == NOT_A_PROBABILITY }
    }
