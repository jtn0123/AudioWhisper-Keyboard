package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.*
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.ui.graphics.Color
import com.jtn0123.audiowhisper.settingsui.ThemeEditorNatives.removeProperty
import com.materialkolor.Contrast
import com.materialkolor.PaletteStyle
import com.materialkolor.dynamiccolor.ColorSpec
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentEditor
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionEditor
import dev.patrickgold.florisboard.lib.ext.ExtensionMaintainer
import dev.patrickgold.florisboard.lib.ext.ExtensionMeta
import kotlinx.serialization.json.*
import org.florisboard.lib.color.MaterialYouFlags
import java.io.File
import kotlin.io.path.createTempDirectory
import kotlin.test.*

/**
 * SettingsThemeEditor's report and save against the real natives, on an extension unpacked in a temporary directory as
 * the workspace's `extDir` holds it; a component the workspace no longer has; and the draft across a saved state.
 */
class SettingsThemeEditorGlueMatchTest {
    private val accent = Color(0xFF00897B)
    private val sheet = File("src/main/assets/ime/theme/org.florisboard.themes/stylesheets/floris_night.json").readText()
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    private fun JsonObject.flag(key: String) = getValue(key).jsonPrimitive.boolean
    private fun labels(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject.text("label") }

    /** An extension directory with component [theme]'s stylesheet and some fonts and images. */
    private fun extDir(theme: ThemeExtensionComponentEditor): File {
        val dir = createTempDirectory("theme-editor").toFile()
        dir.resolve(theme.stylesheetPath()).apply { parentFile?.mkdirs() }.writeText(sheet)
        for (name in listOf("fonts/b.ttf", "fonts/a.ttf", "images/sky.png")) dir.resolve(name).apply { parentFile?.mkdirs() }.writeText("")
        return dir
    }
    private fun component() = ThemeExtensionComponentEditor(id = "sunny", label = "Sunny", authors = listOf("Ada", "Grace"),
        isNightTheme = false, stylesheetPath = "stylesheets/sunny.json")
    /** The extension editor holding [theme], as the workspace's is. */
    private fun editorOf(theme: ThemeExtensionComponentEditor) = ThemeExtensionEditor(ExtensionMeta(id = "local.my.themes",
        version = "0.0.0", title = "My themes", maintainers = listOf(ExtensionMaintainer(name = "Local")),
        license = "(none specified)"), mutableListOf(), mutableStateListOf(theme))

