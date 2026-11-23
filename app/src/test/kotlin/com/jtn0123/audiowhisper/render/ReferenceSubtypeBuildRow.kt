package com.jtn0123.audiowhisper.render

import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.LayoutType
import dev.patrickgold.florisboard.ime.keyboard.extCorePopupMapping
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName

/**
 * Main's `RustKeyGraphs.build` at 61f1513fb, frozen for the subtype trace: a key graph's files came
 * from the Kotlin `Subtype`, its layouts by `SubtypeLayoutMap.get` and its popup mapping, and the
 * keyboard cache kept it under the subtype's `hashCode()`. The row itself is `keyGraphBuild`'s,
 * which this item didn't change and the key graph trace checks.
 */
internal object ReferenceSubtypeBuildRow {
    fun row(mode: KeyboardMode, numberRow: Boolean, numberHints: Boolean, subtype: Subtype,
            namedModifier: (LayoutName) -> ExtensionComponentName?, layout: (LayoutName) -> String?,
            mapping: (ExtensionComponentName) -> String?): Array<String> {
        val files = KeyGraphFiles(
            { type -> layoutOf(subtype, type) }, namedModifier, layout,
            mapping(subtype.popupMapping), mapping(extCorePopupMapping("default")),
        )
        return keyGraphBuild(mode, numberRow, numberHints, files)
    }

    /** The keyboard cache's key for [subtype]. */
    fun cacheKey(subtype: Subtype): Int = subtype.hashCode()

    private fun layoutOf(subtype: Subtype, type: LayoutType): ExtensionComponentName? = with(subtype.layoutMap) {
        when (type) {
            LayoutType.CHARACTERS -> characters
            LayoutType.SYMBOLS -> symbols
            LayoutType.SYMBOLS2 -> symbols2
            LayoutType.NUMERIC -> numeric
            LayoutType.NUMERIC_ADVANCED -> numericAdvanced
            LayoutType.NUMERIC_ROW -> numericRow
            LayoutType.PHONE -> phone
            LayoutType.PHONE2 -> phone2
            else -> null
        }
    }
}
