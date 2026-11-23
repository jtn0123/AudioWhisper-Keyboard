package dev.patrickgold.florisboard.ime.text.keyboard

import com.jtn0123.audiowhisper.golden.GoldenTape
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.lib.FlorisRect
import kotlin.random.Random

/**
 * Records the text keyboard's geometry in Rust ([TextKeyboard], [TextKey.applyFlex]) on [tape], to
 * be checked against the answers the Kotlin it replaced (`ReferenceTextKeyboard`) gave
 * (docs/phase6-kotlin-references.md). Floats go on it as their bits, any NaN as Java's one NaN. It
 * counts how often each outcome was seen, so a caller can tell the runs reached them all.
 */
class TextKeyboardComparison(private val tape: GoldenTape) {
    val seen = mutableMapOf<String, Int>()

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    private fun bits(rect: FlorisRect) = listOf(rect.left, rect.top, rect.right, rect.bottom).map(java.lang.Float::floatToIntBits)

    private fun factors(mode: KeyboardMode, code: Int, type: KeyType): Triple<Float, Float, Float> {
        val key = TextKey(TextKeyData.UNSPECIFIED)
        key.applyFlex(mode, code, type)
        return Triple(key.flayShrink, key.flayGrow, key.flayWidthFactor)
    }

    /** Every mode with key codes near each the table names, and others, of every type. */
    fun flex(random: Random, runs: Int) = repeat(runs) {
        val mode = if (random.nextInt(4) == 0) KeyboardMode.NUMERIC_ADVANCED else KeyboardMode.entries.random(random)
        val code = when (random.nextInt(4)) {
            0 -> random.nextInt()
            1 -> random.nextInt(-300, 13_000)
            else -> CODES.random(random) + random.nextInt(-1, 2) * random.nextInt(2)
        }
        val type = if (random.nextInt(4) == 0) KeyType.NUMERIC else KeyType.entries.random(random)
        val rust = factors(mode, code, type)
        tape.answer("flex", rust.toList().map(java.lang.Float::floatToIntBits))
        see("flex ${rust.first} ${rust.second} ${rust.third}")
    }

    /** The desired key for every mode, at sizes from realistic to zero, huge and NaN. */
    fun desiredKeys(random: Random, runs: Int) = repeat(runs) {
        val mode = KeyboardMode.entries.random(random)
        val rowCount = random.nextInt(0, 9)
        val realistic = random.nextInt(3) != 0
        val width = if (realistic) random.nextFloat() * 2000f else odd(random)
        val height = if (realistic) random.nextFloat() * 1200f else odd(random)
        val base = if (realistic) 20f + random.nextFloat() * 200f else odd(random)
        val marginH = if (realistic) random.nextFloat() * 12f else odd(random)
        val marginV = if (realistic) random.nextFloat() * 12f else odd(random)
        val keyboard = TextKeyboard(Array(rowCount) { emptyArray() }, mode, null, null)
        val rust = TextKey(TextKeyData.UNSPECIFIED)
        keyboard.sizeDesiredKey(rust, width, height, base, marginH, marginV)
        tape.answer("desired", bits(rust.touchBounds) + bits(rust.visibleBounds))
        val rowHeight = height / rowCount
        see(
            when {
                mode !in CHARACTER_MODES -> "desired: base row"
                rowHeight > base * 1.12f -> "desired: capped"
                else -> "desired: a row of the height"
            },
        )
    }

