package dev.patrickgold.florisboard.lib.ext

import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.ime.keyboard.KeyboardExtension
import dev.patrickgold.florisboard.ime.nlp.LanguagePackExtension
import dev.patrickgold.florisboard.ime.theme.ThemeExtension
import kotlinx.serialization.SerializationException

/** JSON syntax/typed UI model conversion. Rust selects the recognized extension type. */
object ExtensionCodec {
    fun decode(text: String): Extension {
        val syntax = try {
            ExtensionJsonConfig.parseToJsonElement(text)
        } catch (_: SerializationException) {
            return ExtensionJsonConfig.decodeFromString(Extension.serializer(), text)
        }
        val serializer = when (ExtensionPolicy.manifest(text, syntax.toString())[ExtensionManifest.SERIAL]) {
            "ime.extension.keyboard" -> KeyboardExtension.serializer()
            "ime.extension.theme" -> ThemeExtension.serializer()
            "ime.extension.languagepack" -> LanguagePackExtension.serializer()
            // Let the same syntax/serialization boundary materialize its exact diagnostic,
            // including the original text/offset, for a Rust-rejected or malformed type.
            else -> return ExtensionJsonConfig.decodeFromString(Extension.serializer(), text)
        }
        return try {
            ExtensionJsonConfig.decodeFromString(serializer, text)
        } catch (_: SerializationException) {
            ExtensionJsonConfig.decodeFromString(Extension.serializer(), text)
        }
    }
}
