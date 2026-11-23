package com.jtn0123.audiowhisper.render

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.RadioButtonChecked
import androidx.compose.material.icons.filled.RadioButtonUnchecked
import dev.patrickgold.florisboard.R

internal data class RendererSubtypes(val selected: Long, val ids: LongArray, val labels: Array<String>)

internal fun rendererSubtypes(r: RootReads): RendererSubtypes {
    val subtypes = r.subtypes.subtypesFlow.value
    return RendererSubtypes(r.subtypes.activeSubtypeFlow.value.id, subtypes.map { it.id }.toLongArray(),
        (listOf(r.string(R.string.select_subtype_panel__header), iconPath(Icons.Default.RadioButtonChecked),
            iconPath(Icons.Default.RadioButtonUnchecked)) + subtypes.map { it.primaryLocale.displayName() }).toTypedArray())
}
