package dev.patrickgold.florisboard.ime.keyboard

import java.io.File
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/**
 * The toasts the keyboard shows while typing take their text from string resources, so it can be
 * translated and reworded in one place. Reads the sources, as these toasts need a running keyboard.
 */
class KeyboardToastTextTest {
    private val sources = File("src/main/kotlin/dev/patrickgold/florisboard/ime/keyboard")
    private val toast = Regex("""show(?:Long|Short)Toast(?:Sync)?\(\s*([^)]*)""")

    @Test
    fun `no toast the keyboard shows has its text written in the code`() {
        val files = sources.walk().filter { it.extension == "kt" }.toList()
        assertTrue(files.isNotEmpty(), "no sources under $sources")
        val calls = files.flatMap { file -> toast.findAll(file.readText()).map { file.name to it.groupValues[1] } }
        assertTrue(calls.any { it.first == "KeyboardEditing.kt" }, "the autocorrect key's toast moved")
        val literal = calls.filter { it.second.trimStart().startsWith("\"") }
        assertEquals(emptyList<Pair<String, String>>(), literal)
    }

    @Test
    fun `the autocorrect key's toast reads from its resource`() {
        val editing = File(sources, "KeyboardEditing.kt").readText()
        val handler = editing.substringAfter("fun KeyboardManager.handleToggleAutocorrect()").substringBefore("\n}\n")
        assertTrue("R.string.autocorrect_toggle__toast_unavailable" in handler, handler)
        val strings = File("src/main/res/values/strings.xml").readText()
        val text = Regex("""<string name="autocorrect_toggle__toast_unavailable"[^>]*>([^<]+)</string>""")
            .find(strings)?.groupValues?.get(1)
        assertTrue(text?.contains("Typing") == true, "the toast says where auto-correction is set: $text")
        // Android shows only two lines of a toast; on a 411 dp wide screen the longer first wording
        // was cut off at 75 characters, before it said where to go.
        assertTrue(text!!.length <= 60, "the toast fits in two lines: ${text.length} characters")
    }
}
