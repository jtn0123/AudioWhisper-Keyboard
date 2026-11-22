package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyGraph
import com.jtn0123.audiowhisper.rust.KeyGraphData
import com.jtn0123.audiowhisper.rust.KeyGraphHints
import com.jtn0123.audiowhisper.rust.KeyGraphInfo
import com.jtn0123.audiowhisper.rust.KeyGraphKey
import com.jtn0123.audiowhisper.rust.KeyGraphOp
import com.jtn0123.audiowhisper.rust.KeyGraphQuestion
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.ime.keyboard.AbstractKeyData
import dev.patrickgold.florisboard.ime.keyboard.Key
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.keyboard.Keyboard
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.popup.PopupKeys
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.keyboard.MultiTextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData

/**
 * A keyboard whose keys Rust builds and computes ([KeyGraph]). Kotlin keeps one [RustKey] for each
 * listed key, the same object across computations, and reads back what Rust decided.
 */
class RustKeyboard(val graph: KeyGraph) : Keyboard() {
    private val info = graph.call(KeyGraphOp.INFO)
    override val mode = KeyboardMode.fromInt(info[KeyGraphInfo.MODE].toInt())
    override val rowCount = info[KeyGraphInfo.ROWS].toInt()
    private val popupSets = HashMap<Int, PopupSet<AbstractKeyData>>()
    private val multis = HashMap<Int, MultiTextKeyData>()
    val keys: List<RustKey> = graph.call(KeyGraphOp.KEYS).let { rows ->
        List(info[KeyGraphInfo.LISTED].toInt()) { RustKey(rows[it * KeyGraphKey.SIZE + KeyGraphKey.BASE].toInt()) }
    }

    override fun keys(): Iterator<Key> = keys.iterator()

    /** The renderer finds keys in Rust; nothing asks this keyboard. */
    override fun getKeyForPos(pointerX: Float, pointerY: Float): Key? = null

    /** The renderer lays the keys out with [layout] of its own sizes. */
    override fun layout(keyboardWidth: Float, keyboardHeight: Float, desiredKey: Key, extendTouchBoundariesDownwards: Boolean) =
        Unit

    /**
     * Computes the keys for [facts] (a `KeyGraphFacts` row and the pairs after it), with [answer]
     * answering each question Rust asks (its `KeyGraphAsk`, `KeyGraphLocale` and value).
     */
    fun compute(facts: Array<String>, answer: (Int, Int, String) -> String) {
        var questions = graph.call(KeyGraphOp.COMPUTE, *facts)
        while (questions.isNotEmpty()) {
            graph.call(KeyGraphOp.ANSWER, *Array(questions.size / KeyGraphQuestion.SIZE) {
                val at = it * KeyGraphQuestion.SIZE
                answer(questions[at + KeyGraphQuestion.ASK].toInt(), questions[at + KeyGraphQuestion.LOCALE].toInt(),
                    questions[at + KeyGraphQuestion.VALUE])
            })
            questions = graph.call(KeyGraphOp.COMPUTE, *facts)
        }
        val rows = graph.call(KeyGraphOp.KEYS)
        for ((i, key) in keys.withIndex()) key.update(rows, i * KeyGraphKey.SIZE, ::data)
    }

    /**
     * Lays the keys out on a keyboard [width] by [height] with rows [rowHeight] tall and key
     * margins [marginH] and [marginV]: the desired key's visible size, then each key's touch and
     * visible bounds, enabled state and code, ten floats a key, which the keys keep too.
     */
    fun layout(width: Float, height: Float, rowHeight: Float, marginH: Float, marginV: Float): FloatArray =
        graph.layout(floatArrayOf(width, height, rowHeight, marginH, marginV)).also { numbers ->
            for ((i, key) in keys.withIndex()) key.place(numbers, 2 + i * 10)
        }

