package com.jtn0123.audiowhisper.render

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.RadioButtonChecked
import androidx.compose.material.icons.filled.RadioButtonUnchecked
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.subtypeManager
import org.florisboard.lib.compose.stringRes

internal data class RendererSubtypes(val selected: Long, val ids: LongArray, val labels: Array<String>)
@Composable
internal fun rendererSubtypes(): RendererSubtypes {
    val manager by LocalContext.current.subtypeManager()
    val subtypes by manager.subtypesFlow.collectAsState()
    val active by manager.activeSubtypeFlow.collectAsState()
    return RendererSubtypes(active.id, subtypes.map { it.id }.toLongArray(),
        (listOf(stringRes(R.string.select_subtype_panel__header), iconPath(Icons.Default.RadioButtonChecked),
            iconPath(Icons.Default.RadioButtonUnchecked)) + subtypes.map { it.primaryLocale.displayName() }).toTypedArray())
}
