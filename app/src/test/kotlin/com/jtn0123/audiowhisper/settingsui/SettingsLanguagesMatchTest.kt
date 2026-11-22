package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.settings.SettingsCodecs
import dev.patrickgold.florisboard.app.ReferenceSettingsCodecs
import dev.patrickgold.florisboard.app.ReferenceSubtypeEditorState
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.core.SubtypeNlpProviderMap
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlinx.serialization.json.*
import java.io.File
import kotlin.random.Random
import kotlin.test.*

/** The language routes the native settings draw (stage4-languages.md), in the native list's order. */
internal val languageRoutes = listOf("settings/localization", "settings/localization/select-locale",
    "settings/localization/language-pack-manage", "settings/localization/subtype/add", "settings/localization/subtype/edit")

/**
 * Languages & Layouts through the real native settings. A list stored as SubtypeManager stored it is moved, deleted,
 * edited and added to, and each write is, byte for byte, what the removed SubtypeManager functions wrote for the same
 * edit through the original kotlinx codec. Then the editor's checks against the frozen SubtypeEditorState's.
 */
class SettingsLanguagesMatchTest {
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private val subtypesKey = "localization__subtypes"
    private val activeKey = "localization__active_subtype_id"
    private val strings: Map<String, String> by lazy {
        val pattern = Regex("""<string name="([^"]+)"[^>]*>(.*?)</string>""", RegexOption.DOT_MATCHES_ALL)
        listOf("strings.xml", "strings_settings_renderer.xml").flatMap { name ->
            pattern.findAll(File("src/main/res/values/$name").readText()).map { it.groupValues[1] to it.groupValues[2].replace("\\'", "'") }
        }.toMap()
    }
    private fun item(slot: Int, part: Int) = (8000 + 4 * slot + part).toString()
    private fun state(n: KeyboardRenderer, op: SettingsUiOp, vararg input: String) =
        Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    /** The one element [pick] finds, failing as an assertion, so a mutation that loses it counts as caught. */
    private fun <T> List<T>.one(what: String, pick: (T) -> Boolean = { true }) =
        filter(pick).also { assertEquals(1, it.size, "one $what") }[0]
    private fun node(s: JsonObject, label: String) = nodes(s).one(label) { it.text("label") == label }
    /** A dialog's choice by its label. */
    private fun choice(s: JsonObject, label: String) =
        nodes(s).one("choice $label") { it.text("id").toInt() in 3000..3999 && it.text("label") == label }.text("id")
    private fun writes(s: JsonObject) = effects(s).filter { it[0] == "write" }
    private fun written(s: JsonObject) = writes(s).one("write").also { assertEquals(subtypesKey, it[1]) }[2]
    private fun component(group: String, name: String) = ExtensionComponentName("org.florisboard.$group", name)
    private fun subtype(id: Long, tag: String, popup: String, characters: String, currency: String) = Subtype.DEFAULT.copy(
        id = id, primaryLocale = FlorisLocale.fromTag(tag), popupMapping = component("localization", popup),
        currencySet = component("currencysets", currency),
        layoutMap = Subtype.DEFAULT.layoutMap.copy(characters = component("layouts", characters)))
    private val english = subtype(1_700_000_000_000, "en-US", "en", "qwerty", "dollar")
    private val german = subtype(1_700_000_000_001, "de-DE", "de", "qwertz", "euro")
    private val french = subtype(1_700_000_000_002, "fr-FR", "fr", "azerty", "euro")
    private val spanish = subtype(Subtype.DEFAULT.id, "es-ES", "es", "qwerty", "euro")