    /**
     * Random keyboards laid out a few times each at random sizes and desired keys, recording every
     * key's bounds after each layout, then random touches near and on the keys' edges.
     */
    fun keyboards(random: Random, runs: Int, touches: Int) = repeat(runs) {
        val sizes = IntArray(if (random.nextInt(30) == 0) 0 else random.nextInt(1, 8)) {
            if (random.nextInt(12) == 0) 0 else random.nextInt(1, 16)
        }
        val mode = KeyboardMode.entries.random(random)
        val rustKeys = Array(sizes.size) { r -> Array(sizes[r]) { TextKey(TextKeyData.UNSPECIFIED) } }
        val rust = TextKeyboard(rustKeys, mode, null, null)
        repeat(random.nextInt(1, 4)) { layout ->
            // A whole number of desired keys wide, with whole factors: rows that fit exactly.
            val fit = random.nextInt(5) == 0
            for (r in sizes.indices) for (k in 0 until sizes[r]) {
                val (shrink, grow, width) = if (fit) {
                    Triple(FIT.random(random), FIT.random(random), FIT.random(random))
                } else {
                    flexOf(random, mode)
                }
                rustKeys[r][k].apply {
                    flayShrink = shrink
                    flayGrow = grow
                    flayWidthFactor = width
                }
            }
            layOut(random, rust, sizes, fit)
            repeat(touches) { touch(random, rust) }
        }
    }

    private fun layOut(random: Random, rust: TextKeyboard, sizes: IntArray, fit: Boolean) {
        val realistic = random.nextInt(4) != 0
        val keyWidth = FIT_WIDTHS.random(random)
        val width = when {
            fit -> keyWidth * random.nextInt(0, 16)
            realistic -> 100f + random.nextFloat() * 2000f
            else -> odd(random)
        }
        val height = if (realistic || fit) 50f + random.nextFloat() * 1000f else odd(random)
        val desired = if (fit) {
            TextKey(TextKeyData.UNSPECIFIED).also { key ->
                key.touchBounds.apply { right = keyWidth; bottom = 60f }
                key.visibleBounds.apply { top = 3f; right = keyWidth; bottom = 57f }
            }
        } else if (random.nextInt(6) != 0) {
            // Drawn in the order the reference's arguments were.
            val desiredWidth = width * (0.9f + random.nextFloat() * 0.2f)
            val base = if (realistic) 20f + random.nextFloat() * 200f else odd(random)
            val marginH = if (random.nextInt(8) == 0) odd(random) else random.nextFloat() * 12f
            val marginV = if (random.nextInt(8) == 0) odd(random) else random.nextFloat() * 12f
            TextKey(TextKeyData.UNSPECIFIED).also { rust.sizeDesiredKey(it, desiredWidth, height, base, marginH, marginV) }
        } else {
            TextKey(TextKeyData.UNSPECIFIED).also { key ->
                for (rect in listOf(key.touchBounds, key.visibleBounds)) {
                    rect.left = edge(random)
                    rect.top = edge(random)
                    rect.right = rect.left + edge(random)
                    rect.bottom = rect.top + edge(random)
                }
            }
        }
        val extend = random.nextBoolean()
        val skipped = sizes.isEmpty() || desired.touchBounds.isEmpty() || desired.visibleBounds.isEmpty() ||
            width.isNaN() || height.isNaN()
        rust.layout(width, height, desired, extend)
        see(if (skipped) "layout: skipped" else "layout: laid out")
        if (!skipped) {
            for (row in rust.arrangement) see(rowOutcome(row, width, desired))
        }
        for (r in sizes.indices) for (k in 0 until sizes[r]) {
            val key = rust.arrangement[r][k]
            tape.answer("layout", bits(key.touchBounds) + bits(key.visibleBounds))
        }
    }

    private fun rowOutcome(row: Array<TextKey>, width: Float, desired: TextKey): String {
        val touch = desired.touchBounds
        val marginH = kotlin.math.abs(touch.width - desired.visibleBounds.width)
        val available = (width - marginH) / touch.width
        val requested = row.fold(0f) { sum, key -> sum + key.flayWidthFactor }
        val growSum = row.fold(0f) { sum, key -> sum + key.flayGrow }
        return when {
            row.isEmpty() -> "row: empty"
            requested == available -> "row: fits exactly"
            requested <= available && growSum == 0f -> "row: grows at the ends"
            requested <= available -> "row: grows"
            else -> "row: shrinks"
        }
    }

