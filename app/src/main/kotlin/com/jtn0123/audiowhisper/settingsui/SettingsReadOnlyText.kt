package com.jtn0123.audiowhisper.settingsui

import android.content.ClipData
import android.content.ClipboardManager
import android.os.Bundle
import android.view.HapticFeedbackConstants
import android.view.accessibility.AccessibilityNodeInfo
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.lib.io.FlorisRef
import dev.patrickgold.florisboard.lib.io.loadTextAsset
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Android glue for the project licence's read-only text. Rust lays it out, owns the selection and
 * places the toolbar, which SettingsSelectionMenu shows; this reads the asset off the main thread,
 * puts copied text on the clipboard, ticks the handle haptic and hands TalkBack's text actions to
 * Rust.
 */
internal class SettingsReadOnlyText(private val host: RustSettingsView, private val scope: CoroutineScope) {
    private val clipboard = host.context.getSystemService(ClipboardManager::class.java)

    fun execute(p: List<String>) {
        when (p[0]) {
            SettingsUiTextEffect.ASSET -> scope.launch {
                val read = withContext(Dispatchers.IO) { FlorisRef.assets(p[1]).loadTextAsset(host.context) }
                host.send(SettingsUiOp.ASSET, *read.fold({ arrayOf(p[1], SettingsUiAssetOutcome.OK, it) },
                    { arrayOf(p[1], SettingsUiAssetOutcome.FAILED, it.message.orEmpty()) }))
            }
            // As Compose's AndroidClipboardManager.setText labels it.
            SettingsUiTextEffect.CLIP -> clipboard.setPrimaryClip(ClipData.newPlainText("plain text", p[1]))
            SettingsUiTextEffect.HAPTIC -> host.performHapticFeedback(HapticFeedbackConstants.TEXT_HANDLE_MOVE)
        }
    }

    /** TalkBack's text actions on the licence's node; null for any other node. */
    fun access(id: Int, action: Int, args: Bundle?): Boolean? {
        if (id != SettingsUiTextNode.TEXT) return null
        val int = { key: String -> (args?.getInt(key, -1) ?: -1).toString() }
        val flag = { yes: Boolean -> if (yes) "1" else "0" }
        host.send(SettingsUiOp.SELECTION, *when (action) {
            AccessibilityNodeInfo.ACTION_SET_SELECTION -> arrayOf(SettingsUiSelection.SET,
                int(AccessibilityNodeInfo.ACTION_ARGUMENT_SELECTION_START_INT), int(AccessibilityNodeInfo.ACTION_ARGUMENT_SELECTION_END_INT))
            AccessibilityNodeInfo.ACTION_COPY -> arrayOf(SettingsUiSelection.COPY)
            AccessibilityNodeInfo.ACTION_NEXT_AT_MOVEMENT_GRANULARITY, AccessibilityNodeInfo.ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY ->
                arrayOf(SettingsUiSelection.TRAVERSE, int(AccessibilityNodeInfo.ACTION_ARGUMENT_MOVEMENT_GRANULARITY_INT),
                    flag(action == AccessibilityNodeInfo.ACTION_NEXT_AT_MOVEMENT_GRANULARITY),
                    flag(args?.getBoolean(AccessibilityNodeInfo.ACTION_ARGUMENT_EXTEND_SELECTION_BOOLEAN) == true))
            else -> return false
        })
        return true
    }
}
