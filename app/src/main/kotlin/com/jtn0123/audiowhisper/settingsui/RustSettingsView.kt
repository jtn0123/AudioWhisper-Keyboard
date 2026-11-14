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

/** Surface, lifecycle and Android transport. Native code supplies every drawn control and action. */
internal class RustSettingsView(context: Context) : FrameLayout(context), SurfaceHolder.Callback, Choreographer.FrameCallback {
    private lateinit var activity: FlorisAppActivity
    private lateinit var platform: SettingsPlatform
    private var fallback: (String) -> Unit = {}
    private val native = KeyboardRenderer(systemRendererFonts())
    private val surface = SurfaceView(context)
    private val endpoint = SettingsInputView(context, this)
    private val choreographer = Choreographer.getInstance()
    private val imm = context.getSystemService(InputMethodManager::class.java)
    private var state = Json.parseToJsonElement(native.settings(1, settingsResources(context) + settingsIcons())[0]).jsonObject
    private var live = false
    private var pending = false
    private var released = false
    private var imeWasVisible = false
    internal var editorText = ""
        private set
    internal val anchor get() = state.getValue("editor").jsonObject.getValue("anchor").jsonPrimitive.int
    internal val selectionEnd get() = state.getValue("editor").jsonObject.getValue("focus").jsonPrimitive.int
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
    fun bind(activity: FlorisAppActivity, prefs: FlorisPreferenceModel, restored: Array<String>?, fallback: (String) -> Unit) {
        check(!::platform.isInitialized)
        this.activity = activity
        this.platform = SettingsPlatform(activity, prefs, ::preferencesPersisted)
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
        platform.enqueue(effects.filter { it[0] == "write" || it[0] == "reset" })
        for (p in effects) {
            when (p[0]) {
                "compose" -> fallback(p[1])
                "finish" -> activity.finish()
                "enable" -> InputMethodUtils.showImeEnablerActivity(activity)
                "picker" -> InputMethodUtils.showImePicker(activity)
                "editor" -> { endpoint.requestFocus(); activity.window.insetsController?.show(android.view.WindowInsets.Type.ime()) }
                "hideEditor" -> { endpoint.clearFocus(); imm.hideSoftInputFromWindow(endpoint.windowToken, 0) }
            }
        }
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
            colors.map { it.toUInt().toString() }.toTypedArray() + platform.facts()
        receive(native.settings(3, config))
    }
    private fun requestFrame() {
        if (live && !pending && !released) { pending = true; choreographer.postFrameCallback(this) }
    }
    override fun doFrame(frameTimeNanos: Long) {
        pending = false
        if (!live || released) return
        if (state.getValue("animating").jsonPrimitive.boolean) receive(native.settings(8, arrayOf(frameTimeNanos.toString())))
        native.frame(frameTimeNanos)
        if (state.getValue("animating").jsonPrimitive.boolean) requestFrame()
    }
    override fun surfaceCreated(holder: SurfaceHolder) = Unit
    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        configure(); native.attach(holder.surface); live = true; requestFrame()
    }
    override fun surfaceDestroyed(holder: SurfaceHolder) {
        live = false; pending = false; choreographer.removeFrameCallback(this); native.detach()
    }
    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        if (state.getValue("editor").jsonObject.getValue("focused").jsonPrimitive.boolean) endpoint.post { endpoint.requestFocus(); imm.restartInput(endpoint) }
    }
    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) { super.onSizeChanged(w,h,oldw,oldh); configure() }
    override fun onInterceptTouchEvent(event: MotionEvent) = true
    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (accessibility.touch(event)) return true
        val p = event.rendererPacket(); receive(native.settingsTouch(p.metadata, p.times, p.coords))
        if (event.actionMasked == MotionEvent.ACTION_UP) performClick()
        return true
    }
    override fun dispatchHoverEvent(event: MotionEvent) = accessibility.hover(event) || super.dispatchHoverEvent(event)
    override fun getAccessibilityNodeProvider() = accessibility
    override fun performClick(): Boolean { super.performClick(); return true }
    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow(); released = true; live = false; choreographer.removeFrameCallback(this)
        accessibility.release(); native.detach(); native.close()
    }
}
