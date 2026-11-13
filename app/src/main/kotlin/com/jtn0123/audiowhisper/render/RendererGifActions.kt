package com.jtn0123.audiowhisper.render

import android.content.Context
import com.jtn0123.audiowhisper.gif.*
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.ime.ImeUiMode
import kotlinx.coroutines.*
import org.florisboard.lib.android.showShortToast

internal class RendererGifActions(private val context: Context,private val sent: () -> Unit) {
    private val keyboard by context.keyboardManager()
    private val editor by context.editorInstance()
    private val scope=CoroutineScope(SupervisorJob()+Dispatchers.Main)
    fun dispatch(op: Int,index: Int,snapshot: RendererGif) {
        when(op) {
            9 -> {GifSearch.stopTyping();keyboard.activeState.imeUiMode=ImeUiMode.MEDIA}
            10 -> {GifSearch.submit();keyboard.activeState.imeUiMode=ImeUiMode.MEDIA}
            0 -> {GifSearch.startTyping();keyboard.activeState.imeUiMode=ImeUiMode.TEXT}
            1 -> GifSearch.load(GifSearch.TRENDING)
            2 -> snapshot.chips.getOrNull(index)?.let(GifSearch::load)
            3 -> snapshot.gifs.getOrNull(index)?.let { gif -> scope.launch {
                try {
                    val outcome=GifSender.send(context,editor,gif)
                    if(outcome!=GifSender.Outcome.FAILED) {
                        GifSearch.sent(gif,keyboard.activeState.isIncognitoMode)
                        keyboard.activeState.imeUiMode=ImeUiMode.TEXT
                    }
                    when(outcome) {
                        GifSender.Outcome.LINKED->context.showShortToast(R.string.gif__linked)
                        GifSender.Outcome.FAILED->context.showShortToast(R.string.gif__send_failed)
                        GifSender.Outcome.SENT->Unit
                    }
                }finally{sent()}
            } }
            4 -> GifSearch.load(snapshot.shown)
            5 -> keyboard.activeState.imeUiMode=ImeUiMode.TEXT
            6 -> GifSearch.showGifs(false)
            7 -> GifSearch.showGifs(true)
        }
    }
    fun close(){scope.cancel()}
}