    private fun touch(random: Random, rust: TextKeyboard) {
        val all = rust.arrangement.flatten()
        val (x, y) = if (all.isNotEmpty() && random.nextInt(4) != 0) {
            val bounds = all.random(random).touchBounds
            near(random, bounds.left, bounds.right) to near(random, bounds.top, bounds.bottom)
        } else {
            val last = all.lastOrNull()?.touchBounds
            val w = last?.right ?: 1000f
            val h = last?.bottom ?: 500f
            when (random.nextInt(8)) {
                0 -> odd(random) to odd(random)
                else -> (random.nextFloat() * 1.2f - 0.1f) * w to (random.nextFloat() * 1.2f - 0.1f) * h
            }
        }
        val actual = rust.getKeyForPos(x, y)
        val index = actual?.let { key -> all.indexOfFirst { it === key } } ?: -1
        tape.answer("touch", index)
        val inSome = all.any { it.touchBounds.contains(x, y) }
        see(
            when {
                index >= 0 -> "touch: a key"
                inSome -> "touch: a key past a row without keys"
                else -> "touch: no key"
            },
        )
    }

    /** A coordinate on, just either side of, or inside the edges [low] and [high]. */
    private fun near(random: Random, low: Float, high: Float): Float = when (random.nextInt(8)) {
        0 -> low
        1 -> high
        2 -> Math.nextDown(low)
        3 -> Math.nextUp(low)
        4 -> Math.nextDown(high)
        5 -> Math.nextUp(high)
        else -> low + (high - low) * random.nextFloat()
    }

    /** A key's flex factors: the table's for a random key mostly, else odd ones. */
    private fun flexOf(random: Random, mode: KeyboardMode): Triple<Float, Float, Float> = when (random.nextInt(10)) {
        in 0..5 -> factors(mode, CODES.random(random), KeyType.entries.random(random))
        in 6..7 -> Triple(NICE.random(random), NICE.random(random), NICE.random(random))
        else -> Triple(edge(random), edge(random), edge(random))
    }

    /** A Float anywhere: the table's, tiny, zero, negative, huge, infinite or NaN. */
    private fun edge(random: Random): Float = when (random.nextInt(4)) {
        0 -> NICE.random(random)
        1 -> ODD.random(random)
        2 -> random.nextFloat() * 4f - 1f
        else -> java.lang.Float.intBitsToFloat(random.nextInt())
    }

    /** A size that's odd: zero, negative, fractional, tiny, huge or NaN. */
    private fun odd(random: Random): Float = when (random.nextInt(3)) {
        0 -> ODD.random(random)
        1 -> (random.nextFloat() - 0.2f) * 3000f
        else -> random.nextFloat() * 10f
    }

    private companion object {
        val CHARACTER_MODES = setOf(
            KeyboardMode.CHARACTERS, KeyboardMode.NUMERIC_ADVANCED, KeyboardMode.SYMBOLS, KeyboardMode.SYMBOLS2,
        )
        val CODES = listOf(
            KeyCode.SHIFT, KeyCode.DELETE, KeyCode.VIEW_CHARACTERS, KeyCode.VIEW_SYMBOLS, KeyCode.VIEW_SYMBOLS2,
            KeyCode.ENTER, KeyCode.SPACE, KeyCode.CJK_SPACE, 44, 46, 61, 48, 97, 0, KeyCode.UNSPECIFIED,
        )
        val FIT = listOf(0f, -0f, 1f, 2f, -1f, 0.5f, 1.5f)
        val FIT_WIDTHS = listOf(100f, 50f, 108f, 37.5f, 0.75f)
        val NICE = listOf(0f, -0f, 1f, 1.5f, 1.56f, 1.26f, 2.68f, 0.5f, 2f, 3f, -1f)
        val ODD = listOf(
            0f, -0f, Float.MIN_VALUE, -Float.MIN_VALUE, 1e-30f, 1e30f, Float.MAX_VALUE, -Float.MAX_VALUE,
            Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY, Float.NaN, 1f / 3f, 0.1f, 1080.5f, -1f,
        )
    }
}
