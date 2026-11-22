/*
 * Copyright (C) 2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package dev.patrickgold.florisboard.ime.text.gestures

import android.content.Context
import androidx.collection.LruCache
import androidx.collection.SparseArrayCompat
import androidx.collection.set
import com.jtn0123.audiowhisper.suggest.GlideGuess
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.keyboard.Key
import dev.patrickgold.florisboard.nlpManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.math.max
import kotlin.math.min

private fun Key.baseCode(): Int {
    return (data as? KeyData)?.code ?: KeyCode.UNSPECIFIED
}

/**
 * Classifies gestures by comparing them with an "ideal gesture".
 *
 * Check out Étienne Desticourt's excellent write up at https://github.com/AnySoftKeyboard/AnySoftKeyboard/pull/1870
 *
 * AudioWhisper: the pruning and scoring that write-up describes are in Rust now (lib/rust, through
 * [SwipeIndex]), giving the same words at the same costs as the Kotlin did; this keeps the swipe,
 * the layout and the word list, and when to index them.
 */
class StatisticalGlideTypingClassifier(context: Context) : GlideTypingClassifier {
    private val nlpManager by context.nlpManager()

    private val gesture = Gesture()
    private var keysByCharacter: SparseArrayCompat<Key> = SparseArrayCompat()
    private var words: List<String> = emptyList()
    private var keys: ArrayList<Key> = arrayListOf()
    private lateinit var index: SwipeIndex
    private var wordDataSubtype: Subtype? = null
    private var layoutSubtype: Subtype? = null
    private var currentSubtype: Subtype? = null
    val ready: Boolean
        get() = currentSubtype == layoutSubtype && wordDataSubtype == layoutSubtype && wordDataSubtype != null
    private val indexCache = LruCache<Subtype, SwipeIndex>(INDEX_CACHE_SIZE)

    /**
     * AudioWhisper: builds a [SwipeIndex] off the main thread, as indexing the dictionary's 160,000
     * words takes a while (B18). Until it's done the previous one keeps working.
     */
    private val indexScope = CoroutineScope(Dispatchers.Default + SupervisorJob())
    private var indexBuild: Job? = null

    /**
     * The minimum distance between points to be added to a gesture.
     */
    private var distanceThresholdSquared = 0

    companion object {
        /**
         * This is a very small cache that caches suggestions, so that they aren't recalculated e.g when releasing
         * a pointer when the suggestions were already calculated. Avoids a lot of micro pauses.
         */
        private const val SUGGESTION_CACHE_SIZE = 5

        /**
         * For multiple subtypes, the index is cached.
         */
        private const val INDEX_CACHE_SIZE = 5

        /** A [GlideTypingGesture.Detector.Position]'s time is in microseconds; a [Gesture]'s in ms. */
        private const val MICROS_PER_MS = 1000f
    }

    override fun addGesturePoint(position: GlideTypingGesture.Detector.Position) {
        if (!gesture.isEmpty) {
            val dx = gesture.getLastX() - position.x
            val dy = gesture.getLastY() - position.y

            if (dx * dx + dy * dy > distanceThresholdSquared) {
                gesture.addPoint(position.x, position.y, position.time / MICROS_PER_MS)
            }
        } else {
            gesture.addPoint(position.x, position.y, position.time / MICROS_PER_MS)
        }
    }

    override fun setLayout(keyViews: List<Key>, subtype: Subtype) {
        // AudioWhisper: a change of words for the same subtype arrives through setWordData (B18).
        if (wordDataSubtype != subtype) setWordData(subtype)
        // stop duplicate calls
        if (layoutSubtype == subtype && keys == keyViews) {
            return
        }

        // if only layout changed but not subtype
        val layoutChanged = layoutSubtype == subtype

        keysByCharacter.clear()
        keys.clear()
        keyViews.forEach {
            keysByCharacter[it.baseCode()] = it
            keys.add(it)
        }
        layoutSubtype = subtype
        distanceThresholdSquared = (keyViews.first().visibleBounds.width / 4).toInt()
        distanceThresholdSquared *= distanceThresholdSquared

        if (
            (wordDataSubtype == layoutSubtype)
            || layoutChanged // should force a re-initialize
        ) {
            initializeIndex(layoutChanged)
        }
    }

    override fun setWordData(subtype: Subtype) {
        // AudioWhisper: called again whenever the word list changes (B18); the provider returns
        // the same list until then, so the same list needs no new index.
        val words = nlpManager.getListOfWords(subtype)
        if (wordDataSubtype == subtype && words === this.words) {
            return
        }

        this.words = words

        this.wordDataSubtype = subtype
        if (wordDataSubtype == layoutSubtype) {
            initializeIndex(false)
        }
    }

    /**
     * Exists because the index requires both word data and layout are initialized,
     * however we don't know what order they're initialized in.
     */
    private fun initializeIndex(invalidateCache: Boolean) {
        val currentSubtype = this.layoutSubtype!!
        val words = this.words
        val cached = when {
            invalidateCache -> null
            else -> indexCache.get(currentSubtype)?.takeIf { it.words === words }
        }
        indexBuild?.cancel()
        if (cached == null) {
            buildIndex(currentSubtype, words)
        } else {
            this.index = cached
            this.currentSubtype = currentSubtype
        }
    }

