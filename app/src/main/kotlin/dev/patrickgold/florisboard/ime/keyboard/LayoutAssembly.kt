package dev.patrickgold.florisboard.ime.keyboard

import com.jtn0123.audiowhisper.rust.LayoutRules
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName

/** Decoded objects stay in Kotlin; Rust chooses load types, key order and hint sources. */
internal object LayoutAssembly {
    fun plan(mode: KeyboardMode, numberRow: Boolean, layouts: (LayoutType) -> ExtensionComponentName?) =
        LayoutRules.plan(mode.value, numberRow).map { type ->
            LayoutType.entries.getOrNull(type)?.let { it to (layouts(it) ?: extCoreLayout("default")) }
        }

    fun modifier(type: LayoutType) = LayoutType.entries[LayoutRules.modifierType(type.ordinal)]

    fun merge(main: LayoutArrangement?, modifier: LayoutArrangement?, extension: LayoutArrangement?): LayoutArrangement =
        LayoutRules.merge(main, modifier, extension) { (it as? TextKeyData)?.code ?: Int.MIN_VALUE }

    fun hints(mode: KeyboardMode, numberHints: Boolean, rows: LayoutArrangement, symbols: LayoutArrangement,
              evaluator: ComputingEvaluator): List<Pair<KeyData?, KeyData?>> {
        val evaluated = symbols.flatten().map { it.compute(evaluator) }
        val indices = LayoutRules.hints(mode.value, numberHints, LayoutRules.sizes(rows), LayoutRules.sizes(symbols),
            evaluated.map { it?.type?.ordinal ?: -1 }.toIntArray())
        return indices.asList().chunked(2).map { (number, symbol) ->
            evaluated.getOrNull(number) to evaluated.getOrNull(symbol)
        }
    }
}
