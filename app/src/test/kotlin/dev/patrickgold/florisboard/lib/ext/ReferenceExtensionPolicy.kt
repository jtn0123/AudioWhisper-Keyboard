package dev.patrickgold.florisboard.lib.ext

import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentImpl
import org.florisboard.lib.color.MaterialYouFlags
import dev.patrickgold.florisboard.ime.keyboard.KeyboardExtension
import dev.patrickgold.florisboard.ime.nlp.LanguagePackExtension
import dev.patrickgold.florisboard.ime.theme.ThemeExtension
import dev.patrickgold.florisboard.ime.text.composing.Appender
import dev.patrickgold.florisboard.ime.text.composing.Composer
import dev.patrickgold.florisboard.ime.text.composing.WithRules
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.polymorphic

/** Policy copied from 6acde09c's component/editor, manager and import screen. */
@OptIn(ExperimentalSerializationApi::class)
object ReferenceExtensionPolicy {
    private val json = Json {
        classDiscriminator = "$"
        encodeDefaults = false
        ignoreUnknownKeys = true
        isLenient = true
        prettyPrint = true
        prettyPrintIndent = "  "
        serializersModule = SerializersModule {
            polymorphic(Extension::class) {
                subclass(KeyboardExtension::class, KeyboardExtension.serializer())
                subclass(ThemeExtension::class, ThemeExtension.serializer())
                subclass(LanguagePackExtension::class, LanguagePackExtension.serializer())
            }
            polymorphic(Composer::class) {
                subclass(Appender::class, Appender.serializer())
                subclass(WithRules::class, WithRules.serializer())
                defaultDeserializer { Appender.serializer() }
            }
        }
    }
    fun componentName(str: String): ExtensionComponentName {
        val data = str.split(":")
        check(data.size == 2) { "Extension component name must be of format <ext_id>:<comp_id>" }
        return ExtensionComponentName(data[0], data[1])
    }

    fun themeComponent(id: String, label: String, authors: List<String>, stylesheetPath: String): ThemeExtensionComponentImpl {
        val component = ThemeExtensionComponentImpl(
            id = id.trim(), label = label.trim(), authors = authors.filterNot { it.isBlank() },
            isNightTheme = true, materialYouFlags = MaterialYouFlags(), stylesheetPath = stylesheetPath.takeUnless { it.isBlank() },
        )
        check(id.isNotBlank()) { "Theme component ID cannot be blank" }
        check(label.isNotBlank()) { "Theme component label cannot be blank" }
        check(authors.isNotEmpty()) { "Theme component authors must contain at least one non-blank author field" }
        return component
    }

    fun stylesheet(id: String, path: String?): String = path.takeUnless { it.isNullOrBlank() } ?: "stylesheets/$id.json"

    fun decode(text: String): Extension = json.decodeFromString(Extension.serializer(), text)

    fun group(serial: String): String = when (serial) {
        "ime.extension.keyboard" -> "ime/keyboard"
        "ime.extension.theme" -> "ime/theme"
        "ime.extension.languagepack" -> "ime/languagepack"
        else -> error("Unknown extension type")
    }

    fun accepts(requested: String, serial: String): Boolean = when (requested) {
        "ext-any" -> serial in listOf("ime.extension.keyboard", "ime.extension.theme", "ime.extension.languagepack")
        "ext-keyboard" -> serial == "ime.extension.keyboard"
        "ext-theme" -> serial == "ime.extension.theme"
        "ext-languagepack" -> serial == "ime.extension.languagepack"
        else -> false
    }

    fun skip(supported: Boolean, parsed: Boolean, core: Boolean) = when {
        !supported -> "unsupported"
        parsed -> if (core) "core" else ""
        else -> "corrupted"
    }
}