    @Test fun theReportCarriesTheFileTheMetaAndTheNamesAndSaveWritesThemBack() {
        val theme = component()
        val dir = extDir(theme)
        try {
            val report = themeEditorReport(dir, "local.my.themes", theme, listOf("moon", "rain"), ThemeEditorNatives.UUID, "0", accent)
            assertEquals(SettingsUiThemeEditor.SIZE, report.size)
            assertEquals(listOf(SettingsUiThemeEditorOutcome.LOADED, sheet, "local.my.themes", "Ada\nGrace", "a.ttf/b.ttf", "sky.png"),
                listOf(SettingsUiThemeEditor.OUTCOME, SettingsUiThemeEditor.STYLESHEET, SettingsUiThemeEditor.EXTENSION,
                    SettingsUiThemeEditor.AUTHORS, SettingsUiThemeEditor.FONTS, SettingsUiThemeEditor.IMAGES).map { report[it] })
            val flags = theme.materialYouFlags
            assertEquals(listOf("stylesheets/sunny.json", flags.paletteStyle.name, flags.contrastLevel.name, flags.specVersion.name,
                PaletteStyle.entries.joinToString("/") { it.name }, Contrast.entries.joinToString("/") { it.name },
                ColorSpec.SpecVersion.entries.joinToString("/") { it.name }, "moon/rain"),
                listOf(SettingsUiThemeEditor.PATH, SettingsUiThemeEditor.PALETTE, SettingsUiThemeEditor.CONTRAST,
                    SettingsUiThemeEditor.SPEC, SettingsUiThemeEditor.PALETTES, SettingsUiThemeEditor.CONTRASTS,
                    SettingsUiThemeEditor.SPECS, SettingsUiThemeEditor.OTHER_IDS).map { report[it] })
            KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
                ThemeEditorNatives.open(n)
                ThemeEditorNatives.state(n, SettingsUiOp.THEME_EDITOR, *report)
                val r = ThemeEditorNatives.fullRules(sheet, 1).single()
                ThemeEditorNatives.state(n, SettingsUiOp.ACTIVATE, ThemeEditorNatives.deleteId(r, 0))
                val save = ThemeEditorNatives.effects(ThemeEditorNatives.state(n, SettingsUiOp.ACTIVATE, ThemeEditorNatives.SAVE_ID.toString()))
                    .last { it[0] == SettingsUiThemeEditorEffect.SAVE }
                assertEquals(listOf(ThemeEditorNatives.UUID, "0", "Sunny", "Ada\nGrace", WireFlag.FALSE, "sunny",
                    "stylesheets/sunny.json", flags.paletteStyle.name, flags.contrastLevel.name, flags.specVersion.name),
                    save.slice(listOf(1, 2) + (4..11)), "Rust sends back the meta Kotlin reported")
                theme.stylesheetEditor = ThemeEditorNatives.composeEditor(sheet)
                theme.stylesheetPathOnLoad = "stylesheets/before.json"
                val editor = editorOf(theme)
                themeEditorSaved(dir, editor, 0, save)
                val saved = editor.themes.single()
                assertNotSame(theme, saved, "a new component, which the extension editor draws again")
                assertNull(saved.stylesheetEditor, "the file is what handleSave copies now")
                assertEquals(theme.materialYouFlags, saved.materialYouFlags)
                assertEquals(listOf("sunny", "stylesheets/sunny.json", "stylesheets/before.json"),
                    listOf(saved.id, saved.stylesheetPath, saved.stylesheetPathOnLoad))
                val expected = ThemeEditorNatives.composeWrites(sheet) { removeProperty(r, 0) }
                assertEquals(expected, dir.resolve(theme.stylesheetPath()).readText())
            }
            val editor = editorOf(theme)
            val packet = listOf(SettingsUiThemeEditorEffect.SAVE, ThemeEditorNatives.UUID, "0", "{}", "Dusk", "Ada\n\nLinus",
                WireFlag.TRUE, "dusk", "", PaletteStyle.entries.last().name, Contrast.entries.last().name, "no such spec")
            themeEditorSaved(dir, editor, 0, packet)
            val dusk = editor.themes.single()
            assertEquals(Triple("Dusk", listOf("Ada", "Linus"), true), Triple(dusk.label, dusk.authors, dusk.isNightTheme))
            assertEquals(listOf("dusk", ""), listOf(dusk.id, dusk.stylesheetPath))
            assertEquals(MaterialYouFlags(PaletteStyle.entries.last(), Contrast.entries.last(), theme.materialYouFlags.specVersion),
                dusk.materialYouFlags, "a flag name no enum has keeps the old flag")
            assertEquals("{}", dir.resolve("stylesheets/dusk.json").takeIf { it.exists() }?.readText(),
                "written where the new id keeps it")
        } finally {
            dir.deleteRecursively()
        }
    }

    @Test fun aComposeEditorsStylesheetIsReportedOverTheFile() {
        val theme = component()
        val dir = extDir(theme)
        try {
            val r = ThemeEditorNatives.fullRules(sheet, 1).single()
            theme.stylesheetEditor = ThemeEditorNatives.composeEditor(sheet).apply { removeProperty(r, 0) }
            val report = themeEditorReport(dir, "local.my.themes", theme, emptyList(), ThemeEditorNatives.UUID, "0", accent)
            assertEquals(ThemeEditorNatives.composeWrites(sheet) { removeProperty(r, 0) }, report[SettingsUiThemeEditor.STYLESHEET])
            dir.resolve(theme.stylesheetPath()).delete()
            theme.stylesheetEditor = null
            assertEquals("", themeEditorReport(dir, "", theme, emptyList(), ThemeEditorNatives.UUID, "0", accent)[SettingsUiThemeEditor.STYLESHEET],
                "no file is an empty stylesheet, as loadThemeEditorStylesheet started empty")
            val byDefault = ThemeExtensionComponentEditor(id = "sunny", label = "Sunny", authors = listOf("Ada"))
            assertEquals("", themeEditorReport(dir, "", byDefault, emptyList(), ThemeEditorNatives.UUID, "0", accent)[SettingsUiThemeEditor.PATH],
                "a component keeping its stylesheet where its id says reports no path of its own")
        } finally {
            dir.deleteRecursively()
        }
    }

    @Test fun aComponentTheWorkspaceNoLongerHasShowsWhyAndNoPreview() {
        assertNull(themeComponent(null, "0"))
        KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
            ThemeEditorNatives.open(n)
            val s = ThemeEditorNatives.state(n, SettingsUiOp.THEME_EDITOR,
                *themeEditorRow(SettingsUiThemeEditorOutcome.MISSING, ThemeEditorNatives.UUID, "0"))
            assertTrue(ThemeEditorNatives.effects(s).none { it[0] == SettingsUiThemeEditorEffect.PREVIEW })
            assertContains(labels(s), "settings__theme_editor__missing_workspace")
            assertFalse(s.flag("canBack"), "Back leaves ThemeEditorActivity for the extension editor")
        }
    }

    @Test fun theDraftSurvivesRecreationAndTheEditorOnlyRestoresAsTheRoot() {
        KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
            ThemeEditorNatives.open(n)
            val r = ThemeEditorNatives.fullRules(sheet, 1).single()
            assertFalse(ThemeEditorNatives.loaded(n, sheet).flag("canBack"), "an unchanged editor's Back leaves the activity")
            assertTrue(ThemeEditorNatives.state(n, SettingsUiOp.ACTIVATE, ThemeEditorNatives.deleteId(r, 1)).flag("canBack"),
                "a draft holds Back to ask")
            val saved = n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
            assertEquals(listOf("9", ThemeEditorNatives.ROUTE), listOf(saved[0], saved[1]))
            KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { recreated ->
                ThemeEditorNatives.configure(recreated)
                val restored = ThemeEditorNatives.state(recreated, SettingsUiOp.RESTORE, *saved)
                assertEquals(ThemeEditorNatives.ROUTE, restored.text("route"))
                assertEquals(listOf(SettingsUiThemeEditorEffect.LOAD, ThemeEditorNatives.UUID, "0"),
                    ThemeEditorNatives.effects(restored).last { it[0] == SettingsUiThemeEditorEffect.LOAD })
                // The workspace's stylesheet comes back as it was; the draft replaces it.
                val loaded = ThemeEditorNatives.loaded(recreated, sheet)
                assertEquals(ThemeEditorNatives.composeWrites(sheet) { removeProperty(r, 1) },
                    ThemeEditorNatives.json(loaded, SettingsUiThemeEditorEffect.PREVIEW))
            }
            KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { tampered ->
                ThemeEditorNatives.configure(tampered)
                val above = saved.copyOf().also { it[1] = "settings/home|${ThemeEditorNatives.ROUTE}"; it[2] = "0|0" }
                assertEquals("settings/home", ThemeEditorNatives.state(tampered, SettingsUiOp.RESTORE, *above).text("route"),
                    "a saved state with the editor above home is refused")
            }
        }
    }
}
