package dev.patrickgold.florisboard.ime.core

import dev.patrickgold.florisboard.ime.keyboard.extCoreComposer
import dev.patrickgold.florisboard.ime.keyboard.extCoreLayout
import dev.patrickgold.florisboard.ime.keyboard.extCorePopupMapping
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertSame

class SubtypeFallbackTest {
    private val saved = Subtype.DEFAULT.copy(id = 42)

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
            val restored = subtype.withSupportedInput()
            assertEquals(saved, restored)
            assertSame(restored, restored.withSupportedInput(), "restoration is idempotent")
        }
    }

    @Test
    fun `saved JSON is restored before the subtype is used`() {
        val savedJson = SubtypeJsonConfig.encodeToString(listOf(
            saved.copy(layoutMap = saved.layoutMap.copy(characters = extCoreLayout("arabic"))),
        ))
        val restored = SubtypeJsonConfig.decodeFromString<List<Subtype>>(savedJson).map { it.withSupportedInput() }
        assertEquals(listOf(saved), restored)
    }

    @Test
    fun `Latin layouts Vietnamese rules and add-ons keep their configuration`() {
        val latin = saved.copy(
            composer = extCoreComposer("telex"),
            layoutMap = saved.layoutMap.copy(characters = extCoreLayout("turkish_q")),
            popupMapping = extCorePopupMapping("vi-VN"),
        )
        assertSame(latin, latin.withSupportedInput())
        val addon = saved.copy(
            composer = ExtensionComponentName("my.addon", "kana-unicode"),
            layoutMap = saved.layoutMap.copy(characters = ExtensionComponentName("my.addon", "arabic")),
            popupMapping = ExtensionComponentName("my.addon", "cjk"),
        )
        assertSame(addon, addon.withSupportedInput())
    }

    private companion object {
        const val HAN = "org.florisboard.nlp.providers.han.shape"
    }
}
