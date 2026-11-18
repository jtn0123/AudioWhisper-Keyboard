package com.jtn0123.audiowhisper.settingsui

import android.app.UiAutomation
import android.graphics.Bitmap
import android.graphics.PointF
import android.graphics.Rect
import android.os.SystemClock
import android.text.format.DateFormat
import android.view.InputDevice
import android.view.MotionEvent
import android.view.accessibility.AccessibilityNodeInfo as Node
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.isUnspecified
import androidx.compose.ui.graphics.toArgb
import dev.patrickgold.florisboard.app.AppTheme
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.theme.ThemeMode
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.themeManager
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.floor
import kotlin.math.hypot
import kotlin.math.roundToInt

/**
 * Other and Theme as Rust draws them (docs/stage4-other-theme.md), on the device's own clock:
 * every row that opens something, the colour dialog through Presets, Custom (the plane and hue bar
 * dragged, a value typed) and Default, the time dialog on the dial (tapped and dragged) and typed,
 * Back from every dialog and from the Compose screens the rows open, rotation with a dialog open,
 * and that the keyboard follows: its theme by the mode and times, its colours by the accent.
 *
 * Receipts hold booleans, counts and pixel fractions only, never typed text or a colour value.
 * The preferences it changes are put back at the end; the host restores the files as well.
 */
internal class SettingsOtherThemeDevice(private val h: SettingsHarness, private val stage: (String) -> Unit) {
    private val prefs by FlorisPreferenceStore
    private val context = h.instrumentation.targetContext
    private val density = context.resources.displayMetrics.density
    private val is24 = DateFormat.is24HourFormat(context)
    private val result = JSONObject()
    private val runner get() = h.instrumentation as SettingsInstrumentation
    private lateinit var out: JSONArray

    fun run(report: JSONObject, out: JSONArray) {
        this.out = out
        val restore = snapshot()
        try {
            other()
            theme()
        } finally {
            h.automation.setRotation(UiAutomation.ROTATION_FREEZE_0)
            restore()
        }
        report.put("other_theme", result.put("clock", if (is24) 24 else 12))
    }

    /** The preferences this walk changes, and how to put them back. */
    private fun snapshot(): () -> Unit {
        val (o, t) = prefs.other to prefs.theme
        val accent = o.accentColor.get()
        val settingsTheme = o.settingsTheme.get()
        val mode = t.mode.get()
        val day = t.dayThemeId.get()
        val night = t.nightThemeId.get()
        val themeAccent = t.accentColor.get()
        val sunrise = t.sunriseTime.get()
        val sunset = t.sunsetTime.get()
        return {
            kotlinx.coroutines.runBlocking {
                o.accentColor.set(accent).getOrThrow()
                o.settingsTheme.set(settingsTheme).getOrThrow()
                t.mode.set(mode).getOrThrow()
                t.dayThemeId.set(day).getOrThrow()
                t.nightThemeId.set(night).getOrThrow()
                t.accentColor.set(themeAccent).getOrThrow()
                t.sunriseTime.set(sunrise).getOrThrow()
                t.sunsetTime.set(sunset).getOrThrow()
            }
        }
    }

    private fun other() {
        stage("other:open")
        h.launch()
        h.click("Other")
        h.settle()
        onRust("Settings theme")
        out.put(h.audit("other"))
        choose("Settings theme", "Dark") { prefs.other.settingsTheme.get() == AppTheme.DARK }
        choose("Settings theme", "System default") { prefs.other.settingsTheme.get() == AppTheme.AUTO }
        colours("other", "Settings accent color") { prefs.other.accentColor.get() }
        stage("other:physical-keyboard")
        h.click("Physical keyboard")
        h.settle()
        onRust("System physical keyboard settings")
        h.back()
        h.settle()
        onRust("Settings theme")
        compose("other", "Back up data", "Settings theme")
        compose("other", "Restore data", "Settings theme")
        result.put("other", true)
    }

