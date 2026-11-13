package dev.patrickgold.florisboard.ime.keyboard

import dev.patrickgold.florisboard.ime.popup.MutablePopupSet
import dev.patrickgold.florisboard.ime.popup.PopupMapping
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.ime.text.keyboard.MultiTextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.ReferenceTextKeyboard
import dev.patrickgold.florisboard.ime.text.keyboard.TextKey
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyboard
import dev.patrickgold.florisboard.lib.FlorisRect
import java.util.Locale

/** The entire evaluated list, including popup/hint settings and bit-exact display/touch bounds.
 * Uses no Android APIs or Robolectric. Resource labels/icons and ICU stay outside the port. */
internal object LayoutSnapshot {
    private fun key(data: KeyData?): Any? = data?.let {
        listOf(it.type, it.code, it.label, it.groupId, (it as? MultiTextKeyData)?.codePoints?.toList())
    }
    private fun bounds(r: FlorisRect) = listOf(r.left.toBits(), r.top.toBits(), r.right.toBits(), r.bottom.toBits())

    fun evaluate(rows: LayoutArrangement, hints: List<Pair<KeyData?, KeyData?>>, mode: KeyboardMode,
                 evaluator: ComputingEvaluator, mappings: List<PopupMapping?>, prepared: LayoutFixtures.Prepared,
                 locale: Locale, config: KeyHintConfiguration, reference: Boolean, width: Float, height: Float): List<Any?> {
        val evaluated = rows.map { row -> row.map { it.compute(evaluator) } }
        val keys = Array(rows.size) { r -> Array(rows[r].size) { k ->
            TextKey(rows[r][k]).apply {
                val data = evaluated[r][k]
                if (data == null) {
                    flayShrink = 0f; flayGrow = 0f; flayWidthFactor = 0f
                } else if (reference) {
                    val flex = ReferenceTextKeyboard.flex(mode, data.code, data.type)
                    flayShrink = flex.first; flayGrow = flex.second; flayWidthFactor = flex.third
                } else applyFlex(mode, data.code, data.type)
            }
        } }
        val desired = ReferenceTextKeyboard.desiredKey(mode, rows.size, width, height, 150f, 2f, 5f)
        if (reference) ReferenceTextKeyboard(keys).layout(width, height, desired, true)
        else TextKeyboard(keys, mode, null, null).layout(width, height, desired, true)
        var index = 0
        return rows.indices.flatMap { r -> rows[r].indices.map { k ->
            val data = evaluated[r][k]
            val (number, symbol) = hints[index++]
            val popup = MutablePopupSet<KeyData>()
            fun merge(set: PopupSet<AbstractKeyData>?) { prepared.popup(set)?.let { popup.merge(it, evaluator) } }
            fun mapped(label: String, variation: KeyVariation) = mappings[0]?.get(variation)?.get(label)
                ?: mappings[1]?.get(variation)?.get(label)
            if (data != null) {
                merge(data.popup)
                if (ReferenceKeyRules.popupMode(mode.value)) {
                    val label = data.label.lowercase(locale)
                    val group = ReferenceKeyRules.popupGroup(data.groupId) ?: label
                    val sources = KeyVariation.entries.flatMap { listOf(mappings[0]?.get(it)?.get(group), mappings[1]?.get(it)?.get(group)) }
                    val present = sources.indices.fold(0) { bits, i -> if (sources[i] != null) bits or (1 shl i) else bits }
                    if (group != label) merge(mapped(label, KeyVariation.ALL))
                    merge(sources.getOrNull(ReferenceKeyRules.popupSource(evaluator.state.keyVariation.value, present)))
                    if (data.type == KeyType.CHARACTER) {
                        fun hint(hint: KeyData?, numeric: Boolean) {
                            if (hint == null || hint.code == data.code) return
                            val computed = hint.compute(evaluator)
                            if (numeric) popup.numberHint = computed else popup.symbolHint = computed
                            val sets = listOf(computed?.popup, mapped(hint.label, KeyVariation.ALL))
                            sets.forEach { set -> prepared.popup(set)?.let {
                                if (numeric) popup.mergeNumberHint(it, evaluator) else popup.mergeSymbolHint(it, evaluator)
                            } }
                        }
                        hint(symbol, false); hint(number, true)
                    }
                }
            }
            val ordered = popup.getPopupKeys(config)
            listOf(r, k, key(data), key(number), key(symbol), key(ordered.hint),
                ordered.prioritized.map(::key), ordered.other.map(::key),
                keys[r][k].flayShrink.toBits(), keys[r][k].flayGrow.toBits(), keys[r][k].flayWidthFactor.toBits(),
                bounds(keys[r][k].touchBounds), bounds(keys[r][k].visibleBounds))
        } }
    }
}
