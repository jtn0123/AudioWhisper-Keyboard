package dev.patrickgold.florisboard.ime.dictionary

import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.lib.ValidationRule
import dev.patrickgold.florisboard.lib.FlorisLocale
import org.florisboard.lib.kotlin.UtilityRules
import org.florisboard.lib.kotlin.tryOrNull

object UserDictionaryValidation {
    private fun rule(name: String, input: String) = UtilityRules.call(14, name, input, "$FREQUENCY_MIN", "$FREQUENCY_MAX")[0]
    private val WordRegex = """^[^\s;,]+${'$'}""".toRegex()

    val Word = ValidationRule<String> {
        forKlass = UserDictionaryEntry::class
        forProperty = "word"
        validator { input ->
            val status = rule("word", input)
            when {
                status == "empty" -> resultInvalid(error = R.string.settings__udm__dialog__word_error_empty)
                status == "invalid" -> resultInvalid(error = R.string.settings__udm__dialog__word_error_invalid, "regex" to WordRegex)
                else -> resultValid()
            }
        }
    }

    val Freq = ValidationRule<String> {
        forKlass = UserDictionaryEntry::class
        forProperty = "freq"
        validator { input ->
            val status = rule("freq", input)
            when {
                status == "empty" -> resultInvalid(error = R.string.settings__udm__dialog__freq_error_empty)
                status == "invalid" -> resultInvalid(error = R.string.settings__udm__dialog__freq_error_invalid)
                else -> resultValid()
            }
        }
    }

    val Shortcut = ValidationRule<String> {
        forKlass = UserDictionaryEntry::class
        forProperty = "shortcut"
        validator { input ->
            val status = rule("shortcut", input)
            when {
                status == "" -> resultValid() // Is optional
                status == "invalid" -> resultInvalid(error = R.string.settings__udm__dialog__shortcut_error_invalid, "regex" to WordRegex)
                else -> resultValid()
            }
        }
    }

    val Locale = ValidationRule<String> {
        forKlass = UserDictionaryEntry::class
        forProperty = "locale"
        validator { input ->
            when {
                input.isBlank() -> resultValid() // Is optional
                tryOrNull { FlorisLocale.fromTag(input.trim()) } == null -> resultInvalid(error = R.string.settings__udm__dialog__locale_error_invalid)
                else -> resultValid()
            }
        }
    }
}