    private fun theme() {
        stage("theme:open")
        h.launch()
        h.click("Theme")
        h.settle()
        onRust("Theme mode")
        out.put(h.audit("theme"))
        val t = prefs.theme
        mode("Always night", ThemeMode.ALWAYS_NIGHT)
        check(!node("Day theme").isEnabled && node("Night theme").isEnabled && !node("Sunrise time").isEnabled)
        follows(t.nightThemeId.get())
        mode("Always day", ThemeMode.ALWAYS_DAY)
        check(node("Day theme").isEnabled && !node("Night theme").isEnabled)
        follows(t.dayThemeId.get())
        manager("Day theme", "Floris Day (Material You)") { t.dayThemeId.get().toString() == "org.florisboard.themes.my:floris_day_my" }
        follows(t.dayThemeId.get())
        // Night is greyed out under Always day; the accent check below needs the Material You night theme active.
        mode("Always night", ThemeMode.ALWAYS_NIGHT)
        manager("Night theme", "Floris Night (Material You)") { t.nightThemeId.get().toString() == "org.florisboard.themes.my:floris_night_my" }
        follows(t.nightThemeId.get())
        colours("theme", "Accent color (Material You themes)") { t.accentColor.get() }
        keyboardAccent()
        mode("Follow time", ThemeMode.FOLLOW_TIME)
        check(node("Sunrise time").isEnabled && node("Sunset time").isEnabled)
        dialTime("Sunrise time") { t.sunriseTime.get().let { it.hour to it.minute } }
        typedTime("Sunset time") { t.sunsetTime.get().let { it.hour to it.minute } }
        timeDialogsClose("Sunrise time") { t.sunriseTime.get().let { it.hour to it.minute } }
        // The keyboard re-reads the times when the mode changes, as it always has.
        mode("Always day", ThemeMode.ALWAYS_DAY)
        mode("Follow time", ThemeMode.FOLLOW_TIME)
        val now = java.time.LocalTime.now()
        val day = now >= java.time.LocalTime.of(15, 15) && now <= java.time.LocalTime.of(19, 45)
        follows(if (day) t.dayThemeId.get() else t.nightThemeId.get())
        listSurvivesRotation()
        compose("theme", "Go to Theme extensions", "Theme mode")
        result.put("theme", true)
    }

    /** A list row's dialog: the entry chosen and confirmed, then stored. */
    private fun choose(row: String, entry: String, stored: () -> Boolean) {
        stage("choose:$row:$entry")
        h.reveal(row)
        h.click(row)
        h.waitFor("$row dialog did not open") { h.find("Cancel") != null }
        h.click(entry)
        h.click("OK")
        h.waitFor("$row dialog did not close") { h.find("Cancel") == null }
        h.waitFor("$row choice not stored", condition = stored)
    }
    private fun mode(entry: String, value: ThemeMode) = choose("Theme mode", entry) { prefs.theme.mode.get() == value }

    /** The keyboard's theme: the one ThemeManager made active, named by the preference that chose it. */
    private fun follows(name: ExtensionComponentName) {
        h.waitFor("keyboard theme did not follow") { context.themeManager().value.activeThemeInfo.value.name == name }
    }

    /** A row that opens the Compose theme manager; a theme picked there; Back to Rust, the row renamed. */
    private fun manager(row: String, label: String, stored: () -> Boolean) {
        stage("theme:manager:$row")
        h.click(row)
        h.waitFor("$row did not open the Compose theme manager") { fallback() && h.find(label) != null }
        h.activity = runner.resumedActivity
        h.settle()
        h.reveal(label)
        // A plain click: Compose refuses accessibility focus while touch exploration is off, which
        // the harness's click asks for first.
        check(node(label).performAction(Node.ACTION_CLICK)) { "theme manager click refused" }
        h.waitFor("$row choice not stored", condition = stored)
        h.back()
        h.settle()
        onRust(row)
        h.waitFor("$row does not name the theme") { node(row).stateDescription?.toString() == label }
        result.put("manager_${row.substringBefore(' ').lowercase()}", true)
    }

    /** A row that opens a screen still in Compose, and Back from it to the Rust screen. */
    private fun compose(screen: String, row: String, home: String) {
        stage("$screen:compose:$row")
        h.reveal(row)
        h.click(row)
        h.waitFor("$row did not open in Compose") { fallback() }
        h.activity = runner.resumedActivity
        h.settle()
        h.back()
        h.settle()
        onRust(home)
        result.put("compose_${row.lowercase().replace(' ', '_')}", true)
    }

