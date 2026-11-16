package com.jtn0123.audiowhisper.settingsui

import com.mikepenz.aboutlibraries.Libs
import com.mikepenz.aboutlibraries.ui.compose.util.author
import com.mikepenz.aboutlibraries.ui.compose.util.htmlReadyLicenseContent
import org.ccil.cowan.tagsoup.HTMLSchema
import org.ccil.cowan.tagsoup.Parser
import org.xml.sax.Attributes
import org.xml.sax.InputSource
import org.xml.sax.helpers.DefaultHandler
import java.io.StringReader
import java.util.Locale

/**
 * ThirdPartyLicensesScreen's decisions, from AboutLibraries 15.3.0 itself where the JVM can run
 * it and transcribed where it cannot. No production code.
 *
 * - The libraries: `Libs.Builder().withJson(json).build()` — the real AndroidParser, HashSet and
 *   `sortedBy { name.lowercase() }` — over desktop org.json.
 * - Each Traditional row (LibraryDefaultComponents and TraditionalRow): the name; the version
 *   when not blank; the author line when not blank; a badge per licence, in set order.
 * - The license dialog's text: `AnnotatedString.fromHtml(htmlReadyLicenseContent)` when that is
 *   not empty. Compose prefixes `<ContentHandlerReplacementTag />` and calls Html.fromHtml with
 *   FROM_HTML_MODE_COMPACT, which parses with TagSoup 1.2; [text] transcribes android16's
 *   HtmlToSpannedConverter for the characters it produces (spans are not compared).
 */
internal object ReferenceLicenceScreens {
    data class Row(val name: String, val version: String?, val author: String?, val badges: List<String>, val dialog: String?)

    fun rows(json: String): List<Row> = Libs.Builder().withJson(json).build().libraries.map { library ->
        val content = library.htmlReadyLicenseContent
        Row(
            name = library.name,
            version = library.artifactVersion?.takeIf { it.isNotBlank() },
            author = library.author.takeIf { it.isNotBlank() },
            badges = library.licenses.map { it.name },
            dialog = content.takeIf { it.isNotEmpty() }?.let { text("<ContentHandlerReplacementTag />$it") },
        )
    }

    private val schema = HTMLSchema()

    /** Html.fromHtml(COMPACT)'s characters for `html`. */
    fun text(html: String): String {
        val handler = Converter()
        val parser = Parser()
        parser.setProperty(Parser.schemaProperty, schema)
        parser.contentHandler = handler
        parser.parse(InputSource(StringReader(html)))
        return handler.out.toString()
    }

    /** HtmlToSpannedConverter's text: block tags keep one line apart, `br` breaks, `img` is U+FFFC. */
    private class Converter : DefaultHandler() {
        val out = StringBuilder()
        private var open = 0
        private fun newlines(min: Int) {
            if (out.isEmpty()) return
            val existing = out.length - out.trimEnd('\n').length
            repeat(min - existing) { out.append('\n') }
        }
        private fun block(tag: String): Boolean {
            val t = tag.lowercase(Locale.ROOT)
            return t in listOf("p", "ul", "li", "div", "blockquote") || (t.length == 2 && t[0] == 'h' && t[1] in '1'..'6')
        }
        override fun startElement(uri: String, local: String, qName: String, attributes: Attributes) {
            if (block(local)) { newlines(1); open++ }
            else if (local.equals("img", ignoreCase = true)) out.append('￼')
        }
        override fun endElement(uri: String, local: String, qName: String) {
            if (local.equals("br", ignoreCase = true)) out.append('\n')
            else if (block(local) && open > 0) { open--; newlines(1) }
        }
        override fun characters(ch: CharArray, start: Int, length: Int) {
            val sb = StringBuilder()
            for (i in 0 until length) {
                val c = ch[i + start]
                if (c == ' ' || c == '\n') {
                    val pred = if (sb.isEmpty()) out.lastOrNull() ?: '\n' else sb.last()
                    if (pred != ' ' && pred != '\n') sb.append(' ')
                } else sb.append(c)
            }
            out.append(sb)
        }
    }
}
