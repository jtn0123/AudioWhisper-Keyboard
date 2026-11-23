package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.gif.*
import com.jtn0123.audiowhisper.dictation.DictationFields
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.R
import java.util.concurrent.atomic.AtomicLong

internal data class RendererGif(val showing: Boolean,val gifs: List<Gif>,val chips: List<String>,val shown: String,
    val status: Int,val labels: Array<String>,val bottom: Float,val ids: LongArray) {
    fun bind(native: KeyboardRenderer) = native.gifs(floatArrayOf(status.toFloat(),bottom)+gifs.map { it.aspectRatio }.toFloatArray(),
        arrayOf(shown)+labels+chips+gifs.map { it.title.ifBlank { labels[3] } },chips.size,ids)
}

/** GIF results and strings; each new results list gets fresh ids, and showing GIFs offers the word typed before the cursor. */
internal class GifSource(private val r: RootReads) {
    private val identities = AtomicLong(-1)
    private var ids: Pair<List<Gif>, LongArray>? = null
    private var showing = false

    fun read(): RendererGif {
        val showing = GifSearch.showingGifs.value
        if (showing && !this.showing) GifSearch.offerTypedWord(r.editor.activeContent.textBeforeSelection,
            r.keyboard.activeState.isIncognitoMode || DictationFields.isSecret(r.editor.activeInfo.inputAttributes.variation))
        this.showing = showing
        val results = GifSearch.results.value
        val shown = GifSearch.shown.value
        val message = when (results) {
            GifSearch.Results.Loading -> "Loading"
            is GifSearch.Results.Failed -> r.string(R.string.gif__try_again, "problem" to results.message)
            is GifSearch.Results.Loaded -> r.string(R.string.gif__none_for, "query" to shown)
        }
        val gifs = (results as? GifSearch.Results.Loaded)?.gifs.orEmpty()
        val ids = ids?.takeIf { it.first == gifs }?.second ?: LongArray(gifs.size) { identities.getAndDecrement() }.also { ids = gifs to it }
        return RendererGif(showing, gifs, GifSearch.chips.value, shown,
            when (results) { GifSearch.Results.Loading -> 0; is GifSearch.Results.Loaded -> 1; is GifSearch.Results.Failed -> 2 },
            arrayOf(r.string(R.string.gif__search_giphy), r.string(R.string.gif__powered_by), message,
                r.string(R.string.gif__send), r.string(R.string.gif__back_to_trending)), r.rowHeight * .8f, ids)
    }
}