    /** The colour dialog: a preset, the plane and hue bar dragged, a value typed, Default, Back, rotation. */
    private fun colours(screen: String, row: String, value: () -> Color) {
        stage("$screen:colour:presets")
        openColour(row)
        out.put(h.audit("$screen:colour-presets"))
        h.click("#f44336")
        confirm(row)
        h.waitFor("preset not stored") { value().toArgb() == 0xFFF44336.toInt() }
        stage("$screen:colour:custom")
        openColour(row)
        page("Custom")
        out.put(h.audit("$screen:colour-custom"))
        dragColour()
        val dragged = shown()
        confirm(row)
        h.waitFor("dragged colour not stored") { value().toArgb() == dragged }
        stage("$screen:colour:typed")
        openColour(row)
        page("Custom")
        h.click("Edit color")
        h.waitFor("hex field not editable") { h.find("Apply changes") != null }
        // Edit makes the field editable; a tap on it focuses it, as in Jetpref.
        val hex = checkNotNull(h.nodes().firstOrNull { it.isEditable && h.label(it) == "Color" }) { "hex field absent" }
        check(hex.performAction(Node.ACTION_CLICK))
        type("#3366cc")
        h.waitFor("the field does not hold the typed value") { var t = ""; h.main { t = host().editorText }; t == "#3366cc" }
        h.click("Apply changes")
        h.waitFor("typed colour not applied") { shownOrNull() == 0xFF3366CC.toInt() }
        confirm(row)
        h.waitFor("typed colour not stored") { value().toArgb() == 0xFF3366CC.toInt() }
        stage("$screen:colour:default")
        openColour(row)
        page("Presets")
        h.click("Default")
        confirm(row)
        h.waitFor("Default not stored as none") { value().isUnspecified }
        stage("$screen:colour:back")
        openColour(row)
        h.back()
        h.waitFor("Back did not close the colour dialog") { h.find("Cancel") == null && h.find(row) != null }
        check(value().isUnspecified)
        stage("$screen:colour:rotation")
        openColour(row)
        closedByRotation(row)
        check(value().isUnspecified)
        result.put("${screen}_colour", true)
    }
    private fun openColour(row: String) {
        h.reveal(row)
        h.click(row)
        h.waitFor("$row dialog did not open") { h.find("Cancel") != null && (h.find("Custom") != null || h.find("Presets") != null) }
    }
    /** The page the neutral button names, unless it is already showing. */
    private fun page(name: String) {
        val other = if (name == "Custom") "Presets" else "Custom"
        if (h.find(other) == null) h.click(name)
        h.waitFor("$name page absent") { h.find(other) != null }
    }
    private fun confirm(row: String) {
        h.click("OK")
        h.waitFor("$row dialog did not close") { h.find("Cancel") == null }
    }
    /** The hex the Custom page's read-only field shows, as a colour. */
    private fun shown(): Int = checkNotNull(shownOrNull()) { "no colour shown" }
    /** The same, or null while the field is still being edited and so shows no hex yet. */
    private fun shownOrNull(): Int? = h.find("Color")?.stateDescription?.toString()
        ?.takeIf { Regex("#[0-9a-f]{6}").matches(it) }?.let(android.graphics.Color::parseColor)

    /**
     * The plane and hue bar are not accessibility nodes (as in Compose), so they are found from the
     * hex field under them, as the page lays them out: the square plane, 12 dp, the 64 dp preview
     * row with the hue track, 12 dp, the field. The plane's top right corner is full saturation
     * and value; the hue bar's middle is cyan.
     */
    private fun dragColour() {
        val field = bounds(node("Color"))
        val w = field.width().toFloat()
        val row = field.top - px(12f) - px(64f)
        val planeTop = row - px(12f) - w
        val centre = PointF(field.left + w / 2, planeTop + w / 2)
        drag(listOf(centre, PointF(centre.x + w / 4, centre.y - w / 4), PointF(field.right - 2f, planeTop + 2f)))
        val hsv = FloatArray(3).also { android.graphics.Color.colorToHSV(shown(), it) }
        check(hsv[1] > 0.95f && hsv[2] > 0.95f) { "plane drag missed the corner" }
        val left = field.left + px(2f) + px(64f) + px(16f) + px(12f)
        val width = field.left + w - px(12f) - left
        val y = row + ((px(64f) - 3 * px(12f)) / 2f).roundToInt() + px(12f) + px(12f) / 2f
        drag(listOf(PointF(left + width * 0.1f, y), PointF(left + width * 0.3f, y), PointF(left + width * 0.5f, y)))
        android.graphics.Color.colorToHSV(shown(), hsv)
        check(hsv[0] in 165f..195f && hsv[1] > 0.95f && hsv[2] > 0.95f) { "hue drag did not reach the middle" }
        result.put("plane_and_hue_dragged", true)
    }

