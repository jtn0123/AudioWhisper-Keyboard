package com.jtn0123.audiowhisper.render

import android.content.Context
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.ime.ImeUiMode
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

/** Executes native commands through the same Room / InputConnection paths as Compose. */
internal class RendererClipboardActions(context: Context) {
    private val clipboard by context.clipboardManager()
    private val keyboard by context.keyboardManager()
    private val prefs by FlorisPreferenceStore
    private val scope = CoroutineScope(SupervisorJob()+Dispatchers.Main)
    fun dispatch(op: Int, index: Int, snapshot: RendererClipboard) {
        when(op) {
            0 -> keyboard.activeState.imeUiMode=ImeUiMode.TEXT
            1 -> scope.launch { prefs.clipboard.historyEnabled.set(!prefs.clipboard.historyEnabled.get()) }
            else -> snapshot.entries.getOrNull(index)?.let { item ->
                when(op) {
                    5,9 -> clipboard.pasteItem(item)
                    7 -> if(item.isPinned) clipboard.unpinClip(item) else clipboard.pinClip(item)
                    8 -> clipboard.deleteClip(item,onlyIfUnpinned=false)
                }
            }
        }
    }
}
