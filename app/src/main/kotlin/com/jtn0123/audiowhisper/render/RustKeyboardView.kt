package com.jtn0123.audiowhisper.render

import android.content.res.Configuration
import android.content.Context
import android.os.SystemClock
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import android.util.Log
import android.graphics.fonts.SystemFonts
import android.view.Choreographer
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.accessibility.AccessibilityNodeProvider
import android.widget.FrameLayout
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.suggest.TapOffset
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.ime.nlp.ClipboardSuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.text.keyboard.TextKey
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyboard
import dev.patrickgold.florisboard.subtypeManager
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import com.jtn0123.audiowhisper.reader.ReadAloud
import dev.patrickgold.florisboard.ime.popup.GlobalStateNumPopupsShowing
import kotlinx.coroutines.flow.update
import org.florisboard.lib.android.showShortToast

private const val PROFILE_LOG_TAG = "aw-profile"

/** Font discovery uses Android's public API; XML paths are used only by the offline format probe. */
internal fun systemRendererFonts(): Array<String> {
    val fonts = SystemFonts.getAvailableFonts().mapNotNull { it.file }.distinctBy { it.path }
    return fonts.sortedWith(compareBy({ when {
        it.name == "Roboto-Regular.ttf" -> 0
        it.name.contains("Emoji") -> 1
        it.name.contains("Symbols") -> 2
        else -> 3
    } }, { it.name })).filter { it.name.startsWith("Roboto") || it.name.contains("Emoji") || it.name.contains("Symbols") || it.name == "DroidSansMono.ttf" }
        .map { it.absolutePath }.toTypedArray()
}

internal data class KeyboardSnapshot(
    val keyboard: TextKeyboard, val keys: List<TextKey>,
    val candidates: List<SuggestionCandidate>, val theme: RendererTheme, val shift: String,
    val keyboardHeight: Float, val stripHeight: Float, val rowHeight: Float, val marginH: Float, val marginV: Float, val smartbar: RendererSmartbar, val preview: Boolean,
    val panels: RendererPanels, val editorOptions: IntArray, val search: RendererSearch, val subtypes: RendererSubtypes,
)

/** A fresh native worker owns each surface generation. Destruction joins it before releasing its window. */
class RustKeyboardView(context: Context) : FrameLayout(context), SurfaceHolder.Callback, Choreographer.FrameCallback {
    private val manager by context.keyboardManager()
    private val nlp by context.nlpManager()
    private val native = KeyboardRenderer(systemRendererFonts())
    private val surface = SurfaceView(context)
    private val choreographer = Choreographer.getInstance()
    private val gestureHost = RendererGestureHost(context, native)
    private var popupSource: TextKey? = null
        set(value) {
            if ((field != null) != (value != null)) {
                GlobalStateNumPopupsShowing.update { it + if (value != null) 1 else -1 }
            }
            field = value
        }
    private var popupChoices: List<KeyData> = emptyList()
    private var snapshot: KeyboardSnapshot? = null
    private var liveSurface = false
    private var surfaceReady = false
    private var framePending = false
    private var released = false
    private var refreshEmojiHistory = false
    private val candidateBindings = RendererCandidateBindings()
    private val smartbarTouch = RendererSmartbarTouch(this, candidateBindings::get, native::tooltipShown) { pointer ->
        native.candidateLongPress(pointer)
        invalidateFrame()
    }
    private val deadlineWakeup = Runnable {
        dispatchActions(native.tick(SystemClock.uptimeNanos()))
        scheduleDeadline()
        invalidateFrame()
    }
    private val prefs by FlorisPreferenceStore
    private val accessibilityResources = rendererAccessibilityResources(context)
    private val accessibility = RendererAccessibility(this, native::semanticSnapshot, { id, action ->
        val actions = native.semanticAction(id, action)
        dispatchActions(actions)
        invalidateFrame()
        actions.isNotEmpty()
    }, { dispatchActions(native.semanticCancel()); invalidateFrame() })

