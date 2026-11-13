package dev.patrickgold.florisboard.ime.keyboard

import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName

/** LayoutManager at 2098ceec, with only IO/coroutines and TextKey allocation removed.
 * Decisions and loop order are retained independently of LayoutAssembly and JNI. */
internal object ReferenceLayoutAssembly {
    fun plan(mode: KeyboardMode, numberRow: Boolean, layouts: (LayoutType) -> ExtensionComponentName?): List<Pair<LayoutType, ExtensionComponentName>?> {
        var main: LayoutType? = null
        var modifier: LayoutType? = null
        var extension: LayoutType? = null
        when (mode) {
            KeyboardMode.CHARACTERS -> {
                if (numberRow) extension = LayoutType.NUMERIC_ROW
                main = LayoutType.CHARACTERS
                modifier = LayoutType.CHARACTERS_MOD
            }
            KeyboardMode.NUMERIC -> main = LayoutType.NUMERIC
            KeyboardMode.NUMERIC_ADVANCED -> main = LayoutType.NUMERIC_ADVANCED
            KeyboardMode.PHONE -> main = LayoutType.PHONE
            KeyboardMode.PHONE2 -> main = LayoutType.PHONE2
            KeyboardMode.SYMBOLS -> {
                extension = LayoutType.NUMERIC_ROW
                main = LayoutType.SYMBOLS
                modifier = LayoutType.SYMBOLS_MOD
            }
            KeyboardMode.SYMBOLS2 -> {
                main = LayoutType.SYMBOLS2
                modifier = LayoutType.SYMBOLS2_MOD
            }
            else -> Unit
        }
        return listOf(main?.let { it to checkNotNull(layouts(it)) },
            modifier?.let { it to extCoreLayout("default") }, extension?.let { it to checkNotNull(layouts(it)) })
    }

    fun modifier(type: LayoutType) = when (type) {
        LayoutType.SYMBOLS -> LayoutType.SYMBOLS_MOD
        LayoutType.SYMBOLS2 -> LayoutType.SYMBOLS2_MOD
        else -> LayoutType.CHARACTERS_MOD
    }

    fun merge(main: LayoutArrangement?, modifier: LayoutArrangement?, extension: LayoutArrangement?): LayoutArrangement {
        val computed = arrayListOf<List<AbstractKeyData>>()
        if (extension != null) {
            for (row in extension) computed.add(row.toList())
        }
        if (main != null && modifier != null) {
            for (mainRowI in main.indices) {
                val mainRow = main[mainRowI]
                if (mainRowI + 1 < main.size) {
                    computed.add(mainRow.toList())
                } else {
                    val row = arrayListOf<AbstractKeyData>()
                    val firstModRow = modifier.firstOrNull()
                    for (modKey in firstModRow ?: listOf()) {
                        if (modKey is TextKeyData && modKey.code == 0) row.addAll(mainRow)
                        else row.add(modKey)
                    }
                    computed.add(row)
                }
            }
            for (modRowI in 1 until modifier.size) computed.add(modifier[modRowI].toList())
        } else if (main != null && modifier == null) {
            for (row in main) computed.add(row.toList())
        } else if (main == null && modifier != null) {
            for (row in modifier) computed.add(row.toList())
        }
        return computed
    }

    fun hints(mode: KeyboardMode, numberHints: Boolean, rows: LayoutArrangement, symbols: LayoutArrangement,
              evaluator: ComputingEvaluator): List<Pair<KeyData?, KeyData?>> {
        val numbers = rows.map { arrayOfNulls<KeyData>(it.size) }
        val hints = rows.map { arrayOfNulls<KeyData>(it.size) }
        fun addRow(r: Int, hintRow: List<AbstractKeyData>, hintType: KeyType) {
            for (k in rows[r].indices) {
                val hintKey = hintRow.getOrNull(k)?.compute(evaluator)
                if (hintKey?.type != hintType) continue
                when (hintType) {
                    KeyType.CHARACTER -> hints[r][k] = hintKey
                    KeyType.NUMERIC -> numbers[r][k] = hintKey
                    else -> Unit
                }
            }
        }
        if (mode == KeyboardMode.CHARACTERS && rows.isNotEmpty()) {
            if (numberHints && symbols.isNotEmpty()) addRow(0, symbols[0], KeyType.NUMERIC)
            val offset = rows.size - symbols.size
            for (r in rows.indices) {
                if (r < offset) continue
                symbols.getOrNull(r - offset)?.let { addRow(r, it, KeyType.CHARACTER) }
            }
        }
        return rows.indices.flatMap { r -> rows[r].indices.map { k -> numbers[r][k] to hints[r][k] } }
    }
}
