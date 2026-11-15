package com.jtn0123.audiowhisper.settingsui

import android.app.Instrumentation
import android.content.Intent
import android.inputmethodservice.InputMethodService
import android.os.Bundle
import android.os.SystemClock
import android.text.InputType
import android.view.View
import android.view.ViewGroup
import android.view.accessibility.AccessibilityNodeInfo as Node
import androidx.core.net.toUri
import dev.patrickgold.florisboard.app.FlorisAppActivity
import org.json.JSONArray
import org.json.JSONObject
import java.lang.ref.WeakReference
import java.lang.reflect.Modifier

/** UiAutomation transport. Never persists node text from an editable node. */
internal class SettingsHarness(val instrumentation: Instrumentation, explore: Boolean = true) {
    val automation = instrumentation.uiAutomation.apply {
        serviceInfo = serviceInfo.apply { flags = (if (explore) flags or android.accessibilityservice.AccessibilityServiceInfo.FLAG_REQUEST_TOUCH_EXPLORATION_MODE else flags and android.accessibilityservice.AccessibilityServiceInfo.FLAG_REQUEST_TOUCH_EXPLORATION_MODE.inv()) or android.accessibilityservice.AccessibilityServiceInfo.FLAG_RETRIEVE_INTERACTIVE_WINDOWS or
            android.accessibilityservice.AccessibilityServiceInfo.FLAG_REPORT_VIEW_IDS }
    }
    var lastStep = "start"
    var activity: FlorisAppActivity? = null
    fun main(block: () -> Unit) {
        if (android.os.Looper.myLooper() == android.os.Looper.getMainLooper()) { block(); return }
        var failure: Throwable? = null
        instrumentation.runOnMainSync { try { block() } catch (error: Throwable) { failure = error } }
        failure?.let { throw it }
    }
    fun waitFor(stage: String, timeout: Long = 12000, condition: () -> Boolean) {
        lastStep = stage
        val end = SystemClock.uptimeMillis() + timeout
        while (SystemClock.uptimeMillis() < end) {
            if (condition()) return
            SystemClock.sleep(40)
        }
        error(stage)
    }
    /**
     * Waits out a navigation transition. Compose keeps the leaving screen's nodes, visible, until
     * its exit animation ends, so a walk reading too early sees two screens' rows. Settled is one
     * app bar's Back at most, and the same labels at the same bounds twice, 250 ms apart.
     */
    fun settle(stage: String = "navigation did not settle") {
        fun shot() = nodes().map { label(it) to android.graphics.Rect().also(it::getBoundsInScreen) }
        waitFor(stage) {
            val before = shot()
            SystemClock.sleep(250)
            before.count { it.first == "Back" } <= 1 && before == shot()
        }
    }
    /**
     * Waits until the IME shows for the focused field. An IME finishes any composition when it
     * starts input (FlorisBoard's start plan does), so edits sent before then race it, as no typing
     * user can: the keyboard is up before the first key. FlorisImeService runs in this process, so
     * it is asked directly. Its start plan runs on the main thread inside onStartInputView, and its
     * connection calls land on the main looper, so an idle main looper means the plan has landed.
     * ([settle] would read the IME's own window, which is this package's too, and never settle.)
     */
    fun awaitIme(stage: String = "IME did not show for the field") {
        var seen = ""
        try {
            waitFor(stage) {
                var shown = false
                main {
                    val ime = ime()
                    val multiLine = (ime?.currentInputEditorInfo?.inputType ?: 0) and InputType.TYPE_TEXT_FLAG_MULTI_LINE != 0
                    seen = "service=${ime != null} shown=${ime?.isInputViewShown} multiLine=$multiLine"
                    shown = ime != null && ime.isInputViewShown && multiLine
                }
                shown
            }
        } catch (error: IllegalStateException) {
            lastStep = "$stage ($seen)"
            throw IllegalStateException(lastStep, error)
        }
        instrumentation.waitForIdleSync()
    }
    /** This app's live IME, found as FixtureInstrumentation finds it. */
    private fun ime(): InputMethodService? = Class.forName("dev.patrickgold.florisboard.FlorisImeServiceKt").declaredFields
        .filter { Modifier.isStatic(it.modifiers) && WeakReference::class.java.isAssignableFrom(it.type) }
        .firstNotNullOfOrNull { field -> field.setAccessible(true); (field.get(null) as? WeakReference<*>)?.get() as? InputMethodService }
    fun launch(route: String = "settings/home") {
        // startActivitySync waits for a new activity. Compose's NavController restarts the task
        // for a deep link, but Rust navigates a live activity in place (onNewIntent), so the
        // wait never ended. A live activity is finished first, on both renderers.
        activity?.let { live ->
            main { live.finish() }
            waitFor("settings activity did not finish") { var gone = false; main { gone = live.isDestroyed }; gone }
        }
        val intent = Intent(Intent.ACTION_VIEW, "ui://florisboard/$route".toUri())
            .setClass(instrumentation.targetContext, FlorisAppActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK).addCategory(Intent.CATEGORY_BROWSABLE)
        activity = instrumentation.startActivitySync(intent) as FlorisAppActivity
        instrumentation.waitForIdleSync()
        // Android's active accessibility window can still be the launcher after a task
        // relaunch. Activate the non-interactive app-bar centre before walking nodes.
        val metrics = instrumentation.targetContext.resources.displayMetrics
        val now = SystemClock.uptimeMillis()
        for (action in listOf(android.view.MotionEvent.ACTION_DOWN, android.view.MotionEvent.ACTION_UP)) {
            val event = android.view.MotionEvent.obtain(now, now+1, action, metrics.widthPixels/2f, 48f*metrics.density, 0)
            event.source = android.view.InputDevice.SOURCE_TOUCHSCREEN
            check(automation.injectInputEvent(event, true))
            event.recycle()
        }
        waitFor("screen did not become accessible") { nodes().any { label(it).contains("Try out your setup") || label(it) == "Keyboard" } }
        activity = (instrumentation as SettingsInstrumentation).resumedActivity ?: activity
    }
    fun nodes(): List<Node> {
        val out = mutableListOf<Node>()
        fun walk(n: Node?) {
            if (n == null) return
            if (n.packageName?.toString() == instrumentation.targetContext.packageName && n.isVisibleToUser) out.add(n)
            for (i in 0 until n.childCount) walk(n.getChild(i))
        }
        automation.clearCache()
        val root = automation.windows.sortedByDescending { it.layer }
            .mapNotNull { it.root }.firstOrNull { it.packageName?.toString() == instrumentation.targetContext.packageName }
            ?: automation.rootInActiveWindow
        walk(root)
        return out
    }
    fun label(n: Node): String {
        if (n.isEditable) return n.hintText?.toString().orEmpty()
        n.contentDescription?.toString()?.takeIf { it.isNotBlank() }?.let { return it }
        n.text?.toString()?.takeIf { it.isNotBlank() }?.let { return it }
        if (n.isEditable) return ""
        return (0 until n.childCount).mapNotNull { n.getChild(it)?.let(::label)?.takeIf(String::isNotBlank) }.joinToString("\n")
    }
    fun find(label: String, classSuffix: String? = null): Node? {
        val matches = nodes().filter {
            (this.label(it) == label || this.label(it).startsWith("$label,") || this.label(it).startsWith("$label\n")) &&
                (classSuffix == null || it.className.toString().endsWith(classSuffix))
        }
        return matches.firstOrNull { it.isClickable && !it.isEditable } ?: matches.firstOrNull { it.isClickable } ?: matches.firstOrNull { this.label(it) == label } ?: matches.lastOrNull()
    }
    fun click(label: String, suffix: String? = null) {
        waitFor("click target unavailable: $label") { find(label, suffix) != null }
        val node = checkNotNull(find(label, suffix))
        node.performAction(Node.ACTION_CLEAR_ACCESSIBILITY_FOCUS)
        check(node.performAction(Node.ACTION_ACCESSIBILITY_FOCUS)) { "accessibility focus refused" }
        check(node.performAction(Node.ACTION_CLICK)) { "accessibility click refused" }
        SystemClock.sleep(150)
    }
    fun toggle(label: String) {
        val all = nodes()
        val row = checkNotNull(find(label))
        val bounds = android.graphics.Rect().also(row::getBoundsInScreen)
        val toggle = all.firstOrNull { n -> n.isClickable && n.isCheckable &&
            (this.label(n).startsWith(label) || bounds.contains(android.graphics.Rect().also(n::getBoundsInScreen))) }
        check(checkNotNull(toggle).performAction(Node.ACTION_CLICK))
        SystemClock.sleep(100)
    }
    fun scroll(forward: Boolean): Boolean {
        val action = if (forward) Node.ACTION_SCROLL_FORWARD else Node.ACTION_SCROLL_BACKWARD
        val node = nodes().firstOrNull { n -> n.isScrollable && n.actionList.any { it.id == action } } ?: return false
        val result = node.performAction(action)
        SystemClock.sleep(120)
        return result
    }
    fun top() { repeat(24) { if (!scroll(false)) return } }
    fun reveal(label: String) {
        repeat(24) {
            if (find(label) != null) return
            check(scroll(true)) { "cannot reveal $label" }
        }
        error("target not reached: $label")
    }
    fun <T : View> views(type: Class<T>): List<T> {
        val found = mutableListOf<T>()
        fun walk(v: View) {
            if (type.isInstance(v)) found.add(type.cast(v)!!)
            if (v is ViewGroup) for (i in 0 until v.childCount) walk(v.getChildAt(i))
        }
        main { walk(checkNotNull(activity).window.decorView) }
        return found
    }
    private fun role(node: Node): String {
        val name = node.className?.toString().orEmpty()
        val roles = listOf("Button", "CheckBox", "RadioButton", "Switch", "SeekBar", "EditText", "ScrollView")
        if (roles.any { name == "android.widget.$it" }) return name
        node.extras.getCharSequence("AccessibilityNodeInfo.roleDescription")?.takeIf { it.isNotBlank() }?.let { return it.toString() }
        // Compose can expose a semantic role through a separate non-clickable child.
        for (i in 0 until node.childCount) {
            val child = node.getChild(i) ?: continue
            val childRole = role(child)
            if (childRole.isNotEmpty()) return childRole
        }
        return ""
    }
    /**
     * A missing label that only a fork of the pinned Jetpref could fix (stage4-followup3.md), named
     * so the suite reports it apart from violations rather than passing it: the bare Switch that
     * ListPreference draws beside a linked row (its name is that row's title), and the Material
     * Slider inside DialogSliderPreference's dialog (named for the dialog). Compose path only.
     */
    private fun knownDefect(n: Node, screen: String): JSONObject? {
        if (n.rangeInfo != null && n.className == "android.widget.SeekBar")
            return JSONObject().put("name", "jetpref-dialog-slider-unlabeled").put("for", screen)
        if (!n.isCheckable || !n.isClickable || n.className != "android.view.View") return null
        val parent = n.parent ?: return null
        // The row is the switch's parent or one of its siblings, depending on Compose's merging.
        val row = (listOf(parent) + (0 until parent.childCount).mapNotNull(parent::getChild))
            .firstOrNull { it != n && it.isClickable && !it.isCheckable && label(it).isNotBlank() } ?: return null
        return JSONObject().put("name", "jetpref-linked-switch-unlabeled").put("for", label(row).substringBefore('\n'))
    }
    /**
     * ListPreference's dialog entry (Jetpref d6e12dda, ListPreference.kt): a Row with
     * `Modifier.selectable` and no `role = Role.RadioButton`, whose RadioButton has `onClick = null`
     * and so no semantics of its own. Labelled and selectable, but TalkBack is not told it is a
     * radio button. Only inside a dialog, and named for that dialog.
     */
    private fun roleless(n: Node, label: String, screen: String): JSONObject? {
        // This repository's own Compose, on Typing: FlorisSimpleCard's Card(onClick) (the spell
        // checker card) and FlorisHyperlinkText's clickable Text (the auto-space feedback link)
        // set no Role, so they are a View and a TextView. Rust exposes both as buttons.
        if (screen.startsWith("two:typing") && n.isClickable && !n.isCheckable && label.isNotBlank()) {
            when (n.className) {
                "android.view.View" -> return JSONObject().put("name", "floris-card-no-role").put("for", label.substringBefore('\n'))
                "android.widget.TextView" -> return JSONObject().put("name", "floris-hyperlink-no-role").put("for", label.substringBefore('\n'))
            }
        }
        if (!n.isCheckable || !n.isClickable || n.className != "android.view.View" || label.isBlank()) return null
        if (find("Cancel") == null) return null
        return JSONObject().put("name", "jetpref-list-entry-no-role").put("for", screen)
    }
    fun audit(screen: String): JSONObject {
        val items = JSONArray()
        val compose = views(RustSettingsView::class.java).isEmpty()
        for (n in nodes().filter { it.isClickable || it.isEditable || it.isScrollable || it.rangeInfo != null || it.isCheckable || it.isFocusable }) {
            lastStep = "audit:$screen:${n.className}"
            val label = if (n.isEditable) "<editable>" else label(n)
            val actions = n.actionList.map { it.id }
            val violations = mutableListOf<String>()
            if (n.isClickable || n.isEditable || n.isScrollable || n.rangeInfo != null || n.isFocusable) {
                n.performAction(Node.ACTION_CLEAR_ACCESSIBILITY_FOCUS)
                if (!n.performAction(Node.ACTION_ACCESSIBILITY_FOCUS)) violations.add("focus action failed")
                else if (!n.performAction(Node.ACTION_CLEAR_ACCESSIBILITY_FOCUS)) violations.add("clear focus action failed")
            }
            if (label.isBlank() && !n.isScrollable) violations.add("missing label")
            val semanticRole = role(n)
            if ((n.isClickable || n.isEditable || n.isScrollable || n.rangeInfo != null || n.isCheckable) && semanticRole.isEmpty())
                violations.add("missing semantic role")
            // Android advertises CLEAR rather than FOCUS on the currently focused node.
            if (Node.ACTION_ACCESSIBILITY_FOCUS !in actions && Node.ACTION_CLEAR_ACCESSIBILITY_FOCUS !in actions)
                violations.add("missing focus action")
            if (n.isClickable && n.isEnabled && Node.ACTION_CLICK !in actions) violations.add("missing click action")
            if (n.rangeInfo != null && n.isEnabled && Node.AccessibilityAction.ACTION_SET_PROGRESS.id !in actions) violations.add("missing progress action")
            val known = when {
                !compose -> null
                "missing label" in violations -> knownDefect(n, screen)?.also { violations.remove("missing label") }
                violations == listOf("missing semantic role") -> roleless(n, label, screen)?.also { violations.clear() }
                else -> null
            }
            items.put(JSONObject().put("label", label).put("role", semanticRole.ifEmpty { n.className?.toString().orEmpty() }).put("raw_class", n.className).put("heading", n.isHeading).put("enabled", n.isEnabled)
                .put("checkable", n.isCheckable).put("checked", (n.checked == Node.CHECKED_STATE_TRUE)).put("selected", n.isSelected)
                .put("value", n.rangeInfo?.current)
                .put("editor_units", if (n.isEditable) { if (n.isShowingHintText) 0 else n.text?.length ?: 0 } else null)
                .put("selection_start", if (n.isEditable) n.textSelectionStart else null).put("selection_end", if (n.isEditable) n.textSelectionEnd else null)
                .put("actions", JSONArray(actions)).put("violations", JSONArray(violations)).put("known_defect", known ?: JSONObject.NULL))
        }
        check(items.length() > 0)
        return JSONObject().put("screen", screen).put("items", items)
    }
    /** Every radio and slider of the open dialog, as the keyboard walk always exercised them. */
    fun exerciseDialog(title: String) {
        val modalSwitch = nodes().firstOrNull { it.isCheckable && it.isClickable && (it.className.toString().endsWith("Switch") || label(it).isBlank()) }
        if (modalSwitch != null && modalSwitch.checked != Node.CHECKED_STATE_TRUE) {
            check(modalSwitch.performAction(Node.ACTION_CLICK))
            SystemClock.sleep(100)
        }
        val radios = nodes().filter { it.isClickable && it.rangeInfo == null && !it.isEditable &&
            !it.className.toString().endsWith("Switch") && label(it).isNotBlank() &&
            label(it).substringBefore('\n') != title && label(it) !in listOf("Default", "Cancel", "OK") }
        for (target in radios) {
            target.performAction(Node.ACTION_CLEAR_ACCESSIBILITY_FOCUS)
            check(target.performAction(Node.ACTION_ACCESSIBILITY_FOCUS))
            val chosen = label(target).substringBefore('\n')
            check(target.performAction(Node.ACTION_CLICK))
            waitFor("radio choice did not select") { nodes().any { label(it).substringBefore('\n') == chosen && (it.isSelected || it.checked == Node.CHECKED_STATE_TRUE) } }
        }
        val sliderCount = nodes().count { it.rangeInfo != null }
        for (slot in 0 until sliderCount) {
            fun slider() = nodes().filter { it.rangeInfo != null }[slot]
            val range = checkNotNull(slider().rangeInfo)
            // Compose correctly refuses SET_PROGRESS when it would make no change.
            // Exercise a real transition through the other endpoint before minimum.
            if (range.current == range.min) {
                progress(range.max, slot)
                waitFor("slider maximum not applied") { slider().rangeInfo!!.current == range.max }
            }
            progress(range.min, slot)
            waitFor("slider minimum not applied") { slider().rangeInfo!!.current == range.min }
            check(slider().performAction(Node.ACTION_SCROLL_FORWARD))
            waitFor("slider forward did not update") { slider().rangeInfo!!.current > range.min }
            check(slider().performAction(Node.ACTION_SCROLL_BACKWARD))
            waitFor("slider backward did not update") { slider().rangeInfo!!.current == range.min }
        }
    }
    fun back() { main { checkNotNull((instrumentation as SettingsInstrumentation).resumedActivity ?: activity).onBackPressedDispatcher.onBackPressed() }; SystemClock.sleep(150); activity = (instrumentation as SettingsInstrumentation).resumedActivity ?: activity }
    fun close() { main { activity?.finish() }; activity = null }
    fun progress(value: Float, slot: Int = 0) {
        val slider = nodes().filter { it.rangeInfo != null }[slot]
        check(slider.performAction(Node.AccessibilityAction.ACTION_SET_PROGRESS.id,
            Bundle().apply { putFloat(Node.ACTION_ARGUMENT_PROGRESS_VALUE, value) }))
        SystemClock.sleep(80)
    }
}
