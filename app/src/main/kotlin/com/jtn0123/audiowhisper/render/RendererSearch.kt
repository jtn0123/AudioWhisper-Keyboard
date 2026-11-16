package com.jtn0123.audiowhisper.render

import androidx.compose.runtime.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Search
import com.jtn0123.audiowhisper.gif.GifSearch
import dev.patrickgold.florisboard.R
import org.florisboard.lib.compose.stringRes

internal data class RendererSearch(val active: Boolean,val labels: Array<String>)
@Composable
internal fun rendererSearch(): RendererSearch {
    val active by GifSearch.typing.collectAsState()
    val query by GifSearch.query.collectAsState()
    return RendererSearch(active,if(active) arrayOf(query,stringRes(R.string.gif__back),stringRes(R.string.gif__search_giphy),stringRes(R.string.gif__search),iconPath(Icons.AutoMirrored.Filled.ArrowBack),iconPath(Icons.Default.Search)) else emptyArray())
}

/** Preserve the action cohort while reserving one independent search row, as GifSearchBar does. */
internal fun gifSearchBarConfig(original: FloatArray,row: Float): FloatArray = original.copyOf().apply {
    this[0]=1f;this[1]=0f
    for(index in 2..7)this[index]=0f
    this[9]=0f;this[11]=row
    for(index in 12 until size)this[index]=0f
}
