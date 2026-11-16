package com.jtn0123.audiowhisper.settingsui

import android.graphics.Rect
import android.view.accessibility.AccessibilityNodeInfo as Node
import org.json.JSONArray
import org.json.JSONObject
import java.security.MessageDigest

/**
 * The third-party licences screen on both renderers: every library row from top to bottom (by
 * its first line, which is the library's name on both), audits of the top, a lower page and a
 * license dialog, and the dialog's text (as a SHA-256: licence text is public, but long) and
 * button. The project licence screen stays in Compose on both renderers; Rust hands it back.
 */
internal class SettingsLicencesDevice(private val h: SettingsHarness, private val stage: (String) -> Unit) {
    private val title = "Third-party licenses"
    private val chrome = setOf("Back", "Switch Keyboard")

    fun run(report: JSONObject, out: JSONArray) {
        stage("licences:open")
        h.top()
        h.reveal("About")
        h.click("About")
        h.settle()
        h.reveal(title)
        h.click(title)
        h.settle()
        h.waitFor("licences did not open") { h.find("Version") == null && rows().isNotEmpty() }
        val rust = h.views(RustSettingsView::class.java).isNotEmpty()
        if (rust) {
            val resumed = (h.instrumentation as SettingsInstrumentation).resumedActivity
            check(resumed?.javaClass?.simpleName != "ComposeSettingsFallbackActivity") { "licences fell back to Compose" }
        }
        out.put(h.audit("licences-top"))
        val first = rows().first()
        stage("licences:walk")
        val names = mutableListOf<String>()
        var pages = 0
        while (true) {
            for (name in rows().map(::name)) if (name !in names) names.add(name)
            if (!h.scroll(true)) break
            pages++
            if (pages == 2) out.put(h.audit("licences-lower"))
        }
        h.top()
        stage("licences:dialog")
        val opened = name(first)
        h.click(opened)
        h.waitFor("license dialog did not open: $opened") { h.find("OK") != null }
        out.put(h.audit("licences:dialog"))
        val ok = checkNotNull(h.find("OK"))
        val text = h.nodes().filter { it != ok && h.label(it).length > 40 }.map { h.label(it) }
        val dialog = JSONObject()
            .put("row", opened)
            .put("button", h.label(ok))
            .put("button_class", ok.className?.toString().orEmpty())
            .put("text_sha256", JSONArray(text.map(::sha256)))
        h.click("OK")
        h.waitFor("license dialog did not close") { h.find("OK") == null }
        h.back()
        h.settle("did not return from the licences")
        h.waitFor("About did not return") { h.find("Version") != null }
        h.back()
        h.settle()
        report.put("licences", JSONObject().put("rows", JSONArray(names)).put("dialog", dialog))
    }

    private fun bounds(n: Node) = Rect().also(n::getBoundsInScreen)
    /** A row's name: its first line (Compose merges the row's texts; Rust labels it by name). */
    private fun name(n: Node) = h.label(n).lines().first()
    /** Library rows on screen: clickable, labelled, not the top bar, not inside another row. */
    private fun rows(): List<Node> {
        val rows = h.nodes().filter { it.isClickable && !it.isEditable && h.label(it).isNotBlank() && name(it) !in chrome && name(it) != title }
        return rows.filter { n -> rows.none { o -> o != n && bounds(o) != bounds(n) && bounds(o).contains(bounds(n)) } }
            .sortedBy { bounds(it).top }
    }
    private fun sha256(text: String) = MessageDigest.getInstance("SHA-256").digest(text.toByteArray()).joinToString("") { "%02x".format(it) }
}
