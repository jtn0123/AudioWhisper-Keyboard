package com.jtn0123.audiowhisper.rust

import kotlin.math.roundToInt
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Test-only InputEventDispatcher and QuickAction.InsertKey reference, before stage 3B6. */
private fun referenceEditing(codes: IntArray, hold: Long, repeat: Long, words: Boolean): IntArray = codes.flatMap { code ->
    val repeatable = code in listOf(-24, -21, -22, -23, -7, -9, -131, -132)
    val data = if (words) when (code) { -7 -> -8; -9 -> -10; else -> code } else code
    val holdFactor = when (code) { 32, 12288, -11 -> 2.5f; -227 -> 2f; else -> 1f }
    val repeatFactor = if (data in listOf(-8, -10, -131, -132)) 5f else 1f
    listOf(if (repeatable) 1 else 0, data, (hold * holdFactor).toInt(), (repeat * repeatFactor).toInt(),
        if (!repeatable && code != -242 && code != -35) 1 else 0)
}.toIntArray()

/** Compose GridCells.Adaptive's size and remainder distribution, without spacing. */
private fun referenceCells(width: Float, minimum: Float): FloatArray {
    val available = width.roundToInt()
    val count = maxOf(available / minimum.roundToInt(), 1)
    val size = available / count
    val remaining = available % count
    var x = 0
    return (0 until count).flatMap { i ->
        val w = size + if (i < remaining) 1 else 0
        listOf(x.toFloat(), w.toFloat()).also { x += w }
    }.toFloatArray()
}

/** EmojiSet.base and variations preserve the first matching tone and exclude all matches. */
private fun referenceChoices(tones: IntArray, preferred: Int): IntArray {
    val base = if (tones.size == 1) 0 else tones.indexOfFirst { it == preferred }.takeIf { it >= 0 } ?: 0
    return (listOf(base) + if (tones.size <= 1) emptyList() else tones.indices.filter { tones[it] != preferred }).toIntArray()
}

class PanelPresentationMatchTest {
    @Test fun editingHoldRepeatDeletionAndOverflowPolicy() {
        for (delay in listOf(1L, 201L, 300L, 501L, 1000L)) for (repeat in listOf(1L, 50L, 100L)) for (words in listOf(false, true)) {
            val codes = IntArray(10001) { it - 9999 }
            assertContentEquals(referenceEditing(codes, delay, repeat, words), PanelRules.editing(codes, delay, repeat, if (words) 1 else 0))
        }
    }
    @Test fun adaptiveEmojiOverflowAndEditorCells() {
        for (density in listOf(1f, 1.5f, 2f, 2.625f, 3f, 3.5f)) for (width in listOf(320f, 411f, 720f, 1080f, 1920f, 2400f)) {
            for (minimum in listOf(42f * density, 48f * density * 1.8f, 48f * density * 2.2f)) {
                assertContentEquals(referenceCells(width, minimum), PanelRules.cells(floatArrayOf(width, minimum)))
            }
        }
    }
    @Test fun preferredToneAndVariants() {
        val random = Random(0x3b06)
        for (tone in 0..5) repeat(2000) {
            val tones = IntArray(random.nextInt(1, 14)) { random.nextInt(6) }
            assertContentEquals(referenceChoices(tones, tone), PanelRules.emojiChoices(tones, tone))
        }
    }
}
