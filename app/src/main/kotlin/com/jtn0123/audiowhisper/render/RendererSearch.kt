package com.jtn0123.audiowhisper.render

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Search
import com.jtn0123.audiowhisper.gif.GifSearch
import dev.patrickgold.florisboard.R

internal data class RendererSearch(val active: Boolean,val labels: Array<String>)

internal fun rendererSearch(r: RootReads): RendererSearch {
    val active = GifSearch.typing.value
    return RendererSearch(active, if (active) arrayOf(GifSearch.query.value, r.string(R.string.gif__back), r.string(R.string.gif__search_giphy),
        r.string(R.string.gif__search), iconPath(Icons.AutoMirrored.Filled.ArrowBack), iconPath(Icons.Default.Search)) else emptyArray())
}
