package com.jtn0123.audiowhisper.render

import android.content.Context
import org.florisboard.lib.android.showShortToast
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.ime.ImeUiMode
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

/** Executes native commands through the same clipboard store / InputConnection paths as Compose. */
internal class RendererClipboardActions(private val context: Context) {
    private val clipboard by context.clipboardManager()
    private val keyboard by context.keyboardManager()
    private val prefs by FlorisPreferenceStore
    private val scope = CoroutineScope(SupervisorJob()+Dispatchers.Main)
    fun dispatch(op: Int, index: Int, snapshot: RendererClipboard) {
        when(op) {
            0 -> keyboard.activeState.imeUiMode=ImeUiMode.TEXT
            1 -> scope.launch { prefs.clipboard.historyEnabled.set(index!=0) }
            13 -> scope.launch { context.showShortToast(dev.patrickgold.florisboard.R.string.clipboard__cleared_history) }
            else -> snapshot.entries.getOrNull(index)?.let { item ->
                when(op) {
                    5,9 -> clipboard.pasteItem(item)
                    7 -> clipboard.pinClip(item)
                    14 -> clipboard.unpinClip(item)
                    8 -> clipboard.deleteClip(item,onlyIfUnpinned=false)
                    12 -> clipboard.clearExactHistory(listOf(item))
                }
            }
        }
    }
}
