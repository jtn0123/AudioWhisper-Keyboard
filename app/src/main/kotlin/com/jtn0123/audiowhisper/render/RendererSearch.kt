package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.gif.GifSearch
import dev.patrickgold.florisboard.R

internal data class RendererSearch(val active: Boolean,val labels: Array<String>)

internal fun rendererSearch(r: RootReads): RendererSearch {
    val active = GifSearch.typing.value
    return RendererSearch(active, if (active) arrayOf(GifSearch.query.value, r.string(R.string.gif__back), r.string(R.string.gif__search_giphy),
        r.string(R.string.gif__search)) else emptyArray())
}
