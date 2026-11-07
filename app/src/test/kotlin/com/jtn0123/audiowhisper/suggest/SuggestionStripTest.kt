package com.jtn0123.audiowhisper.suggest

import java.io.File
import kotlin.test.Test
import kotlin.test.assertTrue

class SuggestionStripTest {
    @Test
    fun `every bundled theme shows the word space will apply in bold`() {
        val stylesheets = File("src/main/assets/ime/theme").walk().filter { it.extension == "json" && it.parentFile?.name == "stylesheets" }.toList()
        assertTrue(stylesheets.size >= 12, "found ${stylesheets.size} stylesheets")
        for (sheet in stylesheets) {
            val text = sheet.readText()
            assertTrue("\"smartbar-candidate-word-text[auto-commit=1]\"" in text, "${sheet.name} has no auto-commit rule")
        }
    }
}
