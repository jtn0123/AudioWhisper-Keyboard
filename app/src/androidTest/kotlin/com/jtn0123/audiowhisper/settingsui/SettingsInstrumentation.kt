package com.jtn0123.audiowhisper.settingsui

import android.app.Activity
import android.app.Instrumentation
import android.os.Bundle
import android.os.SystemClock
import android.view.accessibility.AccessibilityNodeInfo as Node
import com.jtn0123.audiowhisper.rust.NativeRuntime
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import kotlinx.coroutines.runBlocking
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.security.MessageDigest

/** Opt-in device suite; does not participate in the JVM gate or select any IME. */
class SettingsInstrumentation : Instrumentation() {
    private lateinit var options: Bundle
    @Volatile var resumedActivity: dev.patrickgold.florisboard.app.FlorisAppActivity? = null
    override fun callActivityOnResume(activity: Activity) { super.callActivityOnResume(activity); if (activity is dev.patrickgold.florisboard.app.FlorisAppActivity) resumedActivity = activity }
    @Volatile var savedActivity: Activity? = null
    override fun callActivityOnSaveInstanceState(activity: Activity, outState: Bundle) {
        super.callActivityOnSaveInstanceState(activity, outState)
        if (outState.getStringArray("rust-settings-state") != null) savedActivity = activity
    }
    private var stage = "bootstrap"
    override fun onCreate(arguments: Bundle?) {
        super.onCreate(arguments)
        options = arguments ?: Bundle()
        if (options.getString("mode") == "crash") {
            check(!NativeRuntime.load { throw UnsatisfiedLinkError("fixture payload must never be reported") })
        }
        start()
    }
    override fun onStart() {
        val report = JSONObject()
        val h = SettingsHarness(this, options.getString("mode") == "ui")
        try {
            stage = options.getString("mode") ?: "ui"
            when (stage) {
                "crash" -> {
                    h.activity = startActivitySync(android.content.Intent(targetContext, dev.patrickgold.florisboard.app.FlorisAppActivity::class.java)
                        .addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)) as dev.patrickgold.florisboard.app.FlorisAppActivity
                    h.waitFor("crash screen absent") { h.find("Copy details") != null }
                    check(h.nodes().any { h.label(it).contains("could not load its keyboard engine") })
                    check(h.views(android.widget.Button::class.java).size == 1)
                    check(h.views(androidx.compose.ui.platform.ComposeView::class.java).isEmpty())
                    h.main {
                        val clipboard = targetContext.getSystemService(android.content.ClipboardManager::class.java)
                        val original = clipboard.primaryClip
                        try {
                            check(h.views(android.widget.Button::class.java).single().performClick())
                            val copied = clipboard.primaryClip?.getItemAt(0)?.text?.toString()
                            check(copied == CrashReport.nativeLoad(checkNotNull(NativeRuntime.failure)).details)
                            check(copied.contains("fixture payload") == false)
                        } finally {
                            if (original == null) clipboard.clearPrimaryClip() else clipboard.setPrimaryClip(original)
                        }
                        report.put("copy_details_sanitized", true).put("clipboard_restored", true)
                    }
                    report.put("native_load_failure", true).put("non_compose", true)
                    SettingsCrashThemeDevice(h).run(report)
                    SettingsImeFallbackDevice(h).run(report)
                }
                "metrics" -> {
                    SettingsTypographyDevice(h).run(report)
                    SettingsTypographyFixture(h).run(report)
                }
                "text" -> SettingsEditorDevice(h).run(report)
                "compose-text" -> SettingsComposeEditorDevice(h).run(report)
                "ui" -> runUi(h, report)
                "prepare-death" -> SettingsEditorDevice(h).prepareDeath(report)
                else -> error("unknown device suite")
            }
            if (stage == "text" || stage == "compose-text") {
                h.close()
                val prefs by FlorisPreferenceStore
                runBlocking { prefs.other.rustSettingsPrototype.set(false).getOrThrow() }
                SystemClock.sleep(200)
                val bytes = File(targetContext.dataDir, "jetpref_datastore/florisboard-app-prefs.jetpref").readBytes()
                report.put("preference_bytes", bytes.size).put("preference_sha256", digest(bytes))
            }
            report.put("passed", true)
            File(targetContext.filesDir, "settings-device-${stage}.json").writeText(report.toString(2))
            h.close()
            finish(Activity.RESULT_OK, Bundle().apply { putString("settings_result", "PASS $stage") })
        } catch (failure: Throwable) {
            // Never export assertion messages, stack dumps or actual/expected field data.
            report.put("passed", false).put("stage", stage).put("last_step", h.lastStep).put("failure_type", failure.javaClass.name)
            failure.stackTrace.firstOrNull { it.className.startsWith("com.jtn0123.audiowhisper.settingsui.") }?.let {
                report.put("failure_site", "${it.className}.${it.methodName}:${it.lineNumber}")
            }
            if (options.getString("mode") == "ui") report.put("windows", JSONArray(h.automation.windows.map { window ->
                JSONObject().put("type", window.type).put("layer", window.layer).put("active", window.isActive).put("focused", window.isFocused).put("package", window.root?.packageName)
            })).put("visible_nodes", JSONArray(h.nodes().map { node ->
                JSONObject().put("class", node.className).put("clickable", node.isClickable).put("important", node.isImportantForAccessibility).put("bounds", android.graphics.Rect().also(node::getBoundsInScreen).toShortString()).put("label", if (node.isEditable) "<editable>" else h.label(node))
            }))
            File(targetContext.filesDir, "settings-device-failure.json").writeText(report.toString(2))
            h.close()
            finish(Activity.RESULT_CANCELED, Bundle().apply { putString("settings_result", "FAIL $stage (${failure.javaClass.simpleName})") })
        }
    }
    private fun runUi(h: SettingsHarness, report: JSONObject) {
        val prefs by FlorisPreferenceStore
        val screens = JSONArray()
        report.put("screens", screens)
        h.launch()
        screens.put(h.audit("home"))
        val homeRoutes = listOf("Languages & Layouts", "Theme", "Smartbar", "Dictation", "Typing", "Gestures & Glide typing", "Clipboard", "Emojis", "Addons & Extensions", "Other", "About")
        fun homeCount() = h.nodes().count { it.isClickable && h.label(it).substringBefore('\n') in homeRoutes }
        for (title in homeRoutes) {
            stage = "home-route:$title"
            h.click(title)
            h.waitFor("home route did not open") { homeCount() < 5 }
            h.back()
            h.waitFor("home route did not return") { homeCount() >= 10 }
        }
        report.put("home_routes_exercised", homeRoutes.size + 1)
        stage = "ui"
        h.click("Keyboard")
        h.waitFor("Keyboard unavailable") { h.find("Number row") != null }
        screens.put(h.audit("keyboard-top"))
        h.views(RustSettingsView::class.java).firstOrNull()?.let { view ->
            val node = checkNotNull(h.find("Number row"))
            val bounds = android.graphics.Rect().also(node::getBoundsInScreen)
            h.main {
                val origin = IntArray(2).also(view::getLocationOnScreen)
                val now = SystemClock.uptimeMillis()
                for (action in listOf(android.view.MotionEvent.ACTION_HOVER_ENTER, android.view.MotionEvent.ACTION_HOVER_EXIT)) {
                    val event = android.view.MotionEvent.obtain(now, now, action, bounds.exactCenterX()-origin[0], bounds.exactCenterY()-origin[1], 0)
                    event.source = android.view.InputDevice.SOURCE_TOUCHSCREEN
                    check(view.dispatchGenericMotionEvent(event))
                    event.recycle()
                    if (action == android.view.MotionEvent.ACTION_HOVER_ENTER) check(view.accessibilityNodeProvider.findFocus(Node.FOCUS_ACCESSIBILITY) != null)
                }
                check(view.accessibilityNodeProvider.findFocus(Node.FOCUS_ACCESSIBILITY) == null)
            }
            report.put("hover_exit_cleared_focus", true)
        }
        val original = prefs.keyboard.numberRow.get()
        h.click("Number row")
        h.waitFor("switch preference did not change") { prefs.keyboard.numberRow.get() != original }
        h.click("Number row")
        h.waitFor("switch preference did not restore") { prefs.keyboard.numberRow.get() == original }
        for ((label, preference) in listOf("Rust keyboard renderer (prototype)" to prefs.keyboard.rustRendererPrototype,
            "Show utility key" to prefs.keyboard.utilityKeyEnabled, "Popup Visibility" to prefs.keyboard.popupEnabled,
            "Accents include symbol popups" to prefs.keyboard.mergeHintPopupsEnabled,
            "Space bar switches to characters" to prefs.keyboard.spaceBarSwitchesToCharacters)) {
            stage = "toggle:$label"
            h.reveal(label)
            screens.put(h.audit(label))
            val old = preference.get()
            h.click(label)
            h.waitFor("toggle did not persist") { preference.get() != old }
            h.click(label)
            h.waitFor("toggle did not restore") { preference.get() == old }
        }
        h.top()
        if (!prefs.keyboard.utilityKeyEnabled.get()) {
            h.click("Show utility key")
            h.waitFor("utility toggle did not persist") { prefs.keyboard.utilityKeyEnabled.get() }
        }
        for ((label, preference) in listOf("Hinted number row" to prefs.keyboard.hintedNumberRowEnabled,
            "Hinted symbols" to prefs.keyboard.hintedSymbolsEnabled, "One-handed mode" to prefs.keyboard.oneHandedModeEnabled)) {
            stage = "linked-toggle:$label"
            h.reveal(label)
            val old = preference.get()
            h.toggle(label)
            h.waitFor("linked toggle did not change") { preference.get() != old }
            h.toggle(label)
            h.waitFor("linked toggle did not restore") { preference.get() == old }
        }
        h.top()
        val dialogs = listOf("Hinted number row", "Hinted symbols", "Utility key action", "Spacebar label", "Capitalization behavior", "Font size multiplier", "Incognito indicator", "One-handed mode", "One-handed mode keyboard width", "Landscape fullscreen input", "Keyboard height", "Key spacing", "Bottom offset", "Long key press delay")
        for (title in dialogs) {
            stage = "dialog:$title"
            h.reveal(title)
            val row = h.find(title)
            if (row?.isEnabled != true) {
                check(title == "One-handed mode keyboard width")
                h.toggle("One-handed mode")
                h.waitFor("one-handed switch did not enable") { prefs.keyboard.oneHandedModeEnabled.get() }
            }
            h.click(title)
            h.waitFor("dialog not opened") { h.find("Cancel") != null }
            screens.put(h.audit(title))
            if (title == "Hinted number row") {
                h.click("Default")
                h.waitFor("Default did not close") { h.find("Cancel") == null }
                h.click(title)
                h.waitFor("reset dialog not reopened") { h.find("Cancel") != null }
                h.click("Cancel")
                h.waitFor("Cancel did not close") { h.find("Cancel") == null }
                h.click(title)
                h.waitFor("cancelled dialog not reopened") { h.find("Cancel") != null }
            }
            h.exerciseDialog(title)
            h.click("OK")
            h.waitFor("dialog did not close") { h.find("Cancel") == null }
        }
        h.top()
        screens.put(h.audit("keyboard-returned"))
        SettingsGroupOneDevice(h) { stage = it }.run(report, screens)
        SettingsGroupTwoDevice(h) { stage = it }.run(report, screens)
        stage = "ui"
        h.click("Keyboard")
        h.waitFor("Keyboard unavailable") { h.find("Number row") != null }
        report.put("screens", screens)
        var violations = 0
        var known = 0
        for (i in 0 until screens.length()) {
            val items = screens.getJSONObject(i).getJSONArray("items")
            for (j in 0 until items.length()) {
                violations += items.getJSONObject(j).getJSONArray("violations").length()
                if (!items.getJSONObject(j).isNull("known_defect")) known++
            }
        }
        // Known defects are owed, not passed: the device script lists every one by name.
        report.put("accessibility_violations", violations).put("accessibility_known_defects", known)
            .put("accessibility_passed", violations == 0)
        h.waitFor("preference writes pending") { h.views(RustSettingsView::class.java).none { it.pendingWrites > 0 } }
        stage = "picker"
        val priorWindows = h.automation.windows.map { it.id }.toSet()
        h.click("Switch Keyboard")
        var pickerShown = false
        val pickerDeadline = SystemClock.uptimeMillis()+2000
        while (SystemClock.uptimeMillis()<pickerDeadline && !pickerShown) {
            pickerShown = h.automation.windows.any { window ->
                val title = window.title?.toString().orEmpty()
                window.type == android.view.accessibility.AccessibilityWindowInfo.TYPE_INPUT_METHOD ||
                    title.contains("inputmethod",true) || title.contains("input method",true) ||
                    title.contains("keyboard",true) || window.id !in priorWindows
            }
            if (!pickerShown) SystemClock.sleep(40)
        }
        report.put("system_ime_picker", pickerShown)
        if (pickerShown) {
            val pickerWindows = h.automation.windows.map { it.id }.toSet()-priorWindows
            check(h.automation.performGlobalAction(android.accessibilityservice.AccessibilityService.GLOBAL_ACTION_BACK))
            h.waitFor("picker did not dismiss") { h.automation.windows.none { it.id in pickerWindows } && h.find("Keyboard") != null }
        }
        // Explicitly return the selector to the same stored value on both paths.
        h.close()
        runBlocking { prefs.other.rustSettingsPrototype.set(false).getOrThrow() }
        SystemClock.sleep(200)
        val bytes = File(targetContext.dataDir, "jetpref_datastore/florisboard-app-prefs.jetpref").readBytes()
        report.put("preference_bytes", bytes.size).put("preference_sha256", digest(bytes))
        stage = "ui"
    }
    companion object {
        fun digest(bytes: ByteArray): String = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }
    }
}
