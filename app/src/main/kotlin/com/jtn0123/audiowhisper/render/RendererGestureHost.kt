package com.jtn0123.audiowhisper.render

import android.content.Context
import android.content.res.Resources
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.gif.GifSearchKeys
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.glideTypingManager
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.ime.editor.OperationScope
import dev.patrickgold.florisboard.ime.editor.OperationUnit
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.handleArrow
import dev.patrickgold.florisboard.ime.text.gestures.GlideTypingGesture
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData

/** Asks the renderer for each glide point's time, which the swipe scorer weighs. */
private const val GLIDE_TIMES = 1f

/** Android editor and glide callbacks only. Rust chooses when and with which arguments to call. */
internal class RendererGestureHost(private val context: Context, private val native: KeyboardRenderer) {
    private val prefs by FlorisPreferenceStore
    private val manager by context.keyboardManager()
    private val editor by context.editorInstance()
    private val glide by context.glideTypingManager()
    private val glideEnabled get() = prefs.glide.enabled.get() && editor.activeInfo.isRichInputEditor &&
        manager.activeState.keyVariation != KeyVariation.PASSWORD
    private fun actions() = listOf(prefs.gestures.swipeUp.get(), prefs.gestures.swipeDown.get(),
        prefs.gestures.swipeLeft.get(), prefs.gestures.swipeRight.get(), prefs.gestures.spaceBarSwipeUp.get(),
        prefs.gestures.spaceBarSwipeLeft.get(), prefs.gestures.spaceBarSwipeRight.get(),
        prefs.gestures.spaceBarLongPress.get(), prefs.gestures.deleteKeySwipeLeft.get())

    fun configure(snapshot: KeyboardSnapshot) {
        val density = Resources.getSystem().displayMetrics.density
        native.gestures(floatArrayOf(density, context.resources.getDimension(R.dimen.key_width) / density,
            if (glideEnabled && snapshot.keyboard.mode == KeyboardMode.CHARACTERS) 1f else 0f,
            if (editor.activeInfo.isRawInputEditor) 1f else 0f, if (manager.isTypingIntoGifSearch) 1f else 0f,
            if (prefs.glide.showTrail.get()) 1f else 0f, prefs.glide.trailDuration.get().toFloat(),
            prefs.gestures.swipeDistanceThreshold.get().toFloat(), prefs.gestures.swipeVelocityThreshold.get().toFloat(),
            if (manager.inputEventDispatcher.isPressed(KeyCode.SHIFT)) 1f else 0f, GLIDE_TIMES,
            if (prefs.keyboard.popupEnabled.get()) 1f else 0f),
            actions().map { it.name }.toTypedArray())
    }
    fun layout(snapshot: KeyboardSnapshot) {
        configure(snapshot)
        if (glideEnabled && snapshot.keyboard.mode == KeyboardMode.CHARACTERS) {
            // The keys' own subtype: a scene can still show the last one's keys after a switch.
            glide.setLayout(snapshot.keys, snapshot.keyboard.subtype)
        }
    }
    /** The time of the glide point that comes next: the renderer sends it just before the point. */
    private var glideTime = 0

    fun dispatch(type: Int, value: Int, extra: Int) {
        val dispatcher = manager.inputEventDispatcher
        when (type) {
            GLIDE_TIME -> glideTime = value
            GLIDE_POINT -> glide.onGlideAddPoint(
                GlideTypingGesture.Detector.Position(Float.fromBits(value), Float.fromBits(extra), glideTime))
            GLIDE_COMPLETE -> glide.onGlideComplete(GlideTypingGesture.Detector.PointerData(mutableListOf(), 0, true))
            GLIDE_CANCEL -> glide.onGlideCancelled()
            SWIPE_ACTION -> manager.executeSwipeAction(actions()[value])
            CURSOR_LEFT -> manager.handleArrow(KeyCode.ARROW_LEFT, value)
            CURSOR_RIGHT -> manager.handleArrow(KeyCode.ARROW_RIGHT, value)
            MASS_BEGIN -> editor.massSelection.begin()
            MASS_END -> editor.massSelection.end()
            SELECT_CHARACTERS, SELECT_WORDS -> if (editor.activeContent.selection.isValid) editor.setSelectionSurrounding(
                n = value, unit = if (type == SELECT_WORDS) OperationUnit.WORDS else OperationUnit.CHARACTERS,
                scope = if (extra == 0) OperationScope.BEFORE_CURSOR else OperationScope.AFTER_CURSOR)
            DELETE_SELECTION -> if (editor.activeContent.selection.isSelectionMode) editor.deleteBackwards(OperationUnit.CHARACTERS)
            GIF_DELETE_CHARACTERS, GIF_DELETE_WORDS -> GifSearchKeys.swipeDelete(prefs.gestures.deleteKeySwipeLeft.get(), value)?.let {
                GifSearch.deleteSwiped(it, extra != 0)
            }
            GESTURE_FEEDBACK -> FlorisImeService.inputFeedbackController()?.gestureMovingSwipe(
                if (value == KeyCode.DELETE) TextKeyData.DELETE else TextKeyData.SPACE)
            CAPS_LOCK -> dispatcher.sendDownUp(TextKeyData.CAPS_LOCK)
            INPUT_METHOD_PICKER -> dispatcher.sendDownUp(TextKeyData.SYSTEM_INPUT_METHOD_PICKER)
            KEY_CANCEL_CODE -> dispatcher.sendCancel(TextKeyData.SHIFT)
            KEY_FEEDBACK -> FlorisImeService.inputFeedbackController()?.keyPress(TextKeyData.UNSPECIFIED)
            LONG_PRESS_FEEDBACK -> FlorisImeService.inputFeedbackController()?.keyLongPress(
                if (value == KeyCode.SHIFT) TextKeyData.SHIFT else TextKeyData.UNSPECIFIED)
            LONG_PRESS_BLOCK -> Unit // The shared dispatcher consumes the callback's Boolean result.
        }
    }
}