    /** What SettingsLanguages reports for [raw], built with its own row functions. */
    private fun report(raw: String): Array<String> {
        val rows = mutableListOf<String>()
        rows += subtypesRow(raw, loadedSubtypes(raw, english.id))
        val locales = listOf("en-US", "de-DE", "fr-FR", "es-ES").map(FlorisLocale::fromTag)
        for (locale in locales) {
            rows += languageRow(SettingsUiLanguageKind.LOCALE, locale)
            rows += languageRow(SettingsUiLanguageKind.NAME, locale)
        }
        rows += languageRow(SettingsUiLanguageKind.SYSTEM, locales[0])
        val layouts = listOf("qwerty" to "QWERTY", "qwertz" to "QWERTZ", "azerty" to "AZERTY")
        for ((id, label) in layouts) rows += componentRow(SettingsUiLanguageKind.LAYOUT, "org.florisboard.layouts:$id", label, "characters")
        for ((type, id) in listOf("symbols" to "western", "symbols2" to "western", "numeric" to "western_arabic",
            "numericAdvanced" to "western_arabic", "numericRow" to "western_arabic", "phone" to "telpad", "phone2" to "telpad")) {
            rows += componentRow(SettingsUiLanguageKind.LAYOUT, "org.florisboard.layouts:$id", id, type)
        }
        rows += componentRow(SettingsUiLanguageKind.COMPOSER, "org.florisboard.composers:appender", "Appender")
        for (id in listOf("dollar", "euro")) rows += componentRow(SettingsUiLanguageKind.CURRENCY, "org.florisboard.currencysets:$id", id)
        for (id in listOf("default", "en", "de", "fr", "es")) rows += componentRow(SettingsUiLanguageKind.POPUP, "org.florisboard.localization:$id", id)
        for (s in listOf(english, german, french, spanish)) rows += presetRow(s.primaryLocale, s.copy(id = Subtype.DEFAULT.id))
        return rows.toTypedArray()
    }
    private fun prepare(n: KeyboardRenderer, raw: String) {
        n.settings(SettingsUiOp.RESOURCES, n.settings(SettingsUiOp.STRING_KEYS)[2].split("\n").filter { it in strings }
            .flatMap { listOf(it, strings.getValue(it)) }.toTypedArray())
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(SettingsUiOp.CONFIG, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true")).toTypedArray())
        n.settings(SettingsUiOp.PREFERENCES, arrayOf(subtypesKey, raw, activeKey, english.id.toString()))
        assertEquals(listOf(listOf(SettingsUiLanguageEffect.LOAD)), effects(state(n, SettingsUiOp.NAVIGATE, languageRoutes[0])))
        state(n, SettingsUiOp.LANGUAGES, *report(raw))
    }

    /** The stored format, frozen: the default subtype as SubtypeManager has always written it. */
    @Test fun theStoredFormatIsTheOriginalKotlinxOutput() {
        val fixture = """[{"id":1700000000000,"primaryLocale":"en-US","secondaryLocales":[],"nlpProviders":{"spelling":""" +
            """"org.florisboard.nlp.providers.latin","suggestion":"org.florisboard.nlp.providers.latin"},"composer":""" +
            """"org.florisboard.composers:appender","currencySet":"org.florisboard.currencysets:dollar","punctuationRule":""" +
            """"org.florisboard.localization:default","popupMapping":"org.florisboard.localization:en","layoutMap":{"characters":""" +
            """"org.florisboard.layouts:qwerty","symbols":"org.florisboard.layouts:western","symbols2":"org.florisboard.layouts:western",""" +
            """"numeric":"org.florisboard.layouts:western_arabic","numericAdvanced":"org.florisboard.layouts:western_arabic",""" +
            """"numericRow":"org.florisboard.layouts:western_arabic","phone":"org.florisboard.layouts:telpad","phone2":""" +
            """"org.florisboard.layouts:telpad"}}]"""
        assertEquals(fixture, ReferenceSettingsCodecs.subtypes(listOf(english)))
        assertEquals(fixture, SettingsCodecs.subtypes(listOf(english)))
        // The report's rows: a locale's three names and the tags a preset is matched by.
        val row = languageRow(SettingsUiLanguageKind.LOCALE, FlorisLocale.fromTag("de-DE"))
        assertEquals(listOf("de-DE", "Deutsch (Deutschland)", "German (Germany)", "de_DE", "de"),
            listOf(SettingsUiLanguage.ID, SettingsUiLanguage.NATIVE, SettingsUiLanguage.ENGLISH, SettingsUiLanguage.BASE,
                SettingsUiLanguage.LANGUAGE).map { row[it] })
        assertEquals(german.copy(id = -1), SettingsCodecs.subtype(presetRow(german.primaryLocale, german.copy(id = -1))[SettingsUiLanguage.DETAIL]))
    }

    @Test fun movesDeletesEditsAndAddsWriteWhatSubtypeManagerWrote() {
        var list = listOf(english, german, french)
        KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, ReferenceSettingsCodecs.subtypes(list))
            // reorderSubtypes([1, 0, 2]). Slots follow the subtype, so German stays slot 1.
            list = listOf(german, english, french)
            assertEquals(ReferenceSettingsCodecs.subtypes(list), written(state(n, SettingsUiOp.ACTIVATE, item(1, 1))))
            // removeSubtype(english), then Undo puts it back where it stood.
            assertEquals(ReferenceSettingsCodecs.subtypes(list - english), written(state(n, SettingsUiOp.ACTIVATE, item(0, 3))))
            assertEquals(ReferenceSettingsCodecs.subtypes(list), written(state(n, SettingsUiOp.ACTIVATE, "7901")))
            // modifySubtypeWithSameId: French's characters layout set to QWERTY in its editor.
            var s = state(n, SettingsUiOp.ACTIVATE, item(2, 0))
            assertEquals("${languageRoutes[4]}/${french.id}", s.text("route"))
            s = state(n, SettingsUiOp.ACTIVATE, "1003")
            state(n, SettingsUiOp.ACTIVATE, choice(s, "QWERTY"))
            val qwerty = french.copy(layoutMap = french.layoutMap.copy(characters = component("layouts", "qwerty")))
            list = list.map { if (it.id == french.id) qwerty else it }
            s = state(n, SettingsUiOp.ACTIVATE, "1021")
            assertEquals(ReferenceSettingsCodecs.subtypes(list), written(s))
            assertEquals(languageRoutes[0], s.text("route"))
            // addSubtype: a preset already in the list is refused with Compose's message, and writes nothing.
            s = state(n, SettingsUiOp.ACTIVATE, "7900")
            assertEquals(languageRoutes[3], s.text("route"))
            state(n, SettingsUiOp.ACTIVATE, choice(s, german.primaryLocale.displayName()))
            s = state(n, SettingsUiOp.ACTIVATE, "1021")
            assertTrue(writes(s).isEmpty())
            node(s, strings.getValue("settings__localization__subtype_error_already_exists"))
            // A new one is added at the end, stamped with the time.
            state(n, SettingsUiOp.ACTIVATE, "7")
            s = state(n, SettingsUiOp.ACTIVATE, "7909")
            val before = System.currentTimeMillis()
            state(n, SettingsUiOp.ACTIVATE, choice(s, spanish.primaryLocale.displayName()))
            val raw = written(state(n, SettingsUiOp.ACTIVATE, "1021"))
            val added = ReferenceSettingsCodecs.subtypes(raw).last()
            assertTrue(added.id in before..System.currentTimeMillis())
            assertEquals(ReferenceSettingsCodecs.subtypes(list + spanish.copy(id = added.id)), raw)
        }
    }

    /** Each item offers Move up, Move down and Delete to TalkBack, by the ids of the buttons they press. */
    @Test fun anItemsCustomActionsAreItsButtons() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, ReferenceSettingsCodecs.subtypes(listOf(english, german)))
            val s = state(n, SettingsUiOp.SAVE)
            fun custom(slot: Int) = nodes(s).one("item $slot") { it.text("id") == item(slot, 0) }.getValue("custom").jsonArray
                .map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
            val (up, down, delete) = listOf("settings__list__move_up", "settings__list__move_down", "action__delete").map(strings::getValue)
            assertEquals(listOf(listOf(item(0, 2), down), listOf(item(0, 3), delete)), custom(0))
            assertEquals(listOf(listOf(item(1, 1), up), listOf(item(1, 3), delete)), custom(1))
        }
    }

    /** SettingsDerivationMatchTest compared SubtypeEditorState with the frozen copy; the Rust editor's Save now must. */
    @Test fun theEditorAcceptsAndRefusesWhatSubtypeEditorStateDid() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, "[]")
            state(n, SettingsUiOp.NAVIGATE, languageRoutes[3])
            state(n, SettingsUiOp.ACTIVATE, "6")
            val saved = n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
            val random = Random(1712)
            val values = listOf("valid", "00:00", "00-00", "", "00", "synthetic:a", "😀")
            var accepted = 0
            repeat(5000) {
                val old = ReferenceSubtypeEditorState(Subtype.DEFAULT)
                old.primaryLocale.value = FlorisLocale.fromTag(listOf("en-US", "fr-FR", "00-00", "00", "zz").random(random))
                old.nlpProviders.value = SubtypeNlpProviderMap(values.random(random), values.random(random))
                val component = ExtensionComponentName("00", if (random.nextBoolean()) "00" else "valid")
                when (random.nextInt(7)) {
                    0 -> old.composer.value = component
                    1 -> old.currencySet.value = component
                    2 -> old.punctuationRule.value = component
                    3 -> old.popupMapping.value = component
                    4 -> old.layoutMap.value = old.layoutMap.value.copy(characters = component)
                    5 -> old.layoutMap.value = old.layoutMap.value.copy(phone2 = component)
                }
                val draft = Subtype(old.id.value, old.primaryLocale.value, old.secondaryLocales.value, old.nlpProviders.value,
                    old.composer.value, old.currencySet.value, old.punctuationRule.value, old.popupMapping.value, old.layoutMap.value)
                saved[16] = buildJsonObject {
                    put("draft", buildJsonArray { add(SettingsCodecs.subtype(draft)); add(true); add(-1); add(false) })
                    put("search", ""); put("deleted", JsonNull); put("slots", buildJsonArray {})
                }.toString()
                n.settings(SettingsUiOp.PREFERENCES, arrayOf(subtypesKey, "[]"))
                assertEquals(languageRoutes[3], state(n, SettingsUiOp.RESTORE, *saved).text("route"))
                val s = state(n, SettingsUiOp.ACTIVATE, "1021")
                val expected = old.toSubtype()
                assertEquals(expected.isSuccess, writes(s).isNotEmpty(), "draft $it")
                if (expected.isSuccess) {
                    accepted++
                    val added = ReferenceSettingsCodecs.subtypes(written(s)).one("added subtype")
                    assertEquals(expected.getOrThrow().copy(id = added.id), added)
                } else {
                    node(s, strings.getValue("settings__localization__subtype_error_fields_no_value"))
                }
            }
            assertTrue(accepted in 1..4999, "both outcomes are checked: $accepted accepted")
        }
    }
}
