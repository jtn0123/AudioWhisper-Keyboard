package dev.patrickgold.florisboard.ime.nlp

import kotlin.test.Test
import kotlin.test.assertEquals

class NlpBindingMatchTest {
    @Test fun providerlessGlideWordsUseNativeWordEligibility() {
        val context = arrayOf("1", "0", "org.florisboard.nlp.providers.latin", "en",
            "0", "1", "1", "0", "1", "1", "4", "4", "0", "4", "helo", "en", "", "1", "1", "0", "0", "0")
        val glide = WordSuggestionCandidate("hello") // Glide has no sourceProvider object.
        try {
            NlpState.configure(context)
            NlpState.directly(listOf(glide))
            assertEquals(emptyList(), NlpState.candidates.value, "Emoji enabled must not enable glide word candidates")
            context[4] = "1"
            NlpState.configure(context)
            NlpState.directly(listOf(glide, WordSuggestionCandidate("hello"), WordSuggestionCandidate("help")))
            assertEquals(listOf("hello", "help"), NlpState.candidates.value.map { it.text.toString() })
            context[0] = "129"
            NlpState.configure(context)
            NlpState.directly(listOf(glide))
            assertEquals(emptyList(), NlpState.candidates.value, "A password cannot expose providerless words")
        } finally {
            NlpState.field()
            NlpState.configure(Array(22) { "0" })
        }
    }
}
