package com.jtn0123.audiowhisper.settingsui

import android.graphics.Rect
import android.view.accessibility.AccessibilityNodeInfo as Node
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import org.florisboard.lib.android.systemVibratorOrNull
import org.json.JSONArray
import org.json.JSONObject

/**
 * The group 1 screens on both renderers: every enabled row is exercised top to bottom, then the
 * screen's masters are switched so its conditional rows are reached in a second pass. The walk is
 * driven by what the screen shows, so a row one renderer shows, hides or disables differently
 * changes the audit sequence and the preference bytes the host script compares.
 */
internal class SettingsGroupOneDevice(private val h: SettingsHarness, private val stage: (String) -> Unit) {
    private class Screen(
        val name: String,
        val title: String,
        val flips: List<String> = emptyList(),
        val ensureOn: List<String> = emptyList(),
        val choose: List<Pair<String, String>> = emptyList(),
    )
    private val prefs by FlorisPreferenceStore
    private val screens = listOf(
        Screen("input-feedback", "Sounds & Vibration", choose = listOf("Vibration mode" to "Use vibrator directly")),
        Screen("smartbar", "Smartbar", ensureOn = listOf("Enable Smartbar"),
            choose = listOf("Layout" to "Suggestions & Actions shared")),
        Screen("gestures", "Gestures & Glide typing", flips = listOf("Enable glide typing")),
        Screen("clipboard", "Clipboard", ensureOn = listOf("Use internal clipboard", "Clipboard content suggestions",
            "Enable clipboard history", "Clean up old items", "Auto clean sensitive items", "Limit history size")),
        Screen("media", "Emojis", ensureOn = listOf("Enable emoji history", "Enable emoji suggestions")),
    )
    private lateinit var out: JSONArray
    private lateinit var log: JSONArray
    private val seenDisabled = mutableSetOf<String>()
    private val seenEnabled = mutableSetOf<String>()

    fun run(report: JSONObject, out: JSONArray) {
        this.out = out
        val vibrator = h.instrumentation.targetContext.systemVibratorOrNull()
        val facts = JSONObject().put("has_vibrator", vibrator?.hasVibrator() == true)
            .put("has_amplitude_control", vibrator?.hasAmplitudeControl() == true)
        val result = JSONObject().put("vibrator", facts)
        for (screen in screens) {
            stage("group-one:${screen.name}")
            open(screen)
            log = JSONArray()
            seenDisabled.clear()
            seenEnabled.clear()
            walk(screen, facts)
            result.put(screen.name, JSONObject().put("exercised", log)
                .put("never_enabled", JSONArray((seenDisabled - seenEnabled).sorted())))
            close(screen)
        }
        report.put("group_one", result)
    }

    private fun open(screen: Screen) {
        if (screen.name == "input-feedback") {
            h.top()
            h.reveal(screen.title)
            h.click(screen.title)
        } else {
            h.click(screen.title)
        }
        h.waitFor("${screen.name} did not open") { h.find("Languages & Layouts") == null && h.find("Number row") == null && h.find(screen.title) != null }
        out.put(h.audit("${screen.name}-top"))
    }

    private fun close(screen: Screen) {
        h.back()
        if (screen.name == "input-feedback") {
            // No input-feedback row is called "Keyboard"; the keyboard screen's title is.
            h.waitFor("keyboard did not return") { h.find("Keyboard") != null }
            h.back()
        }
        h.waitFor("home did not return") { h.find("Languages & Layouts") != null }
    }

    private fun walk(screen: Screen, facts: JSONObject) {
        val done = mutableSetOf<String>()
        pass(screen, done)
        out.put(h.audit("${screen.name}-bottom"))
        for ((row, choice) in screen.choose) choose(row, choice)
        val switched = screen.flips.onEach { flip(it) } + screen.ensureOn.filter { !checked(it) }.onEach { flip(it) }
        if (screen.name == "input-feedback") vibrationRows(facts)
        pass(screen, done)
        out.put(h.audit("${screen.name}-expanded"))
        h.top()
        for (label in switched.reversed()) flip(label)
    }

    /** Every enabled, not yet exercised row, from the top down, scrolling until the end. */
    private fun pass(screen: Screen, done: MutableSet<String>) {
        h.top()
        while (true) {
            val row = next(done)
            if (row == null) { if (!h.scroll(true)) return; continue }
            done.add(row)
            exercise(screen, row)
        }
    }

    private fun bounds(n: Node) = Rect().also(n::getBoundsInScreen)
    private fun primary(n: Node) = h.label(n).substringBefore('\n')