    /** The time dialog on the dial: 7 and 30 tapped; then 3 PM (or the inner ring's 15) and 15 dragged. */
    private fun dialTime(row: String, value: () -> Pair<Int, Int>) {
        stage("theme:time:dial")
        openTime(row)
        out.put(h.audit("theme:time-dial"))
        h.click(number(7, false))
        h.waitFor("the dial did not move on to the minute") { h.find(number(30, true)) != null }
        h.click(number(30, true))
        if (!is24) h.click("AM")
        confirm(row)
        h.waitFor("tapped time not stored") { value() == 7 to 30 }
        check(summary(row) == if (is24) "7:30" else "7:30 AM")
        openTime(row)
        val start = centre(node(number(if (is24) 0 else 12, false)))
        val dial = PointF(start.x, centre(node(number(3, false))).y)
        dragRound(dial, start, centre(node(number(if (is24) 15 else 3, false))))
        h.waitFor("the drag did not move on to the minute") { h.find(number(15, true)) != null }
        dragRound(dial, centre(node(number(0, true))), centre(node(number(15, true))))
        if (!is24) h.click("PM")
        confirm(row)
        h.waitFor("dragged time not stored") { value() == 15 to 15 }
        check(summary(row) == if (is24) "15:15" else "3:15 PM")
        result.put("time_dial", true)
    }

    /** The time dialog typed: 7 (which moves on) or 19 then Next, 45, and PM on a 12-hour clock. */
    private fun typedTime(row: String, value: () -> Pair<Int, Int>) {
        stage("theme:time:typed")
        openTime(row)
        h.click("Switch time input to keyboard input")
        h.waitFor("hour field absent") { h.find("for hour") != null }
        out.put(h.audit("theme:time-typed"))
        type(if (is24) "19" else "7")
        if (is24) h.main { check(connection().performEditorAction(EditorInfo.IME_ACTION_NEXT)) }
        h.waitFor("the minute field did not take focus") { var f = false; h.main { f = host().numberAction == "done" }; f }
        type("45")
        if (!is24) h.click("PM")
        confirm(row)
        h.waitFor("typed time not stored") { value() == 19 to 45 }
        check(summary(row) == if (is24) "19:45" else "7:45 PM")
        result.put("time_typed", true)
    }

    /** Back and rotation both close the time dialog and leave the time as it was. */
    private fun timeDialogsClose(row: String, value: () -> Pair<Int, Int>) {
        stage("theme:time:back")
        val before = value()
        openTime(row)
        h.back()
        h.waitFor("Back did not close the time dialog") { h.find("Cancel") == null && h.find(row) != null }
        stage("theme:time:rotation")
        openTime(row)
        closedByRotation(row)
        check(value() == before)
    }
    private fun openTime(row: String) {
        h.reveal(row)
        h.click(row)
        h.waitFor("$row dialog did not open") { h.find("Cancel") != null && (h.find(number(3, false)) != null || h.find("for hour") != null) }
        // The dialog opens on the dial each time, as Jetpref's does.
        if (h.find(number(3, false)) == null) h.click("Switch time input to time picker")
        h.waitFor("the dial is absent") { h.find(number(3, false)) != null || h.find(number(15, true)) != null }
        if (h.find(number(3, false)) == null) h.click("Select hour")
        h.waitFor("the hours are absent") { h.find(number(3, false)) != null }
    }
    private fun number(n: Int, minutes: Boolean) = when {
        minutes -> "$n minutes"
        is24 -> "$n hours"
        else -> "$n o'clock"
    }
    private fun summary(row: String) = node(row).stateDescription?.toString()

    /**
     * A finger from `from` round to `to` on the dial centred at `dial`. It first moves straight out
     * past the touch slop, so the hand, which trails the finger by the slop in that direction,
     * ends exactly on `to`.
     */
    private fun dragRound(dial: PointF, from: PointF, to: PointF) {
        val slop = floor(8f * density + 0.5f)
        val (ux, uy) = (from.x - dial.x) to (from.y - dial.y)
        val length = hypot(ux, uy)
        val out = PointF(from.x + ux / length * (slop + 4f), from.y + uy / length * (slop + 4f))
        val end = PointF(to.x + ux / length * slop, to.y + uy / length * slop)
        val path = listOf(from, out) + (1..6).map { k -> PointF(out.x + (end.x - out.x) * k / 6f, out.y + (end.y - out.y) * k / 6f) }
        drag(path)
    }

