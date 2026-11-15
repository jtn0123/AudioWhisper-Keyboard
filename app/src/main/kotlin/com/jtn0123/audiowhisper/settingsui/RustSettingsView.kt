package com.jtn0123.audiowhisper.settingsui

import android.content.Context
import android.view.Choreographer
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.accessibility.AccessibilityNodeInfo
import android.view.inputmethod.InputMethodManager
import android.widget.FrameLayout
import androidx.core.view.SoftwareKeyboardControllerCompat
import androidx.lifecycle.lifecycleScope
import com.jtn0123.audiowhisper.render.RendererAccessibility
import com.jtn0123.audiowhisper.render.rendererPacket
import com.jtn0123.audiowhisper.render.systemRendererFonts
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.app.FlorisAppActivity
import dev.patrickgold.florisboard.app.FlorisPreferenceModel
import dev.patrickgold.florisboard.lib.util.InputMethodUtils
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.launch
import kotlinx.serialization.json.*

private const val SETTINGS_TRACE_TAG = "aw-profile"
private const val UNBUFFERED_TAG = "aw-unbuffered"

/** Surface, lifecycle and Android transport. Native code supplies every drawn control and action. */
internal class RustSettingsView(context: Context) : FrameLayout(context), SurfaceHolder.Callback, Choreographer.VsyncCallback {
    private lateinit var activity: FlorisAppActivity
    private lateinit var platform: SettingsPlatform
    private lateinit var facts: SettingsFacts
    var backAvailability: (Boolean) -> Unit = {}
    private var fallback: (String, List<String>) -> Unit = { _, _ -> }
    private val native = KeyboardRenderer(systemRendererFonts())
    /** The routes native code draws; a Compose fallback hands any of them back. */
    internal val nativeRoutes: List<String> = native.settings(19)[2].split("\n")
    private val surface = SurfaceView(context)
    private val endpoint = SettingsInputView(context, this)
    private val selectionMenu = SettingsSelectionMenu(endpoint, this)
    internal val pendingWrites get() = if (::platform.isInitialized) platform.pending else 0
    internal val inputHasFocus get() = endpoint.hasFocus()
    var restoreInputFocus = false
    private val choreographer = Choreographer.getInstance()
    private val imm = context.getSystemService(InputMethodManager::class.java)
    private var state = Json.parseToJsonElement(native.settings(1,
        settingsResources(context, native.settings(16)[2].split("\n")) + settingsIcons(context.resources))[0]).jsonObject
    private var live = false
    private var pending = false
    private var released = false
    private var imeWasVisible = false
    internal var editorText = ""
        private set
    internal val anchor get() = state.getValue("editor").jsonObject.getValue("anchor").jsonPrimitive.int
    internal val selectionEnd get() = state.getValue("editor").jsonObject.getValue("focus").jsonPrimitive.int
    internal val multiLine get() = state.getValue("editor").jsonObject["multiLine"]?.jsonPrimitive?.boolean == true
    private val accessibility = RendererAccessibility(this, { state.toString() }, { id, action ->
        if (action == 4 || action == 5) send(17, id.toString(), action.toString()) else send(5, id.toString())
        true
    }, { send(12) }, hoverCommits = false, platformAction = { id, action, args ->
        when (action) {
            AccessibilityNodeInfo.ACTION_SET_TEXT -> {
                edit("set", args?.getCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE)?.toString().orEmpty())
            }
            AccessibilityNodeInfo.ACTION_SET_SELECTION -> {
                edit("select", (args?.getInt(AccessibilityNodeInfo.ACTION_ARGUMENT_SELECTION_START_INT) ?: 0).toString(),
                    (args?.getInt(AccessibilityNodeInfo.ACTION_ARGUMENT_SELECTION_END_INT) ?: 0).toString())
            }
            AccessibilityNodeInfo.AccessibilityAction.ACTION_SET_PROGRESS.id -> {
                send(15, id.toString(), (args?.getFloat(AccessibilityNodeInfo.ACTION_ARGUMENT_PROGRESS_VALUE) ?: 0f).toString()); true
            }
            else -> false
        }
    })
    init {
        isClickable = true; isFocusable = true; isFocusableInTouchMode = true
        importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES
        surface.importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_NO
        addView(surface, LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT))
        addView(endpoint, LayoutParams(1, 1))
        surface.holder.addCallback(this)
    }
    fun bind(activity: FlorisAppActivity, prefs: FlorisPreferenceModel, restored: Array<String>?, fallback: (String, List<String>) -> Unit) {
        check(!::platform.isInitialized)
        this.activity = activity
        this.platform = SettingsPlatform(activity, prefs, ::preferencesPersisted)
        this.facts = SettingsFacts(activity).also { it.observe { if (!released) pushFacts() } }
        this.fallback = fallback
        receive(native.settings(2, platform.snapshot()))
        if (restored != null) receive(native.settings(11, restored))
        for (pref in prefs.declaredPreferenceEntries.values) activity.lifecycleScope.launch {
            pref.asFlow().drop(1).collect {
                if (!released && platform.pending == 0) { receive(native.settings(2, platform.snapshot())); configure() }
            }
        }
    }
    private fun preferencesPersisted() { if (!released) { receive(native.settings(2, platform.snapshot())); configure() } }
    fun navigate(route: String) = send(4, route)
    fun save(): Array<String> = if (released) emptyArray() else native.settings(10).drop(3).toTypedArray()
    fun back(phase: Int, progress: Float = 0f) = send(6, phase.toString(), progress.toString())
    internal fun edit(vararg input: String): Boolean {
        if (released) return false
        send(9, *input); return state.getValue("accepted").jsonPrimitive.boolean
    }
    internal fun query(kind: String, length: Int): String = if (released) "" else native.settings(9, arrayOf("query", kind, length.toString()))[2]
    internal fun forwardKey(event: KeyEvent): Boolean {
        send(7, event.action.toString(), event.keyCode.toString(), event.unicodeChar.toString(), event.isShiftPressed.toString())
        return true
    }
    override fun dispatchKeyEvent(event: KeyEvent): Boolean =
        if (event.keyCode == KeyEvent.KEYCODE_BACK) super.dispatchKeyEvent(event) else forwardKey(event)
    private fun send(operation: Int, vararg input: String) {
        if (!released) receive(native.settings(operation, arrayOf(*input)))
    }
    private fun receive(reply: Array<String>) {
        state = Json.parseToJsonElement(reply[0]).jsonObject
        val barColor = state.getValue("systemBarColor").jsonPrimitive.long.toInt()
        activity.window.decorView.setBackgroundColor(barColor)
        (parent as? android.view.View)?.setBackgroundColor(barColor)
        // Like FlorisAppTheme, only the status bar: Compose never sets the navigation bar's
        // appearance, so the system tints its buttons against what is behind them.
        androidx.core.view.WindowCompat.getInsetsController(activity.window, activity.window.decorView)
            .isAppearanceLightStatusBars = state.getValue("lightSystemBars").jsonPrimitive.boolean
        backAvailability(state.getValue("canBack").jsonPrimitive.boolean)
        editorText = reply[1]
        val editor = state.getValue("editor").jsonObject
        if (!editor.getValue("focused").jsonPrimitive.boolean && endpoint.hasFocus()) endpoint.clearFocus()
        val bounds = editor["bounds"]?.takeUnless { it == JsonNull }?.jsonArray
        if (bounds != null) endpoint.layoutParams = LayoutParams(
            (bounds[2].jsonPrimitive.float - bounds[0].jsonPrimitive.float).toInt().coerceAtLeast(1),
            (bounds[3].jsonPrimitive.float - bounds[1].jsonPrimitive.float).toInt().coerceAtLeast(1),
        ).apply { leftMargin = bounds[0].jsonPrimitive.float.toInt(); topMargin = bounds[1].jsonPrimitive.float.toInt() }
        val composing = editor["composing"]?.takeUnless { it == JsonNull }?.jsonArray
        imm.updateSelection(endpoint, anchor, selectionEnd, composing?.get(0)?.jsonPrimitive?.int ?: -1, composing?.get(1)?.jsonPrimitive?.int ?: -1)
        val effects = state.getValue("effects").jsonArray.map { packet -> packet.jsonArray.map { it.jsonPrimitive.content } }
        platform.enqueue(effects.filter { it[0] == "write" || it[0] == "reset" || it[0] == "deleteEmojiHistory" })
        for (p in effects) {
            when (p[0]) {
                "compose" -> fallback(p[1], p[2].split("|"))
                "finish" -> activity.finish()
                "vibrate" -> platform.vibrate(p[1].toInt(), p[2].toInt())
                "enable" -> InputMethodUtils.showImeEnablerActivity(activity)
                "picker" -> InputMethodUtils.showImePicker(activity)
                // Compose's text field shows the keyboard this way: the insets request alone is
                // dropped when no IME is attached yet, and is not retried.
                "editor" -> { endpoint.requestFocus(); SoftwareKeyboardControllerCompat(endpoint).show() }
                "hideEditor" -> { endpoint.clearFocus(); imm.hideSoftInputFromWindow(endpoint.windowToken, 0) }
                "restartInput" -> imm.restartInput(endpoint)
                "checkModels" -> facts.checkModels { if (!released) pushFacts() }
                "url", "intent", "copy", "forgetLearnedWords" -> facts.execute(p)
            }
        }
        selectionMenu.update(state.getValue("selectionToolbar").jsonPrimitive.boolean)
        accessibility.refresh(); requestFrame()
    }
    fun refreshPlatform() = configure()
    fun imeVisibility(visible: Boolean) {
        if (imeWasVisible && !visible) send(9, "blur")
        imeWasVisible = visible
    }
    private fun configure() {
        if (width <= 0 || height <= 0 || released || !::platform.isInitialized) return
        val metrics = resources.displayMetrics
        val colors = platform.palette()
        (parent as? android.view.View)?.setBackgroundColor(colors[0])
        val config = arrayOf(width.toString(), height.toString(), metrics.density.toString(), resources.configuration.fontScale.toString()) +
            colors.take(6).map { it.toUInt().toString() }.toTypedArray() + platform.facts() +
            colors.drop(6).take(6).map { it.toUInt().toString() }.toTypedArray() + (0..32).map {
                android.util.TypedValue.applyDimension(android.util.TypedValue.COMPLEX_UNIT_SP, it.toFloat(), metrics).toString()
            }.toTypedArray() + arrayOf(platform.dialogWindowWidth().toString(), colors[12].toUInt().toString()) + platform.vibratorFacts() +
            colors[13].toUInt().toString()
        receive(native.settings(3, config))
        pushFacts()
    }
    private fun pushFacts() {
        facts.images { name, w, h, pixels -> receive(native.settingsImage(name, w, h, pixels)) }
        receive(native.settings(20, facts.pairs()))
    }
    private fun requestFrame() {
        if (live && !pending && !released) { pending = true; choreographer.postVsyncCallback(this) }
    }
    /** The vsync callback (Java's AChoreographer_postVsyncCallback), so the frame carries its vsync id. */
    override fun onVsync(data: Choreographer.FrameData) {
        val frameTimeNanos = data.frameTimeNanos
        android.os.Trace.beginSection("aw.settings.frame")
        try {
        pending = false
        if (!live || released) return
        if (state.getValue("animating").jsonPrimitive.boolean) receive(native.settings(8, arrayOf(frameTimeNanos.toString())))
        if (android.util.Log.isLoggable(SETTINGS_TRACE_TAG, android.util.Log.DEBUG)) android.util.Log.d(SETTINGS_TRACE_TAG, "submit frame_ns=$frameTimeNanos submit_ns=${System.nanoTime()}")
        native.frame(frameTimeNanos, data.preferredFrameTimeline.vsyncId)
        if (state.getValue("animating").jsonPrimitive.boolean) requestFrame()
        } finally { android.os.Trace.endSection() }
    }
    override fun surfaceCreated(holder: SurfaceHolder) = Unit
    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        // The hybrid GPU path is for measurement only, as on the keyboard.
        configure(); native.attach(holder.surface, hybrid = android.util.Log.isLoggable("aw-hybrid", android.util.Log.DEBUG), timeline = true)
        live = true; requestFrame()
    }
    override fun surfaceDestroyed(holder: SurfaceHolder) {
        live = false; pending = false; choreographer.removeVsyncCallback(this); native.detach()
    }
    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        if (restoreInputFocus) endpoint.post { endpoint.requestFocus(); imm.restartInput(endpoint) }
    }
    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) { super.onSizeChanged(w,h,oldw,oldh); configure() }
    override fun onInterceptTouchEvent(event: MotionEvent) = true
    override fun onTouchEvent(event: MotionEvent): Boolean {
        // Batched, as Compose gets them: Android resamples a batched drag to the frame time, so
        // only the same events make the same gesture scroll as far. Unbuffered (measurement only)
        // arrives sooner but lands a fling up to 9 px from Compose's (stage4-followup3.md).
        if (event.actionMasked == MotionEvent.ACTION_DOWN && android.util.Log.isLoggable(UNBUFFERED_TAG, android.util.Log.DEBUG)) requestUnbufferedDispatch(event)
        if (accessibility.touch(event)) return true
        android.os.Trace.beginSection("aw.settings.input")
        try {
        val delivered = System.nanoTime()
        val p = event.rendererPacket(); receive(native.settingsTouch(p.metadata, p.times, p.coords))
        if (android.util.Log.isLoggable(SETTINGS_TRACE_TAG, android.util.Log.DEBUG)) {
            android.util.Log.d(SETTINGS_TRACE_TAG, "input event_ns=${event.eventTimeNanos} delivered_ns=$delivered reply_ns=${System.nanoTime()} action=${event.actionMasked}")
        }
        if (event.actionMasked == MotionEvent.ACTION_UP) performClick()
        return true
        } finally { android.os.Trace.endSection() }
    }
    override fun dispatchHoverEvent(event: MotionEvent) = accessibility.hover(event) || super.dispatchHoverEvent(event)
    override fun getAccessibilityNodeProvider() = accessibility
    override fun performClick(): Boolean { super.performClick(); return true }
    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow(); released = true; live = false; choreographer.removeVsyncCallback(this)
        if (::facts.isInitialized) facts.release()
        selectionMenu.close(); accessibility.release(); native.detach(); native.close()
    }
}
