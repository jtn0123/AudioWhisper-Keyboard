package dev.patrickgold.florisboard.ime.core

import com.jtn0123.audiowhisper.rust.ImeSubtypeRow
import com.jtn0123.audiowhisper.settings.SettingsCodecs
import dev.patrickgold.florisboard.ime.keyboard.CurrencySet
import dev.patrickgold.florisboard.ime.keyboard.LayoutType
import dev.patrickgold.florisboard.ime.keyboard.extCoreComposer
import dev.patrickgold.florisboard.ime.keyboard.extCoreCurrencySet
import dev.patrickgold.florisboard.ime.keyboard.extCoreLayout
import dev.patrickgold.florisboard.ime.keyboard.extCorePopupMapping
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFails
import kotlin.test.assertNull
import kotlin.test.assertSame

class SubtypeSessionTest {
    private val saved = Subtype.DEFAULT.copy(id = 42)

    private fun load(vararg list: Subtype, session: SubtypeSession = SubtypeSession()) =
        session.load(SettingsCodecs.subtypes(list.toList()))

    @Test
    fun `removed layouts composers popups and Han providers restore a working default with the saved id`() {
        val old = listOf(
            saved.copy(layoutMap = saved.layoutMap.copy(characters = extCoreLayout("jcuken_russian"))),
            saved.copy(composer = extCoreComposer("hangul-unicode")),
            saved.copy(composer = extCoreComposer("kana-unicode")),
            saved.copy(popupMapping = extCorePopupMapping("cjk")),
            saved.copy(nlpProviders = SubtypeNlpProviderMap(suggestion = HAN)),
            saved.copy(nlpProviders = SubtypeNlpProviderMap(spelling = HAN)),
        )
        for (subtype in old) {
            val loaded = load(subtype)
            assertEquals(listOf(saved), loaded.list.map { it.subtype })
            assertEquals(SettingsCodecs.subtypes(listOf(saved)), loaded.repaired)
            assertNull(load(loaded.list.single().subtype).repaired, "restoration is idempotent")
        }
    }

    @Test
    fun `saved JSON is restored before the subtype is used`() {
        val savedJson = SubtypeJsonConfig.encodeToString(listOf(
            saved.copy(layoutMap = saved.layoutMap.copy(characters = extCoreLayout("arabic"))),
        ))
        assertEquals(listOf(saved), SubtypeSession().load(savedJson).list.map { it.subtype })
    }

    @Test
    fun `Latin layouts Vietnamese rules and add-ons keep their configuration`() {
        val latin = saved.copy(
            composer = extCoreComposer("telex"),
            layoutMap = saved.layoutMap.copy(characters = extCoreLayout("turkish_q")),
            popupMapping = extCorePopupMapping("vi-VN"),
        )
        val addon = saved.copy(
            composer = ExtensionComponentName("my.addon", "kana-unicode"),
            layoutMap = saved.layoutMap.copy(characters = ExtensionComponentName("my.addon", "arabic")),
            popupMapping = ExtensionComponentName("my.addon", "cjk"),
        )
        val loaded = load(latin, addon)
        assertEquals(listOf(latin, addon), loaded.list.map { it.subtype })
        assertNull(loaded.repaired)
    }

    @Test
    fun `a row names each layout type in its order`() {
        val fields = mapOf(
            LayoutType.CHARACTERS to ImeSubtypeRow.CHARACTERS, LayoutType.CHARACTERS_MOD to ImeSubtypeRow.CHARACTERS_MOD,
            LayoutType.EXTENSION to ImeSubtypeRow.EXTENSION, LayoutType.NUMERIC to ImeSubtypeRow.NUMERIC,
            LayoutType.NUMERIC_ADVANCED to ImeSubtypeRow.NUMERIC_ADVANCED, LayoutType.NUMERIC_ROW to ImeSubtypeRow.NUMERIC_ROW,
            LayoutType.PHONE to ImeSubtypeRow.PHONE, LayoutType.PHONE2 to ImeSubtypeRow.PHONE2,
            LayoutType.SYMBOLS to ImeSubtypeRow.SYMBOLS, LayoutType.SYMBOLS_MOD to ImeSubtypeRow.SYMBOLS_MOD,
            LayoutType.SYMBOLS2 to ImeSubtypeRow.SYMBOLS2, LayoutType.SYMBOLS2_MOD to ImeSubtypeRow.SYMBOLS2_MOD,
        )
        assertEquals(LayoutType.entries.toSet(), fields.keys)
        for ((type, field) in fields) assertEquals(field, ImeSubtypeRow.CHARACTERS + type.ordinal, "$type")
    }

    @Test
    fun `facts name each layout and the popup mapping and equal subtypes share a key`() {
        val other = saved.copy(id = 43, layoutMap = saved.layoutMap.copy(characters = extCoreLayout("azerty")))
        val entries = load(saved, other, saved).list
        assertEquals(listOf(0, 1, 0), entries.map { it.facts.key })
        for (type in LayoutType.entries) assertEquals(other.layoutMap[type], entries[1].facts.layout(type), "$type")
        assertEquals(other.popupMapping, entries[1].facts.popupMapping)
    }

    @Test
    fun `each locale tag is named once as Android names it`() {
        val asked = mutableListOf<String>()
        val session = SubtypeSession({ asked += it; FlorisLocale.fromTag(it).languageTag() })
        val list = SettingsCodecs.subtypes(listOf(saved)).replace("\"en-US\"", "\"en_US\"")
        val first = session.load(list)
        assertEquals("en-US", first.list.single().subtype.primaryLocale.languageTag())
        assertNull(first.repaired, "a tag named differently is not a repair")
        session.load(list)
        assertEquals(listOf("en_US", "en-US"), asked)
    }

    @Test
    fun `the active subtype follows the stored id the language key and a switch by id`() {
        val session = SubtypeSession()
        val other = saved.copy(id = 43, primaryLocale = FlorisLocale.fromTag("fr-FR"))
        assertEquals(Subtype.DEFAULT, session.active().subtype)
        load(saved, other, session = session)
        session.evaluate(43).let { assertEquals(other, it.active.subtype); assertEquals(false, it.write) }
        session.evaluate(7).let { assertEquals(saved, it.active.subtype); assertEquals(true, it.write) }
        assertEquals(other, session.next().active.subtype)
        assertEquals(saved, session.next().active.subtype)
        assertEquals(other, session.previous().active.subtype)
        assertNull(session.select(7))
        assertEquals(saved, session.select(42)?.active?.subtype)
        assertEquals(saved, session.active().subtype)
    }

    @Test
    fun `a currency set is the loaded one its name names`() {
        val session = SubtypeSession()
        val euro = CurrencySet("euro", "Euro", emptyList())
        val dollar = CurrencySet("dollar", "Dollar", emptyList())
        val sets = mapOf(extCoreCurrencySet("dollar") to dollar, extCoreCurrencySet("euro") to euro)
        assertSame(euro, session.currencySet(sets, extCoreCurrencySet("euro")))
        assertNull(session.currencySet(sets, extCoreCurrencySet("yen")))
        assertNull(session.currencySet(null, extCoreCurrencySet("euro")))
        assertSame(dollar, session.currencySet(mapOf(extCoreCurrencySet("dollar") to dollar), extCoreCurrencySet("dollar")))
    }

    @Test
    fun `a blank list is empty and a broken one throws`() {
        assertEquals(emptyList(), SubtypeSession().load("  ").list)
        assertFails { SubtypeSession().load("[{") }
    }

    private companion object {
        const val HAN = "org.florisboard.nlp.providers.han.shape"
    }
}