    /** MotionEvents straight to the Rust view, from screen pixels: down, moves, up where the last move was. */
    private fun drag(path: List<PointF>) {
        val view = host()
        h.main {
            val origin = IntArray(2).also(view::getLocationOnScreen)
            val start = SystemClock.uptimeMillis()
            val events = path.mapIndexed { i, p -> Triple(if (i == 0) MotionEvent.ACTION_DOWN else MotionEvent.ACTION_MOVE, p, i) } +
                Triple(MotionEvent.ACTION_UP, path.last(), path.size)
            for ((action, p, i) in events) {
                val event = MotionEvent.obtain(start, start + 16L * i, action, p.x - origin[0], p.y - origin[1], 0)
                event.source = InputDevice.SOURCE_TOUCHSCREEN
                view.onTouchEvent(event)
                event.recycle()
            }
        }
        SystemClock.sleep(200)
    }

    /** Text into the focused field, replacing what it holds, once the keyboard is up for it. */
    private fun type(text: String) {
        h.waitFor("the field did not take focus") { var f = false; h.main { f = host().inputHasFocus }; f }
        imeShown()
        h.main {
            val c = connection()
            // A commit replaces the composing text if there is any, else the selection, and the
            // keyboard marks the word at the cursor as composing; so the whole text is made the
            // composing region.
            check(c.setComposingRegion(0, host().editorText.length))
            check(c.commitText(text, 1))
        }
        SystemClock.sleep(150)
    }
    private fun connection(): InputConnection = checkNotNull(h.views(SettingsInputView::class.java).single().onCreateInputConnection(EditorInfo()))
    private fun imeShown() = h.waitFor("the keyboard did not show") { var shown = false; h.main { shown = h.ime()?.isInputViewShown == true }; shown }

    /** A colour or time dialog open across a rotation: closed afterwards, as Compose's remember left it. */
    private fun closedByRotation(row: String) {
        rotate(UiAutomation.ROTATION_FREEZE_90)
        // The row may be below the fold in landscape; the app bar's Back is exported only while no
        // dialog is open, so it shows the screen came back without one.
        h.waitFor("rotation left the dialog open") { h.find("Cancel") == null && h.find("Back") != null }
        rotate(UiAutomation.ROTATION_FREEZE_0)
        h.reveal(row)
        check(h.find("Cancel") == null) { "the dialog came back with portrait" }
        result.put("rotation_closes_${row.substringBefore(' ').lowercase()}", true)
    }
    /** A list dialog open across a rotation: still open, as every other dialog is restored. */
    private fun listSurvivesRotation() {
        stage("theme:list:rotation")
        h.reveal("Theme mode")
        h.click("Theme mode")
        h.waitFor("mode dialog did not open") { h.find("Cancel") != null }
        rotate(UiAutomation.ROTATION_FREEZE_90)
        h.waitFor("rotation closed the list dialog") { h.find("Cancel") != null && h.find("Follow time") != null }
        rotate(UiAutomation.ROTATION_FREEZE_0)
        h.waitFor("the list dialog did not survive") { h.find("Cancel") != null }
        h.back()
        h.waitFor("Back did not close the list dialog") { h.find("Cancel") == null }
        result.put("rotation_keeps_list", true)
    }
    private fun rotate(rotation: Int) {
        val old = runner.resumedActivity
        check(h.automation.setRotation(rotation))
        h.waitFor("rotation did not recreate the activity") { runner.resumedActivity.let { it != null && it !== old } }
        h.activity = runner.resumedActivity
        h.waitFor("no Rust view after rotation") { h.views(RustSettingsView::class.java).size == 1 }
        h.settle("rotation did not settle")
    }

