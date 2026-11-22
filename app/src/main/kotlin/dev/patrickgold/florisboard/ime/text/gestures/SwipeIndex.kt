package dev.patrickgold.florisboard.ime.text.gestures

import androidx.collection.SparseArrayCompat
import com.jtn0123.audiowhisper.rust.GlideChar
import com.jtn0123.audiowhisper.rust.GlideIndex
import com.jtn0123.audiowhisper.rust.GlideKeys
import com.jtn0123.audiowhisper.rust.GlideScores
import com.jtn0123.audiowhisper.rust.GlideWords
import com.jtn0123.audiowhisper.suggest.GlideGuess
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.keyboard.Key
import java.text.Normalizer
import kotlin.math.ln

/**
 * AudioWhisper: glide typing's words indexed for one layout by the Rust scorer (lib/rust), which
 * replaced the classifier's Kotlin pruning and scoring and gives the same words at the same costs.
 * Kotlin still decides what each character means, so case and accents follow Java's rules: see
 * [describe].
 */
class SwipeIndex private constructor(
    /** The words indexed, kept to tell whether this index is for the current word list. */
    val words: List<String>,
    private val index: GlideIndex,
) {
    /**
     * Each word's frequency, NaN until a swipe needs it: a lookup costs a native dictionary call,
     * and a word's frequency doesn't change without the word list changing. An index serves one
     * frequency, so a caller uses either [suggestions] or [tunedSuggestions].
     */
    private val frequencies = FloatArray(words.size) { Float.NaN }

    /**
     * The [max] likeliest words for [swipe], drawn on [keys], with their costs, as FlorisBoard's
     * classifier ranked them; [frequency] gives a word's (see SwipeWords.frequency).
     */
    fun suggestions(
        swipe: StatisticalGlideTypingClassifier.Gesture,
        keys: List<Key>,
        max: Int,
        frequency: (String) -> Double,
    ): List<GlideGuess> {
        if (keys.isEmpty() || swipe.isEmpty) return emptyList()
        return index.query(swipe.xs(), swipe.ys(), glideKeys(keys)).use { query ->
            guesses(query.candidates(), frequency) { candidates, frequencies -> query.score(candidates, frequencies, max) }
        }
    }

    /**
     * As [suggestions], as the keyboard ranks them: with what measured better than FlorisBoard's
     * classifier (GlideQuery.scoreTuned), which also weighs where the finger slowed by [swipe]'s
     * times; [frequency] gives a word's as SwipeWords.tunedFrequency does.
     */
    fun tunedSuggestions(
        swipe: StatisticalGlideTypingClassifier.Gesture,
        keys: List<Key>,
        max: Int,
        frequency: (String) -> Double,
    ): List<GlideGuess> {
        if (keys.isEmpty() || swipe.isEmpty) return emptyList()
        return index.query(swipe.xs(), swipe.ys(), swipe.times(), glideKeys(keys)).use { query ->
            guesses(query.candidates(), frequency) { candidates, frequencies -> query.scoreTuned(candidates, frequencies, max) }
        }
    }

    private fun guesses(
        candidates: IntArray,
        frequency: (String) -> Double,
        score: (IntArray, FloatArray) -> GlideScores,
    ): List<GlideGuess> {
        val candidateFrequencies = FloatArray(candidates.size) { frequencyOf(candidates[it], frequency) }
        val scores = score(candidates, candidateFrequencies)
        return List(scores.words.size) { GlideGuess(words[scores.words[it]], ln(scores.weights[it])) }
    }

    private fun frequencyOf(word: Int, frequency: (String) -> Double): Float {
        val known = frequencies[word]
        if (!known.isNaN()) return known
        return frequency(words[word]).toFloat().also { frequencies[word] = it }
    }

    companion object {
        /** Indexes [words] for the layout whose keys by code are [keysByCharacter]. */
        fun build(words: List<String>, keysByCharacter: SparseArrayCompat<out Key>): SwipeIndex {
            val index = GlideIndex.build(GlideWords(words)) { describe(it, keysByCharacter) }
            return SwipeIndex(words, index)
        }

        /**
         * What [c] means on the layout, as the Kotlin classifier read it. A path visits the key for
         * its lower case, or else for that without its accent (the "e" key for "é"). A word is filed
         * under the accentless key of its first and last letters, so "Éclair" is with the words
         * from "e" to "r".
         */
        internal fun describe(c: Char, keysByCharacter: SparseArrayCompat<out Key>): GlideChar {
            val lower = Character.toLowerCase(c)
            val base = Normalizer.normalize(lower.toString(), Normalizer.Form.NFD)[0].code
            val pathKey = when {
                keysByCharacter.containsKey(lower.code) -> lower.code
                keysByCharacter.containsKey(base) -> base
                else -> GlideChar.NO_KEY
            }
            val endKey = if (keysByCharacter.containsKey(base)) base else GlideChar.NO_KEY
            return GlideChar(lower, pathKey, endKey)
        }

        /** [keys] as drawn now: rotating the screen resizes them without a new layout. */
        internal fun glideKeys(keys: List<Key>): GlideKeys {
            val codes = IntArray(keys.size)
            val bounds = FloatArray(keys.size * 4)
            for ((i, key) in keys.withIndex()) {
                val visible = key.visibleBounds
                codes[i] = (key.data as? KeyData)?.code ?: KeyCode.UNSPECIFIED
                bounds[i * 4] = visible.center.x
                bounds[i * 4 + 1] = visible.center.y
                bounds[i * 4 + 2] = visible.width
                bounds[i * 4 + 3] = visible.height
            }
            return GlideKeys(codes, bounds)
        }
    }
}
