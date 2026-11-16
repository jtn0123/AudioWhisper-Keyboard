package com.jtn0123.audiowhisper.settingsui

import android.graphics.Rect
import android.view.accessibility.AccessibilityNodeInfo as Node
import org.json.JSONArray
import org.json.JSONObject
import java.security.MessageDigest
import kotlin.math.roundToInt

/**
 * The third-party licences screen on both renderers: every library row from top to bottom (by
 * its first line, which is the library's name on both), audits of the top, a lower page and a
 * license dialog, and the dialog's text (as a SHA-256: licence text is public, but long) and
 * button's role. The project licence screen stays in Compose on both renderers; Rust hands it back.
 *
 * The walk pages with TalkBack's scroll forward, which moves both lists by their own height, and
 * waits for Compose's animated page to stop. A row is counted on the page where its name line is
 * inside the list: Compose reports a row clipped by the list's top at its unclipped bounds but
 * labels it from its first visible child, so a row whose name is scrolled away is not named by it.
 */
internal class SettingsLicencesDevice(private val h: SettingsHarness, private val stage: (String) -> Unit) {
    private val title = "Third-party licenses"
    private val chrome = setOf("Back", "Switch Keyboard")
    /** The name's top below the row's: AboutLibraries' 14 dp of padding, on both renderers. */
    private val nameTop = (14 * h.instrumentation.targetContext.resources.displayMetrics.density).roundToInt()

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
        // The name, while the row is on screen: on Compose, a node kept from here read back an
        // empty name once the walk had scrolled the row away.
        val opened = name(rows().first())
        stage("licences:walk")
        val names = mutableListOf<String>()
        var pages = 0
        while (true) {
            for (name in rows().map(::name)) if (name !in names) names.add(name)
            if (!h.scroll(true)) break
            h.settle("licences page did not settle")
            pages++
            if (pages == 2) out.put(h.audit("licences-lower"))
        }
        h.top()
        h.settle("licences did not return to the top")
        stage("licences:dialog")
        h.click(opened)
        h.waitFor("license dialog did not open: $opened") { h.find("OK") != null }
        out.put(h.audit("licences:dialog"))
        val ok = checkNotNull(h.find("OK"))
        // The text nodes themselves: a container's label joins its children's, and the two
        // renderers nest the dialog differently.
        val text = h.nodes().filter { it != ok && it.childCount == 0 && h.label(it).length > 40 && h.label(it) != opened }
            .map { sha256(h.label(it)) }.distinct().sorted()
        val dialog = JSONObject()
            .put("row", opened)
            .put("button", h.label(ok))
            .put("button_role", h.role(ok))
            .put("text_sha256", JSONArray(text))
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
    /**
     * Library rows on screen: clickable, labelled, not the top bar, not inside another row, and
     * with their name line inside the list.
     */
    private fun rows(): List<Node> {
        val all = h.nodes()
        val list = all.filter { it.isScrollable }.maxByOrNull { bounds(it).let { b -> b.width().toLong() * b.height() } }
        val area = list?.let(::bounds) ?: return emptyList()
        val rows = all.filter { it.isClickable && !it.isEditable && !it.isScrollable && h.label(it).isNotBlank() && name(it) !in chrome && name(it) != title }
        return rows.filter { n -> rows.none { o -> o != n && bounds(o) != bounds(n) && bounds(o).contains(bounds(n)) } }
            .filter { (bounds(it).top + nameTop).let { y -> y >= area.top && y < area.bottom } }
            .sortedBy { bounds(it).top }
    }
    private fun sha256(text: String) = MessageDigest.getInstance("SHA-256").digest(text.toByteArray()).joinToString("") { "%02x".format(it) }
}