    private fun next(done: Set<String>): String? {
        val rows = h.nodes().filter { it.isClickable && !it.isEditable && it.rangeInfo == null && h.label(it).isNotBlank() && primary(it) != "Back" }
        // A linked switch lies inside its row; it is exercised with that row, not as one.
        val own = rows.filter { n -> rows.none { o -> o != n && bounds(o) != bounds(n) && bounds(o).contains(bounds(n)) } }
        for (n in own) if (n.isEnabled) seenEnabled.add(primary(n)) else seenDisabled.add(primary(n))
        return own.filter { it.isEnabled && primary(it) !in done }.minByOrNull { bounds(it).top }?.let(::primary)
    }

    private fun checked(label: String): Boolean {
        h.reveal(label)
        return checkNotNull(h.find(label)).checked == Node.CHECKED_STATE_TRUE
    }

    private fun linkedSwitch(label: String): Node? {
        val row = h.find(label) ?: return null
        val all = h.nodes()
        val rowBounds = bounds(row)
        return all.firstOrNull { n -> n != row && n.isClickable && n.isCheckable && rowBounds.contains(bounds(n)) && bounds(n) != rowBounds }
    }

    private fun exercise(screen: Screen, label: String) {
        val row = checkNotNull(h.find(label))
        val linked = linkedSwitch(label) != null
        if (row.isCheckable && !linked) {
            val before = row.checked == Node.CHECKED_STATE_TRUE
            h.click(label)
            h.waitFor("toggle did not change: $label") { checked(label) != before }
            h.click(label)
            h.waitFor("toggle did not restore: $label") { checked(label) == before }
            log.put(JSONObject().put("row", label).put("kind", "toggle"))
            return
        }
        h.click(label)
        h.waitFor("dialog not opened: $label") { h.find("Cancel") != null || h.find("No") != null }
        out.put(h.audit("${screen.name}:$label"))
        if (h.find("Cancel") == null) {
            confirm(label)
            log.put(JSONObject().put("row", label).put("kind", "confirm"))
            return
        }
        h.exerciseDialog(label)
        h.click("OK")
        h.waitFor("dialog did not close: $label") { h.find("Cancel") == null }
        if (linked) {
            val before = checkNotNull(linkedSwitch(label)).checked
            h.toggle(label)
            h.waitFor("linked toggle did not change: $label") { linkedSwitch(label)?.checked != before }
            h.toggle(label)
            h.waitFor("linked toggle did not restore: $label") { linkedSwitch(label)?.checked == before }
        }
        log.put(JSONObject().put("row", label).put("kind", if (linked) "linked" else "dialog"))
    }

    /** JetPrefAlertDialog: "No" leaves the history, "Yes" deletes it. */
    private fun confirm(label: String) {
        check(h.find("Yes") != null && h.find("Default") == null)
        h.click("No")
        h.waitFor("confirmation did not dismiss: $label") { h.find("No") == null }
        h.click(label)
        h.waitFor("confirmation not reopened: $label") { h.find("Yes") != null }
        h.click("Yes")
        h.waitFor("confirmation did not accept: $label") { h.find("Yes") == null }
    }

    private fun choose(row: String, choice: String) {
        h.top()
        h.reveal(row)
        h.click(row)
        h.waitFor("dialog not opened: $row") { h.find("Cancel") != null }
        h.click(choice)
        h.waitFor("choice not selected: $choice") { h.nodes().any { primary(it) == choice && (it.isSelected || it.checked == Node.CHECKED_STATE_TRUE) } }
        h.click("OK")
        h.waitFor("dialog did not close: $row") { h.find("Cancel") == null }
        log.put(JSONObject().put("row", row).put("kind", "choose").put("choice", choice))
    }

    private fun flip(label: String) {
        h.top()
        val before = checked(label)
        h.click(label)
        h.waitFor("master did not change: $label") { checked(label) != before }
        log.put(JSONObject().put("row", label).put("kind", "flip").put("to", !before))
    }

    /** InputFeedbackScreen enables duration with a vibrator and strength with amplitude control. */
    private fun vibrationRows(facts: JSONObject) {
        val direct = prefs.inputFeedback.hapticVibrationMode.get().name == "USE_VIBRATOR_DIRECTLY"
        val haptic = prefs.inputFeedback.hapticEnabled.get()
        check(direct && haptic)
        val vibrator = facts.getBoolean("has_vibrator")
        val amplitude = vibrator && facts.getBoolean("has_amplitude_control")
        h.top()
        h.reveal("Vibration duration")
        check(checkNotNull(h.find("Vibration duration")).isEnabled == vibrator)
        h.reveal("Vibration strength")
        check(checkNotNull(h.find("Vibration strength")).isEnabled == amplitude)
        facts.put("duration_enabled", vibrator).put("strength_enabled", amplitude)
    }
}
