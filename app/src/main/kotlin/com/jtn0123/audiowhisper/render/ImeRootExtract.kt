package com.jtn0123.audiowhisper.render

import android.inputmethodservice.ExtractEditText
import android.view.Gravity
import android.widget.FrameLayout
import android.widget.LinearLayout
import dev.patrickgold.florisboard.FlorisImeService

/**
 * The full-screen editor while the Rust renderer is on: Android's own [ExtractEditText], which
 * the input method protocol and accessibility need, inside a Rust chrome view with its action.
 */
internal class ImeExtractView(private val service: FlorisImeService, editor: ExtractEditText) : FrameLayout(service) {
    private val reads = RootReads(service)
    private val watch = RootWatch(::rebuild)
    private val view: RustExtractView

    init {
        isHapticFeedbackEnabled = true
        layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT)
        editor.id = android.R.id.inputExtractEditText
        editor.layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT)
        editor.background = null
        editor.gravity = Gravity.TOP
        editor.isVerticalScrollBarEnabled = true
        view = RustExtractView(service, editor)
        addView(view)
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        // Android's extract layout pads and margins the editor's frame; the chrome draws its own.
        (parent as? LinearLayout)?.let { frame ->
            frame.layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT).also { it.setMargins(0, 0, 0, 0) }
            frame.setPadding(0, 0, 0, 0)
        }
        val prefs = reads.prefs.keyboard
        reads.scope = watch.start(listOf(reads.editor.activeInfoFlow, reads.keyboard.activeState.flow,
            reads.themes.activeThemeInfo) + listOf(reads.prefs.theme.accentColor, prefs.fontSizeMultiplierPortrait,
            prefs.fontSizeMultiplierLandscape, prefs.oneHandedModeEnabled, prefs.oneHandedModeScaleFactor).map { it.asFlow() })
        watch.request()
    }

    override fun onDetachedFromWindow() {
        watch.stop()
        super.onDetachedFromWindow()
    }

    private fun rebuild() {
        val info = reads.editor.activeInfoFlow.value
        val label = info.extractedActionLabel ?: service.getTextForImeAction(info.imeOptions.action.toInt()) ?: "ACTION"
        view.update(reads.theme(stateful = true), label) {
            if (info.extractedActionId != 0) service.currentInputConnection?.performEditorAction(info.extractedActionId)
            else reads.editor.performEnterAction(info.imeOptions.action)
        }
    }
}
