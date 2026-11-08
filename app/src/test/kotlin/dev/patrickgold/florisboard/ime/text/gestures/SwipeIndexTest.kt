package dev.patrickgold.florisboard.ime.text.gestures

import androidx.collection.SparseArrayCompat
import com.jtn0123.audiowhisper.suggest.GlideGuess
import com.jtn0123.audiowhisper.suggest.SwipeWords
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKey
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.lib.FlorisRect
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.int
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.jupiter.api.TestInstance
import java.io.File
import java.text.Normalizer
import kotlin.math.ceil
import kotlin.math.hypot
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Holds the Rust scorer behind [SwipeIndex] to the Kotlin one it replaced ([ReferenceGlideClassifier]):
 * the same words, in the same order, at the same costs, for the dictionary's words and the odd
 * ones (capitals, accents, apostrophes, doubled letters), swiped cleanly, sloppily and strangely
 * on two keyboards.
 *
 * Two things differ on purpose, so the swipes here avoid them. The Kotlin scorer broke ties
 * between equally near keys by hash order, which changed from run to run, and Rust by layout
 * order; so no swipe starts or ends equally near two keys. And an empty swipe, which Kotlin
 * scored from stale storage, gives nothing ([anEmptySwipeGivesNothing]).
 */
// One instance for every test: indexing the dictionary twice over takes a while.
@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class SwipeIndexTest {
    private val probabilities: Map<String, Int> = run {
        val json = Json.parseToJsonElement(File("src/main/assets/ime/dict/data.json").readText()).jsonObject
        json.mapValues { it.value.jsonPrimitive.int }
    }

    /** Words the dictionary lacks that exercise what the scorer reads from each character. */
    private val oddWords = listOf(
        "I", "a", "A", "Éclair", "éclair", "café", "naïve", "über", "Über", "jalapeño", "façade",
        "İstanbul", "straße", "ﬁsh", "don't", "rock'n'roll", "students'", "'tis", "well-known",
        "co-op", "x-ray", "aa", "aaa", "Aa", "llama", "balloon", "bookkeeper", "Mississippi",
        "hello", "Hello", "HELLO", "MP3", "4ever", "zz", "q", "qq", "pool", "poll", "lull", "lul",
    )

    /** The dictionary's swipeable words, then the odd ones, some of them again. */
    private val words: List<String> =
        probabilities.keys.filter(SwipeWords::isSwipeable) + oddWords + listOf("hello", "café", "aa")

    private val oddProbabilities = oddWords.withIndex().associate { (i, word) -> word to (i * 37) % 256 }

    /** As SwipeWords gives it: 0 for "Über", which no probability is given. */
    private fun frequency(word: String): Double {
        val probability = probabilities[word] ?: oddProbabilities[word]?.takeIf { word != "Über" }
        return probability?.let(SwipeWords::frequencyOf) ?: 0.0
    }

    /** A keyboard the size of a phone's, held one way, and the classifier's index for it. */
    private inner class Board(keyWidth: Float, keyHeight: Float, withApostrophe: Boolean) {
        val keys: List<TextKey> = qwerty(keyWidth, keyHeight, withApostrophe)
        val keysByCharacter = SparseArrayCompat<TextKey>().apply { keys.forEach { put(it.code, it) } }
        val reference = ReferenceGlideClassifier(words, keysByCharacter)
        val index = SwipeIndex.build(words, keysByCharacter)
        private val centres = keys.map { it.visibleBounds.center }

        fun reference(points: List<Pair<Float, Float>>, max: Int): List<GlideGuess> {
            val swipe = ReferenceGlideClassifier.Gesture()
            points.forEach { (x, y) -> swipe.addPoint(x, y) }
            return reference.suggestions(swipe, keys, keysByCharacter, max, ::frequency)
        }

        fun rust(points: List<Pair<Float, Float>>, max: Int): List<GlideGuess> {
            val swipe = StatisticalGlideTypingClassifier.Gesture()
            points.forEach { (x, y) -> swipe.addPoint(x, y) }
            return index.suggestions(swipe, keys, max, ::frequency)
        }

        /** The centre of the key [c] is typed on, as glide typing finds it. */
        fun centre(c: Char): Pair<Float, Float>? {
            val lower = Character.toLowerCase(c)
            val base = Normalizer.normalize(lower.toString(), Normalizer.Form.NFD)[0]
            val key = keysByCharacter[lower.code] ?: keysByCharacter[base.code] ?: return null
            return key.visibleBounds.center.let { it.x to it.y }
        }

        /** Whether two of the three keys nearest (x, y) are equally near: see the class doc. */
        fun tied(x: Float, y: Float): Boolean {
            val nearest = centres.map { ReferenceGlideClassifier.Gesture.distance(it.x, it.y, x, y) }
                .sorted().take(3)
            return nearest.zipWithNext().any { (a, b) -> a == b }
        }

        /** [points] with the first and last moved, if need be, off any tie between keys. */
        fun untied(points: List<Pair<Float, Float>>): List<Pair<Float, Float>> {
            fun nudge(point: Pair<Float, Float>): Pair<Float, Float> {
                var (x, y) = point
                while (tied(x, y)) {
                    x += 0.37f
                    y += 0.23f
                }
                return x to y
            }
            if (points.isEmpty()) return points
            return listOf(nudge(points.first())) + points.subList(1, points.size).let { rest ->
                if (rest.isEmpty()) rest else rest.dropLast(1) + nudge(rest.last())
            }
        }
    }

    private val portrait = Board(keyWidth = 108f, keyHeight = 150f, withApostrophe = false)
    private val landscape = Board(keyWidth = 234f, keyHeight = 100f, withApostrophe = true)

    /**
     * A swipe through [word]'s keys on [board]: a point every [step] px, each up to [wobble] px
     * off, the whole swipe [missBy] px off to one side.
     */
    private fun swipe(
        board: Board,
        word: String,
        random: Random,
        step: Float = 18f,
        wobble: Float = 0f,
        missBy: Float = 0f,
    ): List<Pair<Float, Float>> {
        val centres = word.mapNotNull(board::centre)
        if (centres.isEmpty()) return emptyList()
        val (offsetX, offsetY) = random.nextFloat() * 2 * missBy - missBy to random.nextFloat() * 2 * missBy - missBy
        fun jitter(x: Float, y: Float) = (x + offsetX + (random.nextFloat() * 2 - 1) * wobble) to
            (y + offsetY + (random.nextFloat() * 2 - 1) * wobble)
        val points = mutableListOf(jitter(centres[0].first, centres[0].second))
        for ((from, to) in centres.zipWithNext()) {
            val steps = ceil(hypot(to.first - from.first, to.second - from.second) / step).toInt().coerceAtLeast(1)
            for (s in 1..steps) {
                val x = from.first + (to.first - from.first) * s / steps
                val y = from.second + (to.second - from.second) * s / steps
                points += jitter(x, y)
            }
        }
        return board.untied(points)
    }

    private fun assertSame(board: Board, points: List<Pair<Float, Float>>, max: Int, what: String) {
        val expected = board.reference(points, max)
        assertEquals(expected, board.rust(points, max), what)
    }

    @Test
    fun theDictionarysWordsRankAsBefore() {
        val random = Random(20261108)
        val common = probabilities.keys.filter(SwipeWords::isSwipeable).take(400)
        var nonEmpty = 0
        for ((name, board) in listOf("portrait" to portrait, "landscape" to landscape)) {
            for (i in 0 until 150) {
                val word = common[random.nextInt(common.size)]
                val points = swipe(board, word, random, wobble = 6f + i % 5 * 6f, missBy = i % 4 * 12f)
                val max = listOf(1, 3, 8, 20)[i % 4]
                if (board.reference(points, max).isNotEmpty()) nonEmpty++
                assertSame(board, points, max, "$name: swipe $i for \"$word\", top $max")
            }
        }
        assertTrue(nonEmpty > 280, "only $nonEmpty of 300 swipes found any words")
    }

    @Test
    fun oddWordsRankAsBefore() {
        val random = Random(8)
        for ((name, board) in listOf("portrait" to portrait, "landscape" to landscape)) {
            for (word in oddWords) {
                for (wobble in listOf(0f, 15f)) {
                    val points = swipe(board, word, random, wobble = wobble)
                    if (points.isEmpty()) continue
                    assertSame(board, points, 8, "$name: \"$word\" wobbling $wobble px")
                }
            }
        }
    }

    @Test
    fun strangeSwipesRankAsBefore() {
        val random = Random(500)
        val zigzag = List(700) { i -> (40f + i * 13f % 1000f) to (if (i % 2 == 0) 80f else 400f) }
        val swipes = mapOf(
            "one point" to listOf(310f to 215f),
            "the same point twice" to listOf(310f to 215f, 310f to 215f),
            "the same point ten times" to List(10) { 520f to 230f },
            "two points" to listOf(60f to 70f, 900f to 300f),
            "700 points, past the 500 kept" to zigzag,
            "off the keyboard" to listOf(-500f to -400f, -300f to -900f, -100f to -50f),
            "far away" to listOf(1e6f to 1e6f, 2e6f to 1e6f),
            "across the whole keyboard" to List(60) { i -> (i * 18f) to 300f },
            "back on itself" to listOf(100f to 100f, 500f to 100f, 100f to 100f, 500f to 100f),
            "slow, with points 1 px apart" to List(300) { i -> (200f + i) to (230f + i / 3f) },
        )
        for ((name, board) in listOf("portrait" to portrait, "landscape" to landscape)) {
            for ((what, points) in swipes) {
                for (max in listOf(1, 8, 30)) {
                    assertSame(board, board.untied(points), max, "$name: $what, top $max")
                }
            }
            val sloppy = swipe(board, "keyboard", random, step = 40f, wobble = 60f, missBy = 50f)
            assertSame(board, sloppy, 8, "$name: a very sloppy \"keyboard\"")
        }
    }

    @Test
    fun anEmptySwipeGivesNothing() {
        assertEquals(emptyList(), portrait.rust(emptyList(), 8))
    }

    @Test
    fun noKeysGiveNothing() {
        val swipe = StatisticalGlideTypingClassifier.Gesture()
        swipe(portrait, "hello", Random(1)).forEach { (x, y) -> swipe.addPoint(x, y) }

        assertEquals(emptyList(), portrait.index.suggestions(swipe, emptyList(), 8, ::frequency))
    }

    private companion object {
        private val TextKey.code: Int
            get() = (data as? KeyData)?.code ?: KeyCode.UNSPECIFIED

        /**
         * QWERTY as the keyboard lays it out: three rows of letters, shift and delete either side
         * of the last, then symbols, comma, space, (an apostrophe), period and enter. Each key is
         * drawn 3 px inside its cell, and two gaps have keys with no code.
         */
        fun qwerty(width: Float, height: Float, withApostrophe: Boolean): List<TextKey> {
            val keys = mutableListOf<TextKey>()
            fun add(code: Int, left: Float, row: Int, cells: Float = 1f) {
                val key = TextKey(TextKeyData(code = code))
                key.visibleBounds.applyFrom(
                    FlorisRect.new(left + 3f, row * height + 3f, left + cells * width - 3f, (row + 1) * height - 3f),
                )
                keys += key
            }
            "qwertyuiop".forEachIndexed { i, c -> add(c.code, i * width, 0) }
            add(KeyCode.UNSPECIFIED, 0f, 1, 0.5f)
            "asdfghjkl".forEachIndexed { i, c -> add(c.code, (i + 0.5f) * width, 1) }
            add(KeyCode.UNSPECIFIED, 9.5f * width, 1, 0.5f)
            add(KeyCode.SHIFT, 0f, 2, 1.5f)
            "zxcvbnm".forEachIndexed { i, c -> add(c.code, (i + 1.5f) * width, 2) }
            add(KeyCode.DELETE, 8.5f * width, 2, 1.5f)
            val bottom = buildList {
                add(KeyCode.VIEW_SYMBOLS to 1.5f)
                add(','.code to 1f)
                add(KeyCode.SPACE to if (withApostrophe) 4f else 5f)
                if (withApostrophe) add('\''.code to 1f)
                add('.'.code to 1f)
                add(KeyCode.ENTER to 1.5f)
            }
            var left = 0f
            for ((code, cells) in bottom) {
                add(code, left, 3, cells)
                left += cells * width
            }
            return keys
        }
    }
}
