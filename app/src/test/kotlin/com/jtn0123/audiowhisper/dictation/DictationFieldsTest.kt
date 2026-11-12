package com.jtn0123.audiowhisper.dictation

import dev.patrickgold.florisboard.ime.editor.InputAttributes.Type
import dev.patrickgold.florisboard.ime.editor.InputAttributes.Variation
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class DictationFieldsTest {
    @Test
    fun `dictates into ordinary text fields and raw-key fields`() {
        for (variation in listOf(Variation.NORMAL, Variation.SHORT_MESSAGE, Variation.LONG_MESSAGE, Variation.EMAIL_SUBJECT)) {
            assertTrue(DictationFields.allows(Type.TEXT, variation), "$variation")
        }
        assertTrue(DictationFields.allows(Type.NULL, Variation.NORMAL))
    }

    @Test
    fun `never dictates a password, visible or not`() {
        for (variation in listOf(Variation.PASSWORD, Variation.VISIBLE_PASSWORD, Variation.WEB_PASSWORD)) {
            assertFalse(DictationFields.allows(Type.TEXT, variation), "$variation")
            assertTrue(DictationFields.isSecret(variation), "$variation")
        }
        assertFalse(DictationFields.isSecret(Variation.NORMAL))
    }

    @Test
    fun `never dictates into a PIN, number, phone or date field`() {
        assertFalse(DictationFields.allows(Type.NUMBER, Variation.PASSWORD))
        assertFalse(DictationFields.allows(Type.NUMBER, Variation.NORMAL))
        assertFalse(DictationFields.allows(Type.PHONE, Variation.NORMAL))
        assertFalse(DictationFields.allows(Type.DATETIME, Variation.DATE))
    }
}
