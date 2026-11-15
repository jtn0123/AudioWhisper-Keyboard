package com.jtn0123.audiowhisper.settingsui

import android.app.Instrumentation
import android.content.Intent
import android.graphics.Rect
import android.provider.Settings
import android.view.accessibility.AccessibilityNodeInfo as Node
import org.json.JSONArray
import org.json.JSONObject

/**
 * The group 2 screens on both renderers: Typing (its spell checker card, auto-space card and the
 * forget-learned-words confirmation), User dictionaries, Physical keyboard, About and Dictation.
 * Every enabled row is exercised top to bottom, as in group 1. A row that opens another app is
 * clicked under a blocking monitor, so the walk records which intent each renderer fired and
 * nothing leaves the app; a row that opens a screen is audited there and backed out of.
 *
 * Not clicked: About's version row, which writes the clipboard and its history (neither is in the
 * device baseline), and the vocabulary field, which SettingsEditorDevice edits.
 */
internal class SettingsGroupTwoDevice(private val h: SettingsHarness, private val stage: (String) -> Unit) {
    /** `ensureOn`: masters switched on, when off, for a second pass over the rows they enable. */
    private class Screen(val name: String, val path: List<String>, val title: String, val ensureOn: List<String> = emptyList())
    private val chrome = setOf("Back", "Switch Keyboard")
    private val notClicked = setOf("Version")
    /**
     * Rows leading to screens that stay in Compose on both renderers (Rust hands them to
     * ComposeSettingsFallbackActivity). Their routes are walked but not audited: the audit covers
     * the settings UI that Rust draws. On Rust the list checks itself against the activity.
     */
    private val outside = setOf("Manage system user dictionary", "Manage internal user dictionary",
        "Project license", "Third-party licenses")
    private val screens = listOf(
        Screen("typing", listOf("Typing"), "Typing", ensureOn = listOf("Display suggestions", "Auto-space after punctuation")),
        Screen("dictionary", listOf("Typing", "User dictionaries"), "User dictionaries",
            ensureOn = listOf("Enable system user dictionary", "Enable internal user dictionary")),
        Screen("physical-keyboard", listOf("Other", "Physical keyboard"), "Physical keyboard"),
        Screen("about", listOf("About"), "About"),
        Screen("dictation", listOf("Dictation"), "Dictation"),
    )

    /** Blocks what leaves the app (a browser, system settings) and records it; lets the app's own pass. */
    private class Launches(private val pkg: String) : Instrumentation.ActivityMonitor() {
        val seen = mutableListOf<String>()
        override fun onStartActivity(intent: Intent): Instrumentation.ActivityResult? {
            val web = intent.action == Intent.ACTION_VIEW && intent.data?.scheme in setOf("http", "https")
            val external = web || intent.action == Settings.ACTION_HARD_KEYBOARD_SETTINGS ||
                (intent.component != null && intent.component?.packageName != pkg)
            if (!external) return null
            // Public URLs and system component names only.
            synchronized(seen) {
                seen.add(listOfNotNull(intent.action, intent.data?.toString(), intent.component?.flattenToShortString()).joinToString(" "))
            }
            return Instrumentation.ActivityResult(0, null)
        }
        fun count() = synchronized(seen) { seen.size }
        fun last() = synchronized(seen) { seen.last() }
    }

    private lateinit var out: JSONArray
    private lateinit var log: JSONArray
    private lateinit var launches: Launches
    private val seenDisabled = mutableSetOf<String>()
    private val seenEnabled = mutableSetOf<String>()
    /** Each later line of a row's label, to that row's title. */
    private val titles = mutableMapOf<String, String>()
    private var rust = false

    fun run(report: JSONObject, out: JSONArray) {
        this.out = out
        launches = Launches(h.instrumentation.targetContext.packageName)
        h.instrumentation.addMonitor(launches)
        val result = JSONObject()
        try {
            for (screen in screens) {
                stage("group-two:${screen.name}")
                open(screen)
                log = JSONArray()
                seenDisabled.clear()
                seenEnabled.clear()
                titles.clear()
                walk(screen)
                result.put(screen.name, JSONObject().put("exercised", log)
                    .put("never_enabled", JSONArray((seenDisabled - seenEnabled).sorted())))
                close()
            }
        } finally {
            h.instrumentation.removeMonitor(launches)
        }
        report.put("group_two", result.put("not_clicked", JSONArray(notClicked.sorted())))
    }

    private fun open(screen: Screen) {
        h.top()
        for (step in screen.path) {
            h.reveal(step)
            h.click(step)
            h.settle()
        }
        h.waitFor("${screen.name} did not open") { h.find("Languages & Layouts") == null && h.find(screen.title) != null }
        rust = h.views(RustSettingsView::class.java).isNotEmpty()
        out.put(h.audit("two:${screen.name}-top"))
    }

    /** Back until home: a screen reached through Other or Typing is two screens deep. */
    private fun close() {
        repeat(3) {
            if (h.find("Languages & Layouts") != null) return
            h.back()
            h.settle()
        }
        h.waitFor("home did not return") { h.find("Languages & Layouts") != null }
    }

