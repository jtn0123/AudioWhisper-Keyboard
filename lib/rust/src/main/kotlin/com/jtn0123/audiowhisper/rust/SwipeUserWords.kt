package com.jtn0123.audiowhisper.rust

/**
 * The user's words from [SwipeNative.userWords]' [packed] places among [learned] then
 * [personalWords], each with its probability: a HashMap filled in the order the Kotlin it replaced
 * filled one, so it lists its words in the same order.
 */
internal fun userWordMap(learned: List<String>, personalWords: List<String>, packed: IntArray): Map<String, Int> {
    val probabilities = HashMap<String, Int>()
    for (i in packed.indices step 2) {
        val place = packed[i]
        val word = if (place < learned.size) learned[place] else personalWords[place - learned.size]
        probabilities[word] = packed[i + 1]
    }
    return probabilities
}
