package com.jtn0123.audiowhisper.render

import android.graphics.Rect
import android.os.Bundle
import android.view.MotionEvent
import android.view.View
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityManager
import android.view.accessibility.AccessibilityNodeInfo
import android.view.accessibility.AccessibilityNodeProvider
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import kotlinx.serialization.json.*
import kotlin.math.ceil
import kotlin.math.floor

/** Android transport only: names, states, supported actions and activation policy come from Rust. */
internal class RendererAccessibility(
    private val host: View,
    private val snapshot: () -> String,
    private val activate: (Int, Int) -> Boolean,
    private val cancel: () -> Unit,
    private val hoverCommits: Boolean = true,
    private val platformAction: ((Int, Int, Bundle?) -> Boolean)? = null,
) : AccessibilityNodeProvider() {
    private val manager = host.context.getSystemService(AccessibilityManager::class.java)
    private var nodes = linkedMapOf<Int, JsonObject>()
    private var active = true
    private var focused = View.NO_ID
    private var hovered = View.NO_ID
    private var mode = ""
    private var shift = ""
    private var popup = false
    private var popupReturn = View.NO_ID
    private var down = View.NO_ID
    private var held = false
    private val hold = Runnable {
        if (down != View.NO_ID && supports(down, ACCESS_LONG_CLICK)) {
            held = activate(down, ACCESS_LONG_CLICK)
            refresh()
        }
    }
    val exploring: Boolean get() = active && manager.isEnabled && manager.isTouchExplorationEnabled

    fun refresh(force: Boolean = false) {
        if (!active || (!force && !manager.isEnabled)) return
        val data = Json.parseToJsonElement(snapshot()).jsonObject
        val next = data.getValue("nodes").jsonArray.associateByTo(linkedMapOf()) { it.jsonObject.int("id") }
            .mapValuesTo(linkedMapOf()) { it.value.jsonObject }
        val old = nodes
        val oldFocus = old[focused]
        val nextPopup = data.getValue("popup").jsonPrimitive.boolean
        if (focused != View.NO_ID && focused !in next) {
            event(focused, AccessibilityEvent.TYPE_VIEW_ACCESSIBILITY_FOCUS_CLEARED)
            focused = View.NO_ID
        }
        nodes = next
        // Keep physical-key focus across casing and modes. Enter a new popup at its selected
        // choice; when it closes choose the nearest surviving node to the previous focus.
        val popupTarget = if (nextPopup && !popup) next.entries.firstOrNull {
            it.value.string("role") == "popup item" && it.value.bool("selected")
        }?.key else null
        if (popupTarget != null) { popupReturn = focused; focus(popupTarget) }
        else if (!nextPopup && popup && popupReturn in next) { focus(popupReturn); popupReturn = View.NO_ID }
        else if (focused == View.NO_ID && oldFocus != null && next.isNotEmpty()) {
            val r = bounds(oldFocus)
            // A prediction replacement must not automatically focus/speak a new typed word.
            next.filterValues { it.string("role") != "candidate" }.minByOrNull { (_, n) ->
                val b = bounds(n)
                val dx = b.exactCenterX() - r.exactCenterX()
                val dy = b.exactCenterY() - r.exactCenterY()
                dx * dx + dy * dy
            }?.key?.let(::focus)
        }
        popup = nextPopup
        if (hovered !in next) hovered = View.NO_ID
        // Content events carry no text. Candidate updates must not speak typed words.
        for (id in (old.keys + next.keys)) if (old[id] != next[id]) {
            event(id, AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED, change = AccessibilityEvent.CONTENT_CHANGE_TYPE_UNDEFINED)
        }
        val nextMode = data.getValue("mode").jsonPrimitive.content
        val nextShift = data.getValue("shift").jsonPrimitive.content
        if (mode.isNotEmpty() && mode != nextMode) stateEvent(nextMode)
        else if (shift.isNotEmpty() && shift != nextShift) stateEvent(nextShift)
        mode = nextMode; shift = nextShift
    }

    override fun createAccessibilityNodeInfo(virtualViewId: Int): AccessibilityNodeInfo? {
        if (virtualViewId == View.NO_ID) {
            refresh(force = true)
            return AccessibilityNodeInfo(host).apply {
                host.onInitializeAccessibilityNodeInfo(this)
                if (!hoverCommits) {
                    // Settings' host is only a container; all interaction lives in its
                    // native virtual descendants. Do not expose a dead whole-screen button.
                    isClickable = false; isLongClickable = false; isFocusable = false
                    removeAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_CLICK)
                }
                nodes.keys.forEach { addChild(host, it) }
            }
        }
        val n = nodes[virtualViewId] ?: return null
        return AccessibilityNodeInfo().apply {
            setSource(host, virtualViewId)
            setParent(host)
            packageName = host.context.packageName
            className = nodeClass(n)
            text = n.string("label")
            contentDescription = n.string("description")
            stateDescription = n.string("state").ifEmpty { null }
            isEnabled = n.bool("enabled")
            isImportantForAccessibility = true
            isSelected = n.bool("selected")
            val checked = n["checked"]
            isCheckable = checked != JsonNull
            setChecked(if (checked?.jsonPrimitive?.booleanOrNull == true) AccessibilityNodeInfo.CHECKED_STATE_TRUE else AccessibilityNodeInfo.CHECKED_STATE_FALSE)
            isHeading = n.string("role") == "heading"
            val range = n["range"]?.takeUnless { it == JsonNull }?.jsonArray
            if (range != null) {
                rangeInfo = AccessibilityNodeInfo.RangeInfo(AccessibilityNodeInfo.RangeInfo.RANGE_TYPE_FLOAT,
                    range[0].jsonPrimitive.float, range[1].jsonPrimitive.float, range[2].jsonPrimitive.float)
                if (isEnabled) addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SET_PROGRESS)
            }
            if (n.string("role") == "edit") {
                isEditable = true
                n["hint"]?.jsonPrimitive?.content?.let { hintText = it; isShowingHintText = n["hintShowing"]?.jsonPrimitive?.boolean == true }
                inputType = android.text.InputType.TYPE_CLASS_TEXT
                setTextSelection(n.int("anchor"), n.int("focus"))
                addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SET_TEXT)
                addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SET_SELECTION)
            }
            isFocusable = true
            isAccessibilityFocused = focused == virtualViewId
            isVisibleToUser = host.isShown
            isTextEntryKey = n.string("role") == "key" || n.string("role") == "popup item"
            val r = bounds(n)
            val windowBounds = Rect(r)
            val windowLocation = IntArray(2)
            host.getLocationInWindow(windowLocation)
            windowBounds.offset(windowLocation[0], windowLocation[1])
            setBoundsInWindow(windowBounds)
            val location = IntArray(2)
            host.getLocationOnScreen(location)
            r.offset(location[0], location[1])
            setBoundsInScreen(r)
            addAction(if (isAccessibilityFocused) AccessibilityNodeInfo.AccessibilityAction.ACTION_CLEAR_ACCESSIBILITY_FOCUS
                else AccessibilityNodeInfo.AccessibilityAction.ACTION_ACCESSIBILITY_FOCUS)
            if (supports(virtualViewId, ACCESS_CLICK)) {
                isClickable = true
                addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_CLICK)
            }
            if (supports(virtualViewId, ACCESS_LONG_CLICK)) {
                isLongClickable = true
                addAction(AccessibilityNodeInfo.AccessibilityAction(AccessibilityNodeInfo.ACTION_LONG_CLICK,
                    n.string("longClickLabel").ifEmpty { null }))
            }
            isScrollable = supports(virtualViewId, ACCESS_SCROLL_FORWARD) || supports(virtualViewId, ACCESS_SCROLL_BACKWARD)
            if (supports(virtualViewId, ACCESS_SCROLL_FORWARD)) addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SCROLL_FORWARD)
            if (supports(virtualViewId, ACCESS_SCROLL_BACKWARD)) addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SCROLL_BACKWARD)
            if (supports(virtualViewId, ACCESS_DISMISS)) addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_DISMISS)
        }
    }

    override fun findFocus(focus: Int): AccessibilityNodeInfo? =
        if (focus == AccessibilityNodeInfo.FOCUS_ACCESSIBILITY && focused != View.NO_ID) createAccessibilityNodeInfo(focused) else null

    override fun performAction(virtualViewId: Int, action: Int, arguments: Bundle?): Boolean {
        if (virtualViewId !in nodes) return false
        when (action) {
            AccessibilityNodeInfo.ACTION_ACCESSIBILITY_FOCUS -> return focus(virtualViewId)
            AccessibilityNodeInfo.ACTION_CLEAR_ACCESSIBILITY_FOCUS -> {
                if (focused != virtualViewId) return false
                focused = View.NO_ID
                event(virtualViewId, AccessibilityEvent.TYPE_VIEW_ACCESSIBILITY_FOCUS_CLEARED)
                return true
            }
        }
        val n = nodes.getValue(virtualViewId)
        if (!n.bool("enabled")) return false
        val platformSupported = (n.string("role") == "edit" && (action == AccessibilityNodeInfo.ACTION_SET_TEXT || action == AccessibilityNodeInfo.ACTION_SET_SELECTION)) ||
            (n["range"]?.let { it != JsonNull } == true && action == AccessibilityNodeInfo.AccessibilityAction.ACTION_SET_PROGRESS.id)
        if (platformSupported && platformAction?.invoke(virtualViewId, action, arguments) == true) { refresh(); return true }
        val native = when (action) {
            AccessibilityNodeInfo.ACTION_CLICK -> ACCESS_CLICK
            AccessibilityNodeInfo.ACTION_LONG_CLICK -> ACCESS_LONG_CLICK
            AccessibilityNodeInfo.ACTION_DISMISS -> ACCESS_DISMISS
            AccessibilityNodeInfo.ACTION_SCROLL_FORWARD -> ACCESS_SCROLL_FORWARD
            AccessibilityNodeInfo.ACTION_SCROLL_BACKWARD -> ACCESS_SCROLL_BACKWARD
            else -> return false
        }
        if (!supports(virtualViewId, native)) return false
        val result = activate(virtualViewId, native)
        refresh()
        return result
    }

    fun hover(event: MotionEvent): Boolean {
        if (!exploring) return false
        val id = hit(event.x, event.y)
        when (event.actionMasked) {
            MotionEvent.ACTION_HOVER_ENTER, MotionEvent.ACTION_HOVER_MOVE -> {
                if (id != hovered) {
                    val old = hovered
                    hovered = id
                    if (id != View.NO_ID) {
                        event(id, AccessibilityEvent.TYPE_VIEW_HOVER_ENTER)
                        focus(id)
                    }
                    if (old != View.NO_ID) event(old, AccessibilityEvent.TYPE_VIEW_HOVER_EXIT)
                }
            }
            MotionEvent.ACTION_HOVER_EXIT -> {
                val old = hovered
                hovered = View.NO_ID
                if (old != View.NO_ID) {
                    event(old, AccessibilityEvent.TYPE_VIEW_HOVER_EXIT)
                    if (!hoverCommits && focused == old) {
                        focused = View.NO_ID
                        event(old, AccessibilityEvent.TYPE_VIEW_ACCESSIBILITY_FOCUS_CLEARED)
                    }
                    // Moving outside cancels. A lift inside the same key commits once.
                    if (hoverCommits && id == old && supports(old, ACCESS_CLICK)) {
                        activate(old, ACCESS_CLICK)
                        refresh()
                    }
                }
            }
        }
        return true
    }

    /** TalkBack's double-tap-and-hold arrives as touch; hover itself never starts repeat/glide. */
    fun touch(event: MotionEvent): Boolean {
        if (!exploring) return false
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                down = hit(event.x, event.y).takeIf { it != View.NO_ID } ?: focused
                held = false
                nodes[down]?.get("holdDelayMillis")?.jsonPrimitive?.long?.let { host.postDelayed(hold, it) }
            }
            MotionEvent.ACTION_UP -> {
                host.removeCallbacks(hold)
                val id = hit(event.x, event.y)
                val target = accessibilityReleaseTarget(down, id, held, popup,
                    nodes[id]?.string("role") == "popup item",
                    nodes.entries.firstOrNull { it.value.string("role") == "popup item" && it.value.bool("selected") }?.key)
                if (target != null) activate(target, ACCESS_CLICK)
                else if (held && popup) {
                    nodes.entries.firstOrNull { it.value.string("role") == "popup item" }?.key?.let {
                        activate(it, ACCESS_DISMISS)
                    }
                }
                down = View.NO_ID
                refresh()
            }
            MotionEvent.ACTION_CANCEL -> { host.removeCallbacks(hold); down = View.NO_ID; cancel(); refresh() }
        }
        return true
    }
    fun clear() { host.removeCallbacks(hold); down = View.NO_ID; hovered = View.NO_ID }
    fun release() { clear(); active = false; nodes.clear(); focused = View.NO_ID }
    private fun hit(x: Float, y: Float): Int = accessibilityHitTarget(x, y, host.width, host.height,
        nodes.map { (id, n) ->
            val b = bounds(n)
            AccessibilityTarget(id, intArrayOf(b.left, b.top, b.right, b.bottom), n.string("role") == "popup item")
        }) ?: View.NO_ID
    private fun supports(id: Int, action: Int) = nodes[id]?.get("actions")?.jsonArray?.any { it.jsonPrimitive.int == action } == true
    private fun focus(id: Int): Boolean {
        if (focused == id) return false
        if (focused != View.NO_ID) event(focused, AccessibilityEvent.TYPE_VIEW_ACCESSIBILITY_FOCUS_CLEARED)
        focused = id
        event(id, AccessibilityEvent.TYPE_VIEW_ACCESSIBILITY_FOCUSED)
        return true
    }
    private fun stateEvent(text: String) = event(View.NO_ID, AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED, text)
    private fun event(id: Int, type: Int, text: String? = null, change: Int = 0) {
        if (!manager.isEnabled || !host.isAttachedToWindow) return
        val event = AccessibilityEvent(type)
        event.packageName = host.context.packageName
        event.className = nodes[id]?.let(::nodeClass) ?: host.javaClass.name
        event.setSource(host, id)
        event.contentChangeTypes = change
        if (type == AccessibilityEvent.TYPE_VIEW_HOVER_ENTER || type == AccessibilityEvent.TYPE_VIEW_ACCESSIBILITY_FOCUSED) {
            event.contentDescription = nodes[id]?.string("description")
        }
        if (text != null) event.text.add(text)
        host.parent?.requestSendAccessibilityEvent(host, event)
    }
    private fun bounds(n: JsonObject): Rect {
        val a = n.getValue("bounds").jsonArray.map { it.jsonPrimitive.float }
        return Rect(floor(a[0]).toInt(), floor(a[1]).toInt(), ceil(a[2]).toInt(), ceil(a[3]).toInt())
    }
    private fun nodeClass(n: JsonObject): String = when (n.string("role")) {
        "text", "heading", "link" -> "android.widget.TextView"
        "image" -> "android.widget.ImageView"
        // A clickable Card has no Role, so Compose reports a plain View.
        "card" -> "android.view.View"
        "scroll" -> "android.widget.ScrollView"
        "key" -> "android.inputmethodservice.Keyboard\$Key"
        "toggle" -> "android.widget.Switch"
        "radio" -> "android.widget.RadioButton"
        "slider" -> "android.widget.SeekBar"
        "edit" -> "android.widget.EditText"
        else -> "android.widget.Button"
    }
    private fun JsonObject.int(key: String) = getValue(key).jsonPrimitive.int
    private fun JsonObject.string(key: String) = getValue(key).jsonPrimitive.content
    private fun JsonObject.bool(key: String) = getValue(key).jsonPrimitive.boolean
}