    /**
     * The keyboard's colours follow the Theme accent: with a Material You theme active, the
     * keyboard drawn under one preset, another, then the first again. The second must differ and
     * the third match the first, so the difference is the accent's and not the keyboard's own.
     */
    private fun keyboardAccent() {
        stage("theme:keyboard-accent")
        val red = keyboardWith("#f44336", 1)
        val blue = keyboardWith("#2196f3", 2)
        val again = keyboardWith("#f44336", 3)
        hideIme()
        val changed = difference(red, blue)
        val repeat = difference(red, again)
        result.put("keyboard_accent", JSONObject().put("changed_fraction", changed).put("repeat_fraction", repeat))
        if (changed <= 0.01 || repeat >= 0.002) {
            h.lastStep = "keyboard colours did not follow the accent (changed %.4f, repeat %.4f)".format(changed, repeat)
            error("keyboard colours did not follow the accent")
        }
    }
    private fun keyboardWith(preset: String, capture: Int): IntArray {
        val row = "Accent color (Material You themes)"
        openColour(row)
        page("Presets")
        h.click(preset)
        confirm(row)
        h.top()
        // The keyboard hid when the dialog opened. Shown again within about a second of hiding,
        // the keyboard never re-attaches its key surface and stays blank, accent or no accent
        // (owed: docs/stage4-other-theme.md), so this waits that out to keep the check on the accent.
        SystemClock.sleep(1500)
        val field = checkNotNull(h.nodes().firstOrNull { it.isEditable && h.label(it) == "Try out your setup" }) { "preview field absent" }
        check(field.performAction(Node.ACTION_CLICK))
        imeShown()
        return keyboardPixels(capture)
    }
    /**
     * The keyboard's own pixels, once its keys are drawn and two screenshots 150 ms apart agree:
     * the band at the bottom of the app's window that the keyboard's inset covers. (The
     * keyboard's window, and its input area, reach over the whole app, whose blinking cursor
     * would count as a change.)
     */
    private fun keyboardPixels(capture: Int): IntArray {
        var last: IntArray? = null
        val shades = mutableListOf<Int>()
        repeat(30) {
            val r = Rect()
            h.main {
                val root = host().rootView
                val keyboard = root.rootWindowInsets?.getInsets(android.view.WindowInsets.Type.ime())?.bottom ?: 0
                val at = IntArray(2).also(root::getLocationOnScreen)
                if (keyboard > 0) r.set(at[0], at[1] + root.height - keyboard, at[0] + root.width, at[1] + root.height)
            }
            if (!r.isEmpty) {
                val shot = checkNotNull(h.automation.takeScreenshot())
                val soft = if (shot.config == Bitmap.Config.HARDWARE) shot.copy(Bitmap.Config.ARGB_8888, false) else shot
                val pixels = IntArray(r.width() * r.height())
                soft.getPixels(pixels, 0, r.width(), r.left, r.top, r.width(), r.height())
                if (soft !== shot) soft.recycle()
                shot.recycle()
                // Drawn keys, not the blank window shown before the first frame: anti-aliased key
                // labels give a drawn keyboard several hundred shades (about 870 here), a blank
                // one only its background and the navigation icons' (about 250).
                val drawn = pixels.distinct().size.also(shades::add) >= 500
                if (drawn && last?.contentEquals(pixels) == true) return pixels
                last = pixels
            }
            SystemClock.sleep(150)
        }
        h.lastStep = "keyboard capture $capture did not settle (shades ${shades.joinToString()})"
        error("the keyboard did not settle")
    }
    /** The keyboard hides itself, as its own hide key does. (UiAutomation's global Back does not reach it.) */
    private fun hideIme() {
        h.main { checkNotNull(h.ime()).requestHideSelf(0) }
        h.waitFor("the keyboard did not hide") {
            var hidden = false
            h.main {
                hidden = h.ime()?.isInputViewShown != true &&
                    host().rootWindowInsets?.isVisible(android.view.WindowInsets.Type.ime()) != true
            }
            hidden
        }
    }
    private fun difference(a: IntArray, b: IntArray): Double {
        check(a.size == b.size) { "the keyboard changed size" }
        return a.indices.count { a[it] != b[it] }.toDouble() / a.size
    }

    private fun onRust(row: String) {
        h.waitFor("$row is not on a Rust screen") {
            h.activity = runner.resumedActivity
            !fallback() && h.views(RustSettingsView::class.java).size == 1 && h.find(row) != null
        }
    }
    private fun fallback() = runner.resumedActivity?.javaClass?.simpleName == "ComposeSettingsFallbackActivity"
    private fun host() = h.views(RustSettingsView::class.java).single()
    private fun node(label: String) = checkNotNull(h.find(label)) { "absent: $label" }
    private fun bounds(n: Node) = Rect().also(n::getBoundsInScreen)
    private fun centre(n: Node) = bounds(n).let { PointF(it.exactCenterX(), it.exactCenterY()) }
    private fun px(dp: Float) = (dp * density).roundToInt()
}
