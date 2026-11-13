package com.jtn0123.audiowhisper.render

import androidx.compose.runtime.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import com.jtn0123.audiowhisper.gif.*
import com.jtn0123.audiowhisper.dictation.DictationFields
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import org.florisboard.lib.compose.stringRes

internal data class RendererGif(val showing: Boolean,val gifs: List<Gif>,val chips: List<String>,val shown: String,
    val status: Int,val labels: Array<String>,val bottom: Float,val ids: LongArray) {
    fun bind(native: KeyboardRenderer) = native.gifs(floatArrayOf(status.toFloat(),bottom)+gifs.map { it.aspectRatio }.toFloatArray(),
        arrayOf(shown)+labels+chips+gifs.map { it.title.ifBlank { labels[3] } },chips.size,ids)
}
@Composable
internal fun rendererGif(): RendererGif = with(LocalDensity.current) {
    val showing by GifSearch.showingGifs.collectAsState()
    val results by GifSearch.results.collectAsState()
    val shown by GifSearch.shown.collectAsState()
    val chips by GifSearch.chips.collectAsState()
    val context=LocalContext.current
    val editor by context.editorInstance()
    val keyboard by context.keyboardManager()
    LaunchedEffect(showing) {if(showing) GifSearch.offerTypedWord(editor.activeContent.textBeforeSelection,
        keyboard.activeState.isIncognitoMode || DictationFields.isSecret(editor.activeInfo.inputAttributes.variation))}
    val message=when(val r=results) {
        GifSearch.Results.Loading -> "Loading"
        is GifSearch.Results.Failed -> stringRes(R.string.gif__try_again,"problem" to r.message)
        is GifSearch.Results.Loaded -> stringRes(R.string.gif__none_for,"query" to shown)
    }
    val identities=remember { java.util.concurrent.atomic.AtomicLong(-1) }
    val gifs=(results as? GifSearch.Results.Loaded)?.gifs.orEmpty()
    val ids=remember(gifs) {LongArray(gifs.size){identities.getAndDecrement()}}
    RendererGif(showing,(results as? GifSearch.Results.Loaded)?.gifs.orEmpty(),chips,shown,
        when(results) {GifSearch.Results.Loading->0;is GifSearch.Results.Loaded->1;is GifSearch.Results.Failed->2},
        arrayOf(stringRes(R.string.gif__search_giphy),stringRes(R.string.gif__powered_by),message,
            stringRes(R.string.gif__send),stringRes(R.string.gif__back_to_trending)),FlorisImeSizing.keyboardRowBaseHeight.toPx()*.8f,ids)
}
