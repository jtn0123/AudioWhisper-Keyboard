package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.settingsui.ThemeEditorNatives.configure
import com.jtn0123.audiowhisper.settingsui.ThemeEditorNatives.effects
import com.jtn0123.audiowhisper.settingsui.ThemeEditorNatives.font
import com.jtn0123.audiowhisper.settingsui.ThemeEditorNatives.state
import com.materialkolor.Contrast
import com.materialkolor.PaletteStyle
import com.materialkolor.dynamiccolor.ColorSpec
import dev.patrickgold.florisboard.ime.theme.ThemeExtension
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentImpl
import dev.patrickgold.florisboard.lib.ext.Extension
import dev.patrickgold.florisboard.lib.ext.ExtensionJsonConfig
import dev.patrickgold.florisboard.lib.ext.ExtensionMaintainer
import dev.patrickgold.florisboard.lib.ext.ExtensionMeta
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.florisboard.lib.color.MaterialYouFlags
import kotlin.random.Random
import kotlin.test.*

/**
 * The extension editor through the real native settings against the deleted Compose editor ([ReferenceExtensionEditor]).
 * Save against ExtensionEditScreen's `handleSave`, which wrote `ExtensionJsonConfig.encodeToString<Extension>(editor.build())`:
 * for theme extensions made up of awkward text, every optional field present or not and every Material You flag, the
 * `extension.json` Rust hands Kotlin to write is the same, byte for byte, as the one Compose wrote; Save refuses the meta
 * data Compose refused, and a component Compose's `build()` threw on is named instead of crashing. Then each component's
 * box, a copy's id and each folder's picker against ExtensionComponentView, CreateComponentScreen and
 * ExtensionEditFilesScreen. Every name and id is made up.
 */
class SettingsExtensionEditorMatchTest {
    private val random = Random(2026)

    /** Text kotlinx escapes, text it leaves alone, and text `build()` trims. */
    private val words = listOf("Day", "Nüit", "\"quoted\"", "back\\slash", "tab\there", "moon 🌙", "<b>&amp;</b>", "a/b",
        "line\nbreak", "  padded  ", "\u0007bell", "\u007f", "日本語", "", " ")
    private fun word() = words.random(random)
    private fun <T> maybe(value: () -> T): T? = if (random.nextBoolean()) value() else null
    private fun maintainer() = ExtensionMaintainer(name = listOf("Ada", "Grace Hopper", word()).random(random),
        email = maybe { "a@example.org" }, url = maybe { "https://example.org/" + random.nextInt(9) })
    private fun meta() = ExtensionMeta(id = listOf("local.themes.${random.nextInt(1000)}", "my.themes", "a").random(random),
        version = listOf("1.0.0", "0.0.${random.nextInt(9)}", "2.1", "", "v1").random(random), title = word(),
        description = maybe(::word), keywords = maybe { List(random.nextInt(3)) { word() } }, homepage = maybe { "https://example.org" },
        issueTracker = maybe { "https://example.org/issues" }, maintainers = List(random.nextInt(3)) { maintainer() },
        license = listOf("apache-2.0", "mit", "(none specified)", "").random(random))
    private fun flags() = MaterialYouFlags(PaletteStyle.entries.random(random), Contrast.entries.random(random),
        ColorSpec.SpecVersion.entries.random(random))
    private fun component(i: Int) = ThemeExtensionComponentImpl(id = listOf("theme_$i", " theme_$i ", "", "theme_0").random(random),
        label = word(), authors = List(random.nextInt(3)) { word() }, isNightTheme = random.nextBoolean(), materialYouFlags = flags(),
        stylesheetPath = listOf(null, "stylesheets/custom_$i.json", " ").random(random))
    private fun extension() = ThemeExtension(meta(), dependencies = listOf(null, emptyList(), listOf("org.florisboard.themes")).random(random),
        themes = List(random.nextInt(4)) { component(it) })