    /** AudioWhisper: see [indexScope]. Only the latest build is used; any older one is dropped. */
    private fun buildIndex(subtype: Subtype, words: List<String>) {
        val keys = keysByCharacter.clone()
        indexBuild = indexScope.launch {
            val built = SwipeIndex.build(words, keys)
            withContext(Dispatchers.Main) {
                if (layoutSubtype != subtype || this@StatisticalGlideTypingClassifier.words !== words) return@withContext
                index = built
                indexCache.put(subtype, built)
                currentSubtype = subtype
            }
        }
    }

    override fun initGestureFromPointerData(pointerData: GlideTypingGesture.Detector.PointerData) {
        for (position in pointerData.positions) {
            addGesturePoint(position)
        }
    }

    private val lruSuggestionCache = LruCache<Pair<Gesture, Int>, List<GlideGuess>>(SUGGESTION_CACHE_SIZE)
    override fun getSuggestions(maxSuggestionCount: Int, gestureCompleted: Boolean): List<String> {
        return getScoredSuggestions(snapshot(), maxSuggestionCount).map { it.word }
    }

    /**
     * AudioWhisper: a copy of the gesture so far, to score off the main thread. Scoring the live
     * gesture while the main thread added points gave results for part of the swipe, cached under
     * the whole of it, so lifting the finger could commit a word for less than was drawn.
     */
    fun snapshot(): Gesture = gesture.clone()

    /**
     * AudioWhisper: the suggestions for [swipe], a [snapshot] nothing changes any more, with their
     * costs, so the words before them can re-rank them.
     */
    fun getScoredSuggestions(swipe: Gesture, maxSuggestionCount: Int): List<GlideGuess> {
        val key = Pair(swipe, maxSuggestionCount)
        return lruSuggestionCache.get(key)
            ?: unCachedGetSuggestions(swipe, maxSuggestionCount).also { lruSuggestionCache.put(key, it) }
    }

    private fun unCachedGetSuggestions(swipe: Gesture, maxSuggestionCount: Int): List<GlideGuess> {
        val subtype = currentSubtype ?: return emptyList()
        return index.tunedSuggestions(swipe, keys, maxSuggestionCount) { nlpManager.getFrequencyForWord(subtype, it) }
    }

    override fun clear() {
        gesture.clear()
    }

    /**
     * A gesture's points. AudioWhisper: the storage grows as points are added, up to [MAX_SIZE],
     * instead of every gesture taking 500 points of memory; and each point keeps when it was drawn,
     * which the scorer weighs (see [SwipeIndex.tunedSuggestions]).
     */
    class Gesture private constructor(
        private var xs: FloatArray,
        private var ys: FloatArray,
        private var ts: FloatArray,
        private var size: Int,
    ) {
        constructor(capacity: Int = MAX_SIZE) : this(FloatArray(capacity), FloatArray(capacity), FloatArray(capacity), 0)

        companion object {
            // TODO: Find out optimal max size
            private const val MAX_SIZE = 500
        }

        val isEmpty: Boolean
            get() = size == 0

        /** Adds a point at ([x], [y]), drawn at [time] in milliseconds (0 when unknown). */
        fun addPoint(x: Float, y: Float, time: Float = 0f) {
            if (size >= MAX_SIZE) {
                return
            }
            if (size == xs.size) {
                val capacity = min(MAX_SIZE, max(xs.size * 2, 8))
                xs = xs.copyOf(capacity)
                ys = ys.copyOf(capacity)
                ts = ts.copyOf(capacity)
            }
            xs[size] = x
            ys[size] = y
            ts[size] = time
            size += 1
        }

        /** The points' x coordinates, in order. */
        fun xs(): FloatArray = xs.copyOf(size)

        /** The points' y coordinates, in order. */
        fun ys(): FloatArray = ys.copyOf(size)

        /** When each point was drawn, in milliseconds, in order. */
        fun times(): FloatArray = ts.copyOf(size)

        fun getLastX(): Float = xs.getOrElse(size - 1) { 0f }
        fun getLastY(): Float = ys.getOrElse(size - 1) { 0f }

        fun clear() {
            this.size = 0
        }

        fun clone(): Gesture {
            return Gesture(xs.copyOf(size), ys.copyOf(size), ts.copyOf(size), size)
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as Gesture

            if (this.size != other.size) return false

            for (i in 0 until size) {
                if (xs[i] != other.xs[i] || ys[i] != other.ys[i] || ts[i] != other.ts[i]) return false
            }

            return true
        }

        // AudioWhisper: only the points in the gesture, as equals compares. Hashing the whole
        // storage gave equal gestures different hashes whenever old points were left past the end.
        override fun hashCode(): Int {
            var result = size
            for (i in 0 until size) {
                result = 31 * result + xs[i].hashCode()
                result = 31 * result + ys[i].hashCode()
                result = 31 * result + ts[i].hashCode()
            }
            return result
        }
    }
}
