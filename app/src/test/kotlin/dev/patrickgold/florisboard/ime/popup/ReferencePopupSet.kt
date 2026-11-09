package dev.patrickgold.florisboard.ime.popup

import dev.patrickgold.florisboard.ime.keyboard.AbstractKeyData
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode

/**
 * The Kotlin that `MutablePopupSet`'s ordering and merging were before they moved to Rust
 * (lib/rust/crates/popup/src/order.rs), copied verbatim from PopupSet.kt for [PopupComparison]
 * to hold the new code up against. Only the generic `T` became [KeyData], so that it compiles
 * without an unchecked cast.
 */
class ReferenceMutablePopupSet(
    override var main: KeyData? = null,
    override val relevant: ArrayList<KeyData> = arrayListOf(),
    var symbolHint: KeyData? = null,
    var numberHint: KeyData? = null,
    private val symbolPopups: ArrayList<KeyData> = arrayListOf(),
    private val numberPopups: ArrayList<KeyData> = arrayListOf(),
    private val configCache: MutableMap<KeyHintConfiguration, PopupKeys<KeyData>> = mutableMapOf()
) : PopupSet<KeyData>(main, relevant) {

    fun clear() {
        symbolHint = null
        numberHint = null
        main = null
        relevant.clear()
        symbolPopups.clear()
        numberPopups.clear()
        configCache.clear()
    }

    override fun getPopupKeys(keyHintConfiguration: KeyHintConfiguration): PopupKeys<KeyData> {
        return configCache.getOrPut(keyHintConfiguration) {
            initPopupList(keyHintConfiguration)
        }
    }

    private fun initPopupList(keyHintConfiguration: KeyHintConfiguration): PopupKeys<KeyData> {
        val localMain = main
        val localRelevant = relevant
        val localSymbolHint = symbolHint
        val localNumberHint = numberHint
        if (localSymbolHint != null && keyHintConfiguration.symbolHintMode != KeyHintMode.DISABLED) {
            if (localNumberHint != null && keyHintConfiguration.numberHintMode != KeyHintMode.DISABLED) {
                val hintPopups = if (keyHintConfiguration.mergeHintPopups) { symbolPopups + numberPopups } else { listOf() }
                return when (keyHintConfiguration.symbolHintMode) {
                    KeyHintMode.ACCENT_PRIORITY -> when (keyHintConfiguration.numberHintMode) {
                        // when both hints are present in accent priority, always have a non-hint key first if possible
                        KeyHintMode.ACCENT_PRIORITY -> when {
                            localMain != null -> PopupKeys(localSymbolHint, listOf(localMain, localSymbolHint, localNumberHint), localRelevant + hintPopups)
                            localRelevant.isNotEmpty() -> PopupKeys(localSymbolHint, listOf(localRelevant[0], localSymbolHint, localNumberHint), localRelevant.subList(1, localRelevant.size) + hintPopups)
                            else -> PopupKeys(localSymbolHint, listOf(localSymbolHint, localNumberHint), hintPopups)
                        }
                        // hint priority of number hint wins and overrules accent priority of symbol hint
                        KeyHintMode.HINT_PRIORITY -> PopupKeys(localSymbolHint, listOfNotNull(localNumberHint, localMain, localSymbolHint), localRelevant + hintPopups)
                        // due to smart priority of number hint, main wins if it exists, otherwise number hint overrules accent priority of symbol hint
                        else -> PopupKeys(localSymbolHint, listOfNotNull(localMain, localNumberHint, localSymbolHint), localRelevant + hintPopups)
                    }
                    KeyHintMode.HINT_PRIORITY -> when (keyHintConfiguration.numberHintMode) {
                        // when both hints are present in hint priority, symbol hint wins
                        KeyHintMode.HINT_PRIORITY -> PopupKeys(localSymbolHint, listOfNotNull(localSymbolHint, localNumberHint, localMain), localRelevant + hintPopups)
                        // hint priority of symbol hint wins, and overrules potential accent priority of number hint
                        else -> PopupKeys(localSymbolHint, listOfNotNull(localSymbolHint, localMain, localNumberHint), localRelevant + hintPopups)
                    }
                    else -> when (keyHintConfiguration.numberHintMode) {
                        // smart priority of symbol hint wins, and overrules accent priority of number hint
                        KeyHintMode.ACCENT_PRIORITY -> PopupKeys(localSymbolHint, listOfNotNull(localMain, localSymbolHint, localNumberHint), localRelevant + hintPopups)
                        // hint priority of number hint wins
                        KeyHintMode.HINT_PRIORITY -> PopupKeys(localSymbolHint, listOfNotNull(localNumberHint, localMain, localSymbolHint), localRelevant + hintPopups)
                        // when both hints are in smart priority, always have main first if possible
                        else -> PopupKeys(localSymbolHint, listOfNotNull(localMain, localSymbolHint, localNumberHint), localRelevant + hintPopups)
                    }
                }
            } else {
                val hintPopups = if (keyHintConfiguration.mergeHintPopups) { symbolPopups } else { listOf() }
                return when (keyHintConfiguration.symbolHintMode) {
                    // in accent priority, always show a non-hint key first if possible
                    KeyHintMode.ACCENT_PRIORITY -> when {
                        localMain != null -> PopupKeys(localSymbolHint, listOf(localMain, localSymbolHint), localRelevant + hintPopups)
                        localRelevant.isNotEmpty() -> PopupKeys(localSymbolHint, listOf(localRelevant[0], localSymbolHint), localRelevant.subList(1, localRelevant.size) + hintPopups)
                        else -> PopupKeys(localSymbolHint, listOf(localSymbolHint), hintPopups)
                    }
                    // in hint priority, always show hint first
                    KeyHintMode.HINT_PRIORITY -> PopupKeys(localSymbolHint, listOfNotNull(localSymbolHint, localMain), localRelevant + hintPopups)
                    // in smart priority, show main first if possible
                    else -> PopupKeys(localSymbolHint, listOfNotNull(localMain, localSymbolHint), localRelevant + hintPopups)
                }
            }
        } else if (localNumberHint != null && keyHintConfiguration.numberHintMode != KeyHintMode.DISABLED) {
            val hintPopups = if (keyHintConfiguration.mergeHintPopups) { numberPopups } else { listOf() }
            return when (keyHintConfiguration.numberHintMode) {
                // in accent priority, always show a non-hint key first if possible
                KeyHintMode.ACCENT_PRIORITY -> when {
                    localMain != null -> PopupKeys(localNumberHint, listOf(localMain, localNumberHint), localRelevant + hintPopups)
                    localRelevant.isNotEmpty() -> PopupKeys(localNumberHint, listOf(localRelevant[0], localNumberHint), localRelevant.subList(1, localRelevant.size) + hintPopups)
                    else -> PopupKeys(localNumberHint, listOf(localNumberHint), hintPopups)
                }
                // in hint priority, always show hint first
                KeyHintMode.HINT_PRIORITY -> PopupKeys(localNumberHint, listOfNotNull(localNumberHint, localMain), localRelevant + hintPopups)
                // in smart priority, show main first if possible
                else -> PopupKeys(localNumberHint, listOfNotNull(localMain, localNumberHint), localRelevant + hintPopups)
            }
        } else {
            // if no hints shall be shown, use main first if possible
            return PopupKeys(null, listOfNotNull(localMain), localRelevant)
        }
    }

    fun merge(other: PopupSet<AbstractKeyData>, evaluator: ComputingEvaluator) {
        mergeInternal(other, evaluator, relevant, true)
    }

    fun mergeSymbolHint(hintPopups: PopupSet<AbstractKeyData>, evaluator: ComputingEvaluator) {
        mergeInternal(hintPopups, evaluator, symbolPopups)
    }

    fun mergeNumberHint(hintPopups: PopupSet<AbstractKeyData>, evaluator: ComputingEvaluator) {
        mergeInternal(hintPopups, evaluator, numberPopups)
    }

    private fun mergeInternal(other: PopupSet<AbstractKeyData>, evaluator: ComputingEvaluator, targetList: MutableList<KeyData>, useMain: Boolean = false) {
        other.relevant.forEach {
            val data = it.compute(evaluator)
            if (data != null) {
                targetList.add(data)
            }
        }
        other.main?.let {
            val data = it.compute(evaluator)
            if (data != null) {
                if (useMain && main == null) {
                    main = data
                } else {
                    targetList.add(data)
                }
            }
        }
    }
}
