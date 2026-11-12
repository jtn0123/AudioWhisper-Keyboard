package com.jtn0123.audiowhisper.dictation

import dev.patrickgold.florisboard.ime.editor.InputAttributes
import dev.patrickgold.florisboard.ime.editor.InputAttributes.Type
import dev.patrickgold.florisboard.ime.editor.InputAttributes.Variation

/**
 * The fields dictation may type into: text, and raw-key fields (type NULL, such as terminals).
 * Never a password or PIN, which would mean saying it aloud, and not number, phone or date fields,
 * where words don't belong.
 */
object DictationFields {
    private val SECRET = setOf(Variation.PASSWORD, Variation.VISIBLE_PASSWORD, Variation.WEB_PASSWORD)

    fun allows(type: Type, variation: Variation): Boolean =
        (type == Type.TEXT || type == Type.NULL) && !isSecret(variation)

    /** A password or PIN field, whose text must not leave the keyboard. */
    fun isSecret(variation: Variation): Boolean = variation in SECRET

    fun allows(field: InputAttributes): Boolean = allows(field.type, field.variation)
}