    /** Key [index]'s popup keys for the hint settings [config]. */
    fun popupKeys(index: Int, config: KeyHintConfiguration): PopupKeys<KeyData> {
        val row = graph.call(KeyGraphOp.POPUPS, "$index", *hints(config))
        val all = List((row.size - 1) / KeyGraphData.SIZE) { data(row, 1 + it * KeyGraphData.SIZE) }
        val prioritized = row[0].toInt()
        return PopupKeys(null, all.take(prioritized), all.drop(prioritized))
    }

    /** Whether each key has popup keys for the hint settings [config]. */
    fun popupFlags(config: KeyHintConfiguration): IntArray =
        graph.call(KeyGraphOp.POPUP_FLAGS, *hints(config)).map { if (WireFlag.decode(it)) 1 else 0 }.toIntArray()

    /**
     * The key data a `KeyGraphData` row at [at] of [row] describes: the same multi-text key, and
     * the same popup set, for the same id, as Kotlin's graph held one object each.
     */
    private fun data(row: Array<String>, at: Int): KeyData {
        val type = KeyType.entries[row[at + KeyGraphData.TYPE].toInt()]
        val label = row[at + KeyGraphData.LABEL]
        val group = row[at + KeyGraphData.GROUP].toInt()
        val popup = row[at + KeyGraphData.POPUP].toInt().takeIf { it >= 0 }?.let { popupSets.getOrPut(it) { PopupSet() } }
        val multi = row[at + KeyGraphData.MULTI].toInt()
        if (multi >= 0) return multis.getOrPut(multi) {
            val codes = graph.call(KeyGraphOp.CODE_POINTS, "$multi").map(String::toInt).toIntArray()
            MultiTextKeyData(type, codes, label, group, popup)
        }
        return TextKeyData(type, row[at + KeyGraphData.CODE].toInt(), label, group, popup)
    }

    companion object {
        /** The keyboard shown before any is built. */
        val Placeholder by lazy { RustKeyboard(KeyGraph.placeholder()) }

        /** [keyboard] if Rust built it, else [Placeholder]. */
        fun of(keyboard: Keyboard) = keyboard as? RustKeyboard ?: Placeholder

        private fun hints(config: KeyHintConfiguration) = KeyGraphHints.row(
            KeyGraphHints.SYMBOL to "${config.symbolHintMode.ordinal}",
            KeyGraphHints.NUMBER to "${config.numberHintMode.ordinal}",
            KeyGraphHints.MERGE to WireFlag.encode(config.mergeHintPopups),
        )
    }
}

/** One listed key of a [RustKeyboard]: what Rust computed for it, and where it was laid out. */
class RustKey internal constructor(base: Int) : Key(TextKeyData(code = base)) {
    var computedData: KeyData = TextKeyData.UNSPECIFIED
        private set
    /** The data the key had when it went down. */
    var computedDataOnDown: KeyData = TextKeyData.UNSPECIFIED
    /** The icon token Rust picked, 0 for none. */
    var icon = 0
        private set

    internal fun update(row: Array<String>, at: Int, data: (Array<String>, Int) -> KeyData) {
        isVisible = WireFlag.decode(row[at + KeyGraphKey.VISIBLE])
        isEnabled = WireFlag.decode(row[at + KeyGraphKey.ENABLED])
        computedData = data(row, at + KeyGraphKey.TYPE)
        label = row[at + KeyGraphKey.SHOWN].takeIf { WireFlag.decode(row[at + KeyGraphKey.LABELED]) }
        hintedLabel = row[at + KeyGraphKey.HINT].takeIf { WireFlag.decode(row[at + KeyGraphKey.HINTED]) }
        icon = row[at + KeyGraphKey.ICON].toInt()
    }

    internal fun place(numbers: FloatArray, at: Int) {
        touchBounds.apply { left = numbers[at]; top = numbers[at + 1]; right = numbers[at + 2]; bottom = numbers[at + 3] }
        visibleBounds.apply { left = numbers[at + 4]; top = numbers[at + 5]; right = numbers[at + 6]; bottom = numbers[at + 7] }
    }
}