    init {
        // Snygg draws the IME background in another SurfaceView at the default media layer.
        // Order this surface above that background, while keeping Compose/autofill above it.
        surface.setZOrderMediaOverlay(true)
        surface.holder.setFormat(android.graphics.PixelFormat.TRANSLUCENT)
        addView(surface, LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT))
        surface.holder.addCallback(this)
        isClickable = true
        importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES
        surface.importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_NO
    }

    private val mediaBuffers = RendererMediaBuffers(context,native) { accessibility.refresh(); scheduleDeadline(); invalidateFrame() }
    private val gifActions = RendererGifActions(context) { native.gifSent(); invalidateFrame() }
    private val clipboardActions = RendererClipboardActions(context)

    internal fun update(value: KeyboardSnapshot) {
        if (released) { trace(SurfaceEvent.REJECTED_UPDATE); return }
        if (snapshot?.keyboard !== value.keyboard) {
            trace(SurfaceEvent.KEYBOARD_REPLACED)
            cancelInput()
            native.touch(intArrayOf(MotionEvent.ACTION_CANCEL, 0), longArrayOf(), floatArrayOf())
        }
        if (snapshot?.theme != value.theme) {
            native.theme(value.theme.json, value.theme.paletteNames.toTypedArray(), value.theme.paletteColors.toIntArray(),
                floatArrayOf(value.theme.density, value.theme.fontScale, value.theme.fontMultiplier))
        }
        val changed = snapshot?.let { old ->
            old.smartbar.actions.indices.filter { i ->
                old.smartbar.actions[i] != value.smartbar.actions.getOrNull(i) ||
                    (old.smartbar.codes[i * BAR_ACTION_STRIDE + BAR_FLAGS_OFFSET] and BAR_ENABLED != 0 &&
                        value.smartbar.codes.getOrNull(i * BAR_ACTION_STRIDE + BAR_FLAGS_OFFSET)?.and(BAR_ENABLED) != BAR_ENABLED)
            }.toIntArray()
        } ?: intArrayOf()
        dispatchActions(native.barSettings(prefs.keyboard.longPressDelay.get().toLong() * 1_000_000, changed))
        if (snapshot?.panels?.kind == 3 && value.panels.kind != 3) savePanelArrangement()
        snapshot = value
        if (width > 0 && height > 0) updateScene()
    }

    private fun updateScene() {
        val s = snapshot ?: return
        val desired = TextKey(TextKeyData.UNSPECIFIED)
        s.keyboard.sizeDesiredKey(desired, width.toFloat(), s.keyboardHeight, s.rowHeight, s.marginH, s.marginV)
        s.keyboard.layout(width.toFloat(), s.keyboardHeight, desired, true)
        val geometry = FloatArray(s.keys.size * 10)
        val metadata = mutableListOf(s.keyboard.mode.toString(), s.shift)
        val labels = mutableListOf<String>()
        for ((i, key) in s.keys.withIndex()) {
            val t = key.touchBounds
            val v = key.visibleBounds
            floatArrayOf(t.left, t.top, t.right, t.bottom, v.left, v.top, v.right, v.bottom,
                if (key.isEnabled) 1f else 0f, key.computedData.code.toFloat()).copyInto(geometry, i * 10)
            metadata.add(key.computedData.type.toString())
            labels.add(key.label ?: "")
            labels.add(key.hintedLabel ?: "")
            labels.add(key.foregroundImageVector?.let(::iconPath) ?: "")
        }
        val candidates = s.candidates.map { it.text.toString() }.ifEmpty { if (s.preview) emptyList() else rendererEmojiSamples() }
        val flags = if (s.candidates.isEmpty()) IntArray(candidates.size) else s.candidates.map {
            (if (it.isEligibleForAutoCommit) CANDIDATE_AUTO_COMMIT else 0) or (if (it is ClipboardSuggestionCandidate) CANDIDATE_CLIPBOARD else 0)
        }.toIntArray()
        native.smartbar(s.smartbar.config, s.smartbar.codes, s.smartbar.labels + s.candidates.flatMap { listOf(it.icon?.let(::iconPath) ?: "", it.secondaryText?.toString() ?: "") })
        manager.smartbarVisibleDynamicActionsCount = s.smartbar.measure(width.toFloat())[2].toInt()
        native.update(floatArrayOf(width.toFloat(), height.toFloat(), s.stripHeight, desired.visibleBounds.width, desired.visibleBounds.height),
            metadata.toTypedArray(), flags, geometry, labels.toTypedArray(), candidates.toTypedArray())
        native.panels(floatArrayOf(s.panels.kind.toFloat(), s.theme.density,
            s.smartbar.config[11], manager.smartbarVisibleDynamicActionsCount.toFloat(), android.view.ViewConfiguration.getKeyRepeatDelay().toFloat() * 1_000_000,
            if (prefs.gestures.deleteKeyLongPress.get() == dev.patrickgold.florisboard.ime.text.gestures.SwipeAction.DELETE_WORD) 1f else 0f,
            android.view.ViewConfiguration.getLongPressTimeout().toFloat() * 1_000_000), s.panels.data, s.panels.labels)
        if (s.panels.kind == 7) native.subtypes(s.subtypes.selected, s.subtypes.ids, s.subtypes.labels)
        if (s.panels.kind == 5) s.panels.gif.bind(native)
        if (s.panels.kind == 4) s.panels.clipboard.bind(native)
        if (s.panels.kind == 1) { s.panels.emoji.bind(native, refreshEmojiHistory); refreshEmojiHistory = false }
        native.search(s.theme.density,s.search.labels,SystemClock.uptimeNanos())
        candidateBindings.bind(native, s.candidates)
        native.semanticConfig(accessibilityResources.first, accessibilityResources.second,
            s.smartbar.actions.map { (it as? QuickAction.InsertText)?.data ?: "" }.toTypedArray(),
            s.keys.map { if (it.computedPopups.getPopupKeys(prefs.keyboard.keyHintConfiguration()).isNotEmpty()) 1 else 0 }.toIntArray(),
            s.candidates.map { if (it.isEligibleForUserRemoval) 1 else 0 }.toIntArray(), s.smartbar.checkedCodes, s.editorOptions)
        accessibility.refresh()
        mediaBuffers.update(s.panels.clipboard.takeIf { s.panels.kind==4 },s.panels.gif.takeIf { s.panels.kind==5 })
        gestureHost.layout(s)
        attachIfReady()
        scheduleDeadline()
        invalidateFrame()
    }

    private fun attachIfReady() {
        if (released || liveSurface || !surfaceReady || snapshot == null || width <= 0 || height <= 0) return
        native.attach(surface.holder.surface, hybrid = Log.isLoggable("aw-hybrid", Log.DEBUG))
        liveSurface = true
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        if (w > 0 && h > 0 && snapshot != null && !released) updateScene()
    }

    private fun invalidateFrame() {
        if (!released && snapshot != null) accessibility.refresh()
        if (!released && liveSurface && !framePending) {
            framePending = true
            choreographer.postFrameCallback(this)
        }
    }

    override fun doFrame(frameTimeNanos: Long) {
        framePending = false
        if (!released && liveSurface) {
            val trailVisible = native.frame(frameTimeNanos)
            if (trailVisible || snapshot?.smartbar?.animating == true) invalidateFrame()
        }
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        surfaceReady = true
        trace(SurfaceEvent.CREATED)
    }
    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        trace(SurfaceEvent.CHANGED)
        surfaceReady = true
        if (released) return
        cancelInput()
        if (liveSurface) native.detach()
        liveSurface = false
        // update() also attaches when the first snapshot arrives after this callback.
        if (snapshot != null && width > 0 && height > 0) updateScene()
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        trace(SurfaceEvent.DESTROYED)
        surfaceReady = false
        liveSurface = false
        cancelInput()
        choreographer.removeFrameCallback(this)
        framePending = false
        if (!released) native.detach()
    }

    override fun dispatchTouchEvent(event: MotionEvent): Boolean = onTouchEvent(event)
    override fun getAccessibilityNodeProvider(): AccessibilityNodeProvider = accessibility
    override fun dispatchHoverEvent(event: MotionEvent): Boolean = accessibility.hover(event) || super.dispatchHoverEvent(event)

    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (released || !liveSurface) { trace(SurfaceEvent.REJECTED_TOUCH); return false }
        if (accessibility.touch(event)) return true
        val pointers = event.pointerCount
        val metadata = IntArray(2 + pointers)
        metadata[0] = event.actionMasked
        metadata[1] = event.actionIndex
        for (p in 0 until pointers) metadata[2 + p] = event.getPointerId(p)
        val times = LongArray(event.historySize + 2)
        times[0] = event.downTime * 1_000_000
        val coords = FloatArray((event.historySize + 1) * pointers * 2)
        for (h in 0..event.historySize) {
            times[h + 1] = if (h < event.historySize) event.getHistoricalEventTimeNanos(h) else event.eventTimeNanos
            for (p in 0 until pointers) {
                coords[(h * pointers + p) * 2] = if (h < event.historySize) event.getHistoricalX(p, h) else event.getX(p)
                coords[(h * pointers + p) * 2 + 1] = if (h < event.historySize) event.getHistoricalY(p, h) else event.getY(p)
            }
        }
        snapshot?.let(gestureHost::configure)
        val actions = native.touch(metadata, times, coords)
        dispatchActions(actions, event)
        scheduleDeadline()
        snapshot?.let { state -> mediaBuffers.update(state.panels.clipboard.takeIf { state.panels.kind==4 },state.panels.gif.takeIf { state.panels.kind==5 }) }
        invalidateFrame()
        if (event.actionMasked == MotionEvent.ACTION_UP) performClick()
        return true
    }

    override fun performClick(): Boolean { super.performClick(); return true }

    private fun scheduleDeadline() {
        removeCallbacks(deadlineWakeup)
        val deadline = native.nextDeadline()
        if (!released && deadline >= 0) postDelayed(deadlineWakeup,
            ((deadline - SystemClock.uptimeNanos() + 999_999) / 1_000_000).coerceAtLeast(0))
    }
    private fun dispatchActions(actions: IntArray, event: MotionEvent? = null) {
        for (a in actions.indices step 3) dispatch(actions[a], actions[a + 1], actions[a + 2], event)
    }
    private fun dispatch(action: Int, index: Int, pointer: Int, event: MotionEvent?) {
        val s = snapshot ?: return
        val dispatcher = manager.inputEventDispatcher
        if (action == PANEL_SUBTYPE) {
            if (pointer != 0) { val subtypes by context.subtypeManager(); s.subtypes.ids.getOrNull(index)?.let(subtypes::switchToSubtypeById) }
            manager.activeState.isSubtypeSelectionVisible = false
            return
        }
        if (action == PANEL_GIF) {gifActions.dispatch(index,pointer,s.panels.gif);return}
        if (action == PANEL_CLIPBOARD) {
            clipboardActions.dispatch(index,pointer,s.panels.clipboard)
            return
        }
        if (action == PANEL_CHANGED) return
        if (action == PANEL_EMOJI || action == PANEL_HISTORY) {
            refreshEmojiHistory = action == PANEL_HISTORY
            s.panels.emoji.emojis.getOrNull(index)?.let { emoji ->
                if (action == PANEL_EMOJI) dispatcher.sendDownUp(emoji)
                kotlinx.coroutines.runBlocking {
                    val helper = dev.patrickgold.florisboard.ime.media.emoji.EmojiHistoryHelper
                    when {
                        action == PANEL_EMOJI -> helper.markEmojiUsed(prefs, emoji)
                        pointer == 1 -> helper.pinEmoji(prefs, emoji)
                        pointer == 2 -> helper.unpinEmoji(prefs, emoji)
                        pointer == 7 -> helper.moveEmoji(prefs, emoji, -1)
                        pointer == 8 -> helper.moveEmoji(prefs, emoji, 1)
                        else -> {
                            helper.removeEmoji(prefs, emoji)
                            context.showShortToast(dev.patrickgold.florisboard.R.string.emoji__history__removal_success_message, "emoji" to emoji.value)
                        }
                    }
                }
            }
            return
        }
        if (action in PANEL_DOWN..PANEL_REPEAT) {
            when (action) {
                PANEL_DOWN -> (s.panels.actions.getOrNull(index) as? QuickAction.InsertKey)?.let {
                    FlorisImeService.inputFeedbackController()?.keyPress(TextKeyData.UNSPECIFIED)
                    dispatcher.sendDown(it.data, scheduleHold = false)
                }
                PANEL_UP -> when (val a = s.panels.actions.getOrNull(index)) {
                    is QuickAction.InsertKey -> dispatcher.sendUp(a.data)
                    is QuickAction.InsertText -> a.onPointerUp(context)
                    null -> Unit
                }
                PANEL_REPEAT -> manager.onInputKeyRepeat(TextKeyData(code = pointer))
                PANEL_CANCEL -> s.panels.actions.getOrNull(index)?.onPointerCancel(context)
                PANEL_MODE -> {
                    if (index == 4) {
                        com.jtn0123.audiowhisper.gif.GifSearch.showGifs(true)
                        return
                    }
                    if (s.panels.kind == 3) savePanelArrangement()
                    manager.activeState.isActionsEditorVisible = index == 3
                    manager.activeState.isActionsOverflowVisible = index == 2 || index == 3
                }
            }
            return
        }
        if (action == ACCESS_READ_TOP) { ReadAloud.read(context, ReadAloud.From.Top); return }
        if (action >= GLIDE_POINT && action != OPEN_POPUP) { gestureHost.dispatch(action, index, pointer); return }
        if (action in BAR_DOWN until GLIDE_POINT) { smartbarTouch.dispatch(action, index, pointer, s); return }
        when (action) {
            KEY_DOWN -> {
                val key = s.keys[index]
                val data = key.computedData
                key.computedDataOnDown = data
                val p = event?.findPointerIndex(pointer) ?: -1
                if (p >= 0 && event != null) {
                    val v = key.visibleBounds
                    nlp.tapTrail.pressed(data.code, TapOffset.of(event.getX(p), event.getY(p) - s.stripHeight,
                        v.center.x, v.center.y, v.width, v.height))
                }
                dispatcher.sendDown(data, onLongPress = {
                    val choices = key.computedPopups.getPopupKeys(prefs.keyboard.keyHintConfiguration()).toList()
                    val result = native.longPress(pointer, choices.isNotEmpty(), dispatcher.isUninterruptedEventSequence(data))
                    dispatchActions(result)
                    result.indices.step(3).any { result[it] == LONG_PRESS_BLOCK }
                })
                FlorisImeService.inputFeedbackController()?.keyPress(data)
            }
            KEY_UP -> {
                val key = s.keys[index]
                val down = key.computedDataOnDown
                val current = key.computedData
                if (down == current) dispatcher.sendUp(down)
                else { dispatcher.sendCancel(down); dispatcher.sendDownUp(current) }
            }
            KEY_CANCEL -> {
                dispatcher.sendCancel(s.keys[index].computedDataOnDown)
                if (popupSource === s.keys[index]) {
                    popupSource = null
                    popupChoices = emptyList()
                }
            }
            OPEN_POPUP -> {
                val key = s.keys[index]
                popupSource = key
                val choices = key.computedPopups.getPopupKeys(prefs.keyboard.keyHintConfiguration())
                popupChoices = choices.toList()
                native.popup(index, popupChoices.map { it.label }.toTypedArray(), popupChoices.map { it.code }.toIntArray())
                native.popupGrid(choices.prioritized.size, resources.configuration.orientation == Configuration.ORIENTATION_LANDSCAPE)
                invalidateFrame()
            }
            POPUP_SELECT -> {
                popupSource?.computedDataOnDown?.let(dispatcher::sendCancel)
                popupChoices.getOrNull(index)?.let(dispatcher::sendDownUp)
                popupSource = null
                popupChoices = emptyList()
            }
            CANDIDATE_COMMIT -> s.candidates.getOrNull(index)?.let(manager::commitCandidate)
        }
    }

    private fun savePanelArrangement() {
        val s = snapshot ?: return
        val a = native.panelArrangement()
        kotlinx.coroutines.runBlocking {
            prefs.smartbar.actionArrangement.set(dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionArrangement(
                s.panels.actions.getOrNull(a[0][0]), a[1].map { s.panels.actions[it] }, a[2].map { s.panels.actions[it] }))
        }
    }

    private fun cancelInput() {
        accessibility.clear()
        dispatchActions(native.semanticCancel())
        removeCallbacks(deadlineWakeup)
        dispatchActions(native.touch(intArrayOf(MotionEvent.ACTION_CANCEL, 0), longArrayOf(), floatArrayOf()))
        smartbarTouch.cancel()
        candidateBindings.clear()
        popupSource = null
        popupChoices = emptyList()
    }

    private enum class SurfaceEvent(val code: Int) {
        CREATED(2), CHANGED(3), DESTROYED(4), RELEASE(5), ATTACHED(6), DETACHED(7),
        REJECTED_TOUCH(10), REJECTED_UPDATE(11), KEYBOARD_REPLACED(12),
    }

    private fun trace(event: SurfaceEvent) {
        if (Log.isLoggable(PROFILE_LOG_TAG, Log.DEBUG)) {
            Log.d(PROFILE_LOG_TAG, "view event=${event.code} id=${System.identityHashCode(this)} live=${if (liveSurface) 1 else 0} released=${if (released) 1 else 0}")
        }
    }

    override fun onLayout(changed: Boolean, left: Int, top: Int, right: Int, bottom: Int) {
        super.onLayout(changed, left, top, right, bottom)
        if (!released && !surface.holder.surface.isValid) {
            // A newly inserted AndroidView needs another traversal for SurfaceView's pre-draw.
            surface.postInvalidateOnAnimation()
        }
    }

    override fun onAttachedToWindow() { super.onAttachedToWindow(); trace(SurfaceEvent.ATTACHED) }
    override fun onDetachedFromWindow() { super.onDetachedFromWindow(); trace(SurfaceEvent.DETACHED) }

    fun release() {
        mediaBuffers.close()
        gifActions.close()
        trace(SurfaceEvent.RELEASE)
        if (released) return
        if (snapshot?.panels?.kind == 3) savePanelArrangement()
        cancelInput()
        accessibility.release()
        liveSurface = false
        choreographer.removeFrameCallback(this)
        surface.holder.removeCallback(this)
        native.close()
        surfaceReady = false
        released = true
    }
}
