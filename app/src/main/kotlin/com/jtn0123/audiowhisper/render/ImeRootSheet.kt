package com.jtn0123.audiowhisper.render

import android.content.Context
import android.view.WindowInsets
import android.widget.FrameLayout
import androidx.core.graphics.Insets
import androidx.core.view.WindowInsetsCompat
import dev.patrickgold.florisboard.FlorisImeService

/** Compose's safe drawing insets: system bars, the IME and cutouts, each side's largest. */
internal fun safeDrawing(insets: WindowInsets, view: android.view.View): Insets =
    WindowInsetsCompat.toWindowInsetsCompat(insets, view).getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.ime() or WindowInsetsCompat.Type.displayCutout())

/**
 * The subtype sheet over the whole input method window while the Rust renderer is on: a Rust
 * chrome view, there only while the keyboard's state shows the sheet, inside the safe area.
 */
internal class ImeSubtypeSheet(context: Context) : FrameLayout(context) {
    private val service = context as FlorisImeService
    private val reads = RootReads(service)
    private val watch = RootWatch(::rebuild)
    private var sheet: RustChromeView? = null
    private var safe = Insets.NONE

    init {
        layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT)
        isHapticFeedbackEnabled = true
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        val prefs = reads.prefs.keyboard
        reads.scope = watch.start(listOf(reads.keyboard.activeState.flow, reads.themes.activeThemeInfo,
            reads.subtypes.subtypesFlow, reads.subtypes.activeSubtypeFlow) + listOf(reads.prefs.theme.accentColor,
            prefs.fontSizeMultiplierPortrait, prefs.fontSizeMultiplierLandscape, prefs.oneHandedModeEnabled,
            prefs.oneHandedModeScaleFactor).map { it.asFlow() })
        watch.request()
    }

    override fun onDetachedFromWindow() {
        watch.stop()
        sheet?.let(::removeView)
        sheet = null
        super.onDetachedFromWindow()
    }

    override fun onApplyWindowInsets(insets: WindowInsets): WindowInsets {
        val next = safeDrawing(insets, this)
        if (next != safe) {
            safe = next
            setPadding(next.left, next.top, next.right, 0)
            watch.request()
        }
        return super.onApplyWindowInsets(insets)
    }

    private fun rebuild() {
        val state = reads.keyboard.activeState
        if (!state.isSubtypeSelectionVisible) {
            sheet?.let(::removeView)
            sheet = null
            return
        }
        val data = rendererSubtypes(reads)
        val view = sheet ?: RustChromeView(service).also { addView(it); sheet = it }
        view.subtypeAction = { index ->
            index?.let { data.ids.getOrNull(it) }?.let(reads.subtypes::switchToSubtypeById)
            state.isSubtypeSelectionVisible = false
        }
        view.updateSubtypes(reads.theme(stateful = true), data, safe.bottom.toFloat())
    }
}