    private fun walk(screen: Screen) {
        val done = mutableSetOf<String>()
        pass(screen, done)
        out.put(h.audit("two:${screen.name}-bottom"))
        val switched = screen.ensureOn.filter { !checked(it) }.onEach { flip(it) }
        pass(screen, done)
        out.put(h.audit("two:${screen.name}-expanded"))
        h.top()
        for (label in switched.reversed()) flip(label)
    }

    private fun pass(screen: Screen, done: MutableSet<String>) {
        h.top()
        while (true) {
            val row = next(done)
            if (row == null) { if (!h.scroll(true)) return; continue }
            done.add(row)
            if (row in notClicked) {
                log.put(JSONObject().put("row", row).put("kind", "not-clicked"))
                continue
            }
            exercise(screen, row)
        }
    }

    private fun bounds(n: Node) = Rect().also(n::getBoundsInScreen)

    /**
     * A row by its title. Compose leaves a child scrolled out of view out of its row's node, so a
     * row whose title is above the viewport is labelled from its summary down; that row is the
     * one whose whole label held the summary. Rust labels a row whole however it is clipped.
     */
    private fun primary(n: Node): String {
        val lines = h.label(n).lines()
        val title = titles[lines.first()] ?: lines.first()
        lines.drop(1).forEach { titles.putIfAbsent(it, title) }
        return title
    }

    private fun next(done: Set<String>): String? {
        // A disabled row is clickable on Compose, as a disabled View is, but not on Rust; neither
        // offers ACTION_CLICK. Disabled nodes count as rows on both, so both see Languages.
        val rows = h.nodes().filter { (it.isClickable || !it.isEnabled) && !it.isEditable && it.rangeInfo == null && h.label(it).isNotBlank() && primary(it) !in chrome }
        val own = rows.filter { n -> rows.none { o -> o != n && bounds(o) != bounds(n) && bounds(o).contains(bounds(n)) } }
        for (n in own) if (n.isEnabled) seenEnabled.add(primary(n)) else seenDisabled.add(primary(n))
        return own.filter { it.isEnabled && primary(it) !in done }.minByOrNull { bounds(it).top }?.let(::primary)
    }

    private fun checked(label: String): Boolean {
        if (h.find(label) == null) h.top()
        h.reveal(label)
        return checkNotNull(h.find(label)).checked == Node.CHECKED_STATE_TRUE
    }

    private fun exercise(screen: Screen, label: String) {
        val row = checkNotNull(h.find(label))
        if (row.isCheckable) {
            val before = row.checked == Node.CHECKED_STATE_TRUE
            h.click(label)
            h.waitFor("toggle did not change: $label") { checked(label) != before }
            h.click(label)
            h.waitFor("toggle did not restore: $label") { checked(label) == before }
            log.put(JSONObject().put("row", label).put("kind", "toggle"))
            return
        }
        val fired = launches.count()
        h.click(label)
        h.waitFor("row did nothing: $label") { h.find("Cancel") != null || launches.count() > fired || h.find(screen.title) == null }
        when {
            launches.count() > fired -> {
                check(launches.count() == fired + 1) { "one row, one launch" }
                log.put(JSONObject().put("row", label).put("kind", "launch").put("intent", launches.last()))
            }
            h.find("Cancel") == null -> {
                val resumed = (h.instrumentation as SettingsInstrumentation).resumedActivity
                if (rust) check((resumed?.javaClass?.simpleName == "ComposeSettingsFallbackActivity") == (label in outside)) {
                    "outside the port, or not: $label"
                }
                if (label !in outside) out.put(h.audit("two:${screen.name}>$label"))
                h.back()
                // Until the popped screen's exit ends, its rows are still nodes the walk would take.
                h.settle("did not return from: $label")
                h.waitFor("did not return from: $label") { h.find(screen.title) != null }
                log.put(JSONObject().put("row", label).put("kind", "route"))
            }
            h.find("Delete") != null -> {
                out.put(h.audit("two:${screen.name}:$label"))
                confirm(label)
                log.put(JSONObject().put("row", label).put("kind", "confirm"))
            }
            else -> {
                out.put(h.audit("two:${screen.name}:$label"))
                h.exerciseDialog(label)
                h.click("OK")
                h.waitFor("dialog did not close: $label") { h.find("Cancel") == null }
                log.put(JSONObject().put("row", label).put("kind", "dialog"))
            }
        }
    }

    /** ForgetLearnedWordsDialog: Cancel keeps the words, Delete erases them (the baseline restores them). */
    private fun confirm(label: String) {
        check(h.find("Default") == null && h.find("OK") == null)
        h.click("Cancel")
        h.waitFor("confirmation did not dismiss: $label") { h.find("Delete") == null }
        h.click(label)
        h.waitFor("confirmation not reopened: $label") { h.find("Delete") != null }
        h.click("Delete")
        h.waitFor("confirmation did not accept: $label") { h.find("Delete") == null }
    }

    private fun flip(label: String) {
        h.top()
        val before = checked(label)
        h.click(label)
        h.waitFor("master did not change: $label") { checked(label) != before }
        log.put(JSONObject().put("row", label).put("kind", "flip").put("to", !before))
    }
}
