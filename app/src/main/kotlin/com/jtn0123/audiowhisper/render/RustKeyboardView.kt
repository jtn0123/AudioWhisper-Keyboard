package com.jtn0123.audiowhisper.render

import android.content.Context
import android.util.Log
import android.graphics.fonts.SystemFonts
import android.view.Choreographer
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.widget.FrameLayout
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.suggest.TapOffset
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.nlp.ClipboardSuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.text.keyboard.TextKey
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyboard
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.nlpManager

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
)

/** A fresh native worker owns each surface generation. Destruction joins it before releasing its window. */
class RustKeyboardView(context: Context) : FrameLayout(context), SurfaceHolder.Callback, Choreographer.FrameCallback {
    private val manager by context.keyboardManager()
    private val nlp by context.nlpManager()
    private val native = KeyboardRenderer(systemRendererFonts())
    private val surface = SurfaceView(context)
    private val choreographer = Choreographer.getInstance()
    private val pressed = mutableMapOf<Int, KeyData>()
    private val popupChoices = mutableMapOf<Int, List<KeyData>>()
    private var snapshot: KeyboardSnapshot? = null
    private var liveSurface = false
    private var surfaceReady = false
    private var framePending = false
    private var released = false
    private val smartbarTouch = RendererSmartbarTouch(this) { pointer ->
        native.candidateLongPress(pointer)
        invalidateFrame()
    }
    private val prefs by FlorisPreferenceStore

    init {
        // Snygg draws the IME background in another SurfaceView at the default media layer.
        // Order this surface above that background, while keeping Compose/autofill above it.
        surface.setZOrderMediaOverlay(true)
        addView(surface, LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT))
        surface.holder.addCallback(this)
        isClickable = true
    }

    internal fun update(value: KeyboardSnapshot) {
        if (released) { trace(SurfaceEvent.REJECTED_UPDATE); return }
        if (snapshot?.keyboard !== value.keyboard) {
            cancelInput()
            native.touch(intArrayOf(MotionEvent.ACTION_CANCEL, 0), longArrayOf(), floatArrayOf())
        }
        if (snapshot?.theme != value.theme) {
            native.theme(value.theme.json, value.theme.paletteNames.toTypedArray(), value.theme.paletteColors.toIntArray(),
                floatArrayOf(value.theme.density, value.theme.fontScale, value.theme.fontMultiplier))
        }
        smartbarTouch.sync(value)
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
            (if (it.isEligibleForAutoCommit) 1 else 0) or (if (it is ClipboardSuggestionCandidate) 2 else 0)
        }.toIntArray()
        native.smartbar(s.smartbar.config, s.smartbar.codes, s.smartbar.labels + s.candidates.flatMap { listOf(it.icon?.let(::iconPath) ?: "", it.secondaryText?.toString() ?: "") })
        manager.smartbarVisibleDynamicActionsCount = s.smartbar.measure(width.toFloat())[2].toInt()
        native.update(floatArrayOf(width.toFloat(), height.toFloat(), s.stripHeight),
            metadata.toTypedArray(), flags, geometry, labels.toTypedArray(), candidates.toTypedArray())
        attachIfReady()
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
        if (!released && liveSurface && !framePending) {
            framePending = true
            choreographer.postFrameCallback(this)
        }
    }

    override fun doFrame(frameTimeNanos: Long) {
        framePending = false
        if (!released && liveSurface) {
            native.frame(frameTimeNanos)
            if (snapshot?.smartbar?.animating == true) invalidateFrame()
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

    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (released || !liveSurface) { trace(SurfaceEvent.REJECTED_TOUCH); return false }
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
        val actions = native.touch(metadata, times, coords)
        for (a in actions.indices step 3) dispatch(actions[a], actions[a + 1], actions[a + 2], event)
        invalidateFrame()
        if (event.actionMasked == MotionEvent.ACTION_UP) performClick()
        return true
    }

    override fun performClick(): Boolean { super.performClick(); return true }

    private fun dispatch(action: Int, index: Int, pointer: Int, event: MotionEvent) {
        val s = snapshot ?: return
        val dispatcher = manager.inputEventDispatcher
        if (action >= 5) { smartbarTouch.dispatch(action, index, pointer, s); return }
        when (action) {
            0 -> {
                val key = s.keys[index]
                val data = key.computedData
                pressed[pointer] = data
                val p = event.findPointerIndex(pointer)
                if (p >= 0) {
                    val v = key.visibleBounds
                    nlp.tapTrail.pressed(data.code, TapOffset.of(event.getX(p), event.getY(p) - s.stripHeight,
                        v.center.x, v.center.y, v.width, v.height))
                }
                dispatcher.sendDown(data, onLongPress = {
                    val choices = key.computedPopups.getPopupKeys(prefs.keyboard.keyHintConfiguration()).toList()
                    if (choices.isNotEmpty() && pressed[pointer] == data && !released) {
                        popupChoices[pointer] = choices
                        native.popup(index, choices.map { it.label }.toTypedArray(), choices.map { it.code }.toIntArray())
                        invalidateFrame()
                        true
                    } else false
                })
            }
            1 -> pressed.remove(pointer)?.let { down ->
                val current = s.keys[index].computedData
                if (down == current) dispatcher.sendUp(down)
                else { dispatcher.sendCancel(down); dispatcher.sendDownUp(current) }
                popupChoices.remove(pointer)
            }
            2 -> { pressed.remove(pointer)?.let(dispatcher::sendCancel); popupChoices.remove(pointer) }
            3 -> {
                pressed.remove(pointer)?.let(dispatcher::sendCancel)
                popupChoices.remove(pointer)?.getOrNull(index)?.let(dispatcher::sendDownUp)
            }
            4 -> s.candidates.getOrNull(index)?.let(manager::commitCandidate)
        }
    }

    private fun cancelInput() {
        smartbarTouch.cancel()
        for (data in pressed.values) manager.inputEventDispatcher.sendCancel(data)
        pressed.clear()
        popupChoices.clear()
    }

    private enum class SurfaceEvent(val code: Int) {
        CREATED(2), CHANGED(3), DESTROYED(4), RELEASE(5), ATTACHED(6), DETACHED(7),
        REJECTED_TOUCH(10), REJECTED_UPDATE(11),
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
        trace(SurfaceEvent.RELEASE)
        if (released) return
        cancelInput()
        liveSurface = false
        choreographer.removeFrameCallback(this)
        surface.holder.removeCallback(this)
        native.close()
        surfaceReady = false
        released = true
    }
}