    private fun MaterialYouFlags.names() = "${paletteStyle.name}/${contrastLevel.name}/${specVersion.name}"
    /** Native settings on the editor of [ext], Kotlin's report of its unpacked manifest in; the state that answers. */
    private fun opened(n: KeyboardRenderer, ext: ThemeExtension): JsonObject {
        configure(n)
        val id = ext.meta.id
        val asked = effects(state(n, SettingsUiOp.NAVIGATE, "ext/edit/$id"))
        assertEquals(listOf(SettingsUiExtEditorEffect.OPEN, id, "", ""), asked.single())
        val head = editorRow(SettingsUiExtEditorKind.WORKSPACE, "ws", id, SettingsUiExtEditorOutcome.READY, label = "1",
            detail = MaterialYouFlags().names(), text = ExtensionJsonConfig.encodeToString<Extension>(ext))
        return state(n, SettingsUiOp.EXT_EDITOR, *head)
    }
    /** The SYNC effect the editor of [ext] answers Kotlin's report with. */
    private fun open(n: KeyboardRenderer, ext: ThemeExtension): List<String> = effects(opened(n, ext)).single()
    private fun label(s: JsonObject, id: Int) =
        s.getValue("nodes").jsonArray.map { it.jsonObject }.singleOrNull { it.getValue("id").jsonPrimitive.content == "$id" }
            ?.getValue("label")?.jsonPrimitive?.content

    @Test fun saveWritesTheManifestComposeWroteByteForByte() {
        val outcomes = IntArray(3)
        repeat(400) {
            val ext = extension()
            val editor = ext.edit()
            val manifest = ExtensionJsonConfig.encodeToString<Extension>(ext)
            KeyboardRenderer(arrayOf(font)).use { n ->
                open(n, ext)
                // Deleting the first component is a change both make the same way.
                if (editor.themes.isNotEmpty() && random.nextBoolean()) {
                    state(n, SettingsUiOp.ACTIVATE, DELETE_FIRST)
                    editor.themes.removeAt(0)
                }
                val effects = effects(state(n, SettingsUiOp.ACTIVATE, SAVE_ID))
                val work = effects.lastOrNull { it[0] == SettingsUiFileEffect.WORK }
                val compose = ReferenceExtensionEditor.handleSave(editor)
                when {
                    compose == null -> assertNull(work, "Compose refused this meta data: $manifest").also { outcomes[0]++ }
                    compose.isFailure -> {
                        outcomes[1]++
                        assertNull(work, "Compose's build() threw: $manifest")
                        assertEquals(listOf(SettingsUiFileEffect.TOAST, INVALID), effects.last { it[0] == SettingsUiFileEffect.TOAST })
                    }
                    else -> {
                        outcomes[2]++
                        assertNotNull(work, manifest)
                        assertEquals(SettingsUiJob.EXT_SAVE, work[SettingsUiWork.JOB])
                        assertEquals(compose.getOrThrow(), work[SettingsUiWork.TEXT], manifest)
                        assertEquals("ws", work[SettingsUiWork.EXTENSION])
                        assertEquals("${ext.meta.id}.flex", work[SettingsUiWork.NAME])
                        assertEquals(ext.meta.id, work[SettingsUiWork.TARGET])
                    }
                }
            }
        }
        assertTrue(outcomes.all { it > 40 }, "refused, thrown, written: ${outcomes.toList()}")
    }

    /** The copy Kotlin keeps for the theme editor is the extension as kotlinx wrote it, nothing trimmed; only an empty
     * dependency list, which `build()` drops too, is left out. */
    @Test fun theDraftKotlinKeepsIsTheExtensionUntrimmed() {
        repeat(200) {
            val ext = extension()
            KeyboardRenderer(arrayOf(font)).use { n ->
                val sync = open(n, ext)
                assertEquals(listOf(SettingsUiExtEditorEffect.SYNC, "ws"), sync.take(2))
                val kept = ThemeExtension(ext.meta, ext.dependencies?.takeUnless { it.isEmpty() }, ext.themes)
                assertEquals(ExtensionJsonConfig.encodeToString<Extension>(kept), sync[2])
            }
        }
    }

    /** Each component's box says what ExtensionComponentView said of the component ExtensionEditScreen held: its label
     * over `<extension>:<component>`, then its authors, whether it is a night theme and its stylesheet, untrimmed. */
    @Test fun eachComponentBoxSaysWhatExtensionComponentViewSaid() {
        repeat(150) {
            val ext = extension()
            val editor = ext.edit()
            KeyboardRenderer(arrayOf(font)).use { n ->
                val s = opened(n, ext)
                for ((c, component) in editor.themes.withIndex()) {
                    val box = COMP_ID + 8 * c
                    assertEquals(component.label, label(s, box), "box $c")
                    assertEquals("${ext.meta.id}:${component.id}", label(s, box + 1), "box $c")
                    assertEquals(ReferenceExtensionScreens.componentText(component), label(s, box + 2), "box $c")
                }
                assertNull(label(s, COMP_ID + 8 * editor.themes.size), "no box past the last component")
            }
        }
    }

