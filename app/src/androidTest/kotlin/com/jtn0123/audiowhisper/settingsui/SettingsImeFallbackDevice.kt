package com.jtn0123.audiowhisper.settingsui

import android.app.Dialog
import android.os.ParcelFileDescriptor
import android.os.Process
import android.os.SystemClock
import android.provider.Settings
import android.view.accessibility.AccessibilityNodeInfo as Node
import android.view.accessibility.AccessibilityWindowInfo
import android.view.inputmethod.InputMethodManager
import android.view.textservice.TextInfo
import android.widget.EditText
import dev.patrickgold.florisboard.R
import org.json.JSONObject

/**
 * The keyboard in a process whose native engine failed to load (crash mode). The system binds
 * FlorisImeService into this same process, so showing the keyboard here is the real failure
 * path: it must show the platform notice, survive show/hide cycles and never crash-loop.
 */
internal class SettingsImeFallbackDevice(private val h: SettingsHarness) {
    private val context = h.instrumentation.targetContext
    private val imm = context.getSystemService(InputMethodManager::class.java)

    fun run(report: JSONObject) {
        val ours = selected()
        h.lastStep = "our keyboard is not the selected input method"
        check(ours?.startsWith("${context.packageName}/") == true)
        // Switching keyboards rewrites these; the device baseline does not track them.
        val subtypes = SUBTYPE_KEYS.associateWith { shell("settings get secure $it").trim() }
        val notice = context.getString(R.string.crash__keyboard_unavailable)
        val switch = context.getString(R.string.crash__switch_keyboard)
        val pid = Process.myPid()
        lateinit var field: EditText
        var dialog: Dialog? = null
        try {
            rebind(checkNotNull(ours))
            h.main {
                val activity = checkNotNull(h.activity)
                field = EditText(activity)
                dialog = Dialog(activity).apply { setContentView(field); show() }
            }
            h.waitFor("fallback field never focused") { field.hasWindowFocus() }
            val cycles = 3
            repeat(cycles) { cycle ->
                var asked = 0L
                h.waitFor("keyboard notice absent (cycle $cycle)") {
                    if (SystemClock.uptimeMillis() - asked > 1000) {
                        asked = SystemClock.uptimeMillis()
                        h.main { field.requestFocus(); imm.showSoftInput(field, 0) }
                    }
                    keyboard().any { h.label(it) == notice }
                }
                val buttons = keyboard().filter { it.className == "android.widget.Button" }
                h.lastStep = "keyboard notice has no single switch button (cycle $cycle)"
                check(buttons.size == 1 && h.label(buttons.single()) == switch)
                h.main { imm.hideSoftInputFromWindow(field.windowToken, 0) }
                h.waitFor("keyboard did not hide (cycle $cycle)") { keyboard().isEmpty() }
            }
            h.lastStep = "process restarted"
            check(Process.myPid() == pid)
            report.put("ime_fallback_notice", true).put("ime_show_hide_cycles", cycles).put("ime_process_survived", true)
            // The spell checker shares the process; with no engine it offers no opinion.
            h.lastStep = "spell checker answered without an engine"
            val session = dev.patrickgold.florisboard.FlorisSpellCheckerService().createSession()
            session.onCreate()
            check(session.onGetSuggestions(TextInfo("probe"), 3).suggestionsCount == 0)
            report.put("spell_checker_inert", true)
        } finally {
            h.main { dialog?.dismiss() }
            if (selected() != ours) shell("ime set $ours")
            for ((key, value) in subtypes) shell("settings put secure $key $value")
        }
        h.lastStep = "keyboard subtypes not restored"
        check(SUBTYPE_KEYS.all { shell("settings get secure $it").trim() == subtypes[it] })
        report.put("ime_subtypes_restored", true)
    }

    private fun selected(): String? =
        Settings.Secure.getString(context.contentResolver, Settings.Secure.DEFAULT_INPUT_METHOD)

    /**
     * `am instrument` force-stops the package without the broadcast that makes the system bind
     * its keyboard again, so nothing would serve the field. Switching to another enabled
     * keyboard and back binds ours again, into this crash-mode process.
     */
    private fun rebind(ours: String) {
        h.lastStep = "no other keyboard is enabled"
        val other = imm.enabledInputMethodList.map { it.id }.first { it != ours }
        for (id in listOf(other, ours)) {
            shell("ime set $id")
            h.waitFor("keyboard selection did not change") { selected() == id }
        }
    }

    private fun shell(command: String): String =
        ParcelFileDescriptor.AutoCloseInputStream(h.automation.executeShellCommand(command)).use { it.readBytes() }.decodeToString()

    private companion object {
        val SUBTYPE_KEYS = listOf("selected_input_method_subtype", "input_methods_subtype_history")
    }

    /** Visible nodes of the input-method window, which the system lists only while it is shown. */
    private fun keyboard(): List<Node> {
        h.automation.clearCache()
        val out = mutableListOf<Node>()
        fun walk(node: Node?) {
            if (node == null) return
            if (node.isVisibleToUser) out.add(node)
            for (i in 0 until node.childCount) walk(node.getChild(i))
        }
        h.automation.windows.filter { it.type == AccessibilityWindowInfo.TYPE_INPUT_METHOD }.forEach { walk(it.root) }
        return out
    }
}