    /** Create component's copy of one of the extension's own themes takes the id CreateComponentScreen gave it, among
     * ids that already carry its suffixes. */
    @Test fun aCopyOfItsOwnThemeTakesTheIdComposeGaveIt() {
        val pool = listOf("night", "night_1", "night_2", "day", "day_1", "night_1_1")
        repeat(150) {
            val ids = pool.shuffled(random).take(1 + random.nextInt(4))
            val ext = ThemeExtension(meta().copy(id = "local.themes.1", version = "1.0.0", title = "Mine", license = "mit",
                maintainers = listOf(ExtensionMaintainer("Ada"))), themes = ids.map { id -> component(0).copy(id = id) })
            val i = random.nextInt(ids.size)
            val expected = ReferenceExtensionEditor.copyId(ext.edit().themes, ids[i])
            KeyboardRenderer(arrayOf(font)).use { n ->
                open(n, ext)
                state(n, SettingsUiOp.ACTIVATE, "$ADD_ID")
                state(n, SettingsUiOp.ACTIVATE, "${SOURCE_ID + i}")
                val work = assertNotNull(effects(state(n, SettingsUiOp.ACTIVATE, CREATE_ID)).lastOrNull { it[0] == SettingsUiFileEffect.WORK })
                assertEquals(SettingsUiJob.EXT_COMPONENT_COPY, work[SettingsUiWork.JOB])
                val done = Array(SettingsUiDone.SIZE) { "" }.also {
                    it[SettingsUiDone.REPORT] = SettingsUiFiles.DONE
                    it[SettingsUiDone.JOB] = SettingsUiJob.EXT_COMPONENT_COPY
                    it[SettingsUiDone.RESULT] = SettingsUiJobResult.OK
                    it[SettingsUiDone.NAME] = work[SettingsUiWork.NAME]
                }
                val sync = effects(state(n, SettingsUiOp.FILES, *done)).last { it[0] == SettingsUiExtEditorEffect.SYNC }
                val copied = decodeDraft(sync[2]).themes
                assertEquals(ids + expected, copied.map { it.id }, "copy of ${ids[i]} among $ids")
            }
        }
    }

    /** Each folder's Add asks the picker for what ExtensionEditFilesScreen's filter took. */
    @Test fun eachFoldersPickerTakesWhatComposesFilterTook() {
        for ((f, folder) in listOf(SettingsUiExtEditorFolder.FONTS, SettingsUiExtEditorFolder.IMAGES).withIndex()) {
            KeyboardRenderer(arrayOf(font)).use { n ->
                open(n, extension())
                state(n, SettingsUiOp.ACTIVATE, "$FILES_ID")
                val pick = effects(state(n, SettingsUiOp.ACTIVATE, "${FOLDER_ID + 1000 * f + 1}")).single()
                assertEquals(listOf("pick", "open", ReferenceExtensionEditor.MIME_TYPES.getValue(folder).types.joinToString("|"), ""), pick)
            }
        }
    }

    private companion object {
        /** The editor's Save, the bottom bar's row 4. */
        const val SAVE_ID = "1004"
        /** Create component's Create, its bottom bar's row 7. */
        const val CREATE_ID = "1007"
        /** Component `c`'s box is `COMP_ID + 8 * c`, then 1 its name, 2 its text, 3 Delete and 4 Edit (`paint.rs`). */
        const val COMP_ID = 410_000
        /** The first component's Delete. */
        const val DELETE_FIRST = "${COMP_ID + 3}"
        /** The bundled themes' Add, Create component's sources and the files sheet's folders (`paint.rs`, `sheets.rs`). */
        const val ADD_ID = 400_021
        const val SOURCE_ID = 431_000
        const val FILES_ID = 400_012
        const val FOLDER_ID = 421_000
        /** The toast an unbuildable component gets, its key with no strings given. */
        const val INVALID = "settings__ext_editor__component_invalid"
    }
}
