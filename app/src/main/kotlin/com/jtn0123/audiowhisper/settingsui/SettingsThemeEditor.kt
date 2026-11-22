package com.jtn0123.audiowhisper.settingsui

import androidx.activity.ComponentActivity
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.isUnspecified
import com.jtn0123.audiowhisper.rust.*
import com.materialkolor.Contrast
import com.materialkolor.PaletteStyle
import com.materialkolor.dynamiccolor.ColorSpec
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.app.PrettyPrintConfig
import dev.patrickgold.florisboard.cacheManager
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentEditor
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionEditor
import dev.patrickgold.florisboard.ime.theme.ThemeManager
import dev.patrickgold.florisboard.ime.theme.extPreviewTheme
import dev.patrickgold.florisboard.lib.cache.CacheManager
import dev.patrickgold.florisboard.themeManager
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.kotlin.io.subDir
import org.florisboard.lib.kotlin.io.subFile
import org.florisboard.lib.snygg.SnyggStylesheet
import org.florisboard.lib.snygg.ui.snyggPalette
import java.io.File

/** What the native theme editor needs from the extension editor's workspace, which lives in the app's CacheManager
 * and only Kotlin reaches: the component's stylesheet text, meta and file names as one `THEME_EDITOR` report; on Save,
 * the stylesheet Rust wrote put back into the workspace with the meta, as ThemeEditorScreen's edits were for
 * ExtensionEditScreen's `handleSave` to zip; and the draft on the keyboard through `previewThemeInfo`. */
internal class SettingsThemeEditor(private val activity: ComponentActivity, private val report: (Array<String>) -> Unit) {
    private val cacheManager by activity.cacheManager()
    private val themeManager by activity.themeManager()
    private val prefs by FlorisPreferenceStore

    /** A `themeEditor`, `themeEditorSave` or `themeEditorPreview` effect. */
    fun execute(p: List<String>) {
        when (p[0]) {
            SettingsUiThemeEditorEffect.LOAD -> report(load(p[1], p[2]))
            SettingsUiThemeEditorEffect.SAVE -> save(p)
            SettingsUiThemeEditorEffect.PREVIEW -> preview(p)
        }
    }
    /** The settings view is gone: the keyboard shows its own theme again, as leaving ThemeEditorScreen did. */
    fun release() { themeManager.previewThemeInfo.value = null }

    private fun component(uuid: String, index: String) =
        themeComponent(editorWorkspace(cacheManager, uuid), index)

    private fun load(uuid: String, index: String): Array<String> {
        val found = component(uuid, index)
            ?: return themeEditorRow(SettingsUiThemeEditorOutcome.MISSING, uuid, index)
        val (workspace, theme) = found
        val others = workspace.editor?.themes.orEmpty().filter { it !== theme }.map { it.id }
        return themeEditorReport(workspace.extDir, workspace.editor?.meta?.id.orEmpty(), theme, others, uuid, index, accent())
    }
    private fun accent() = keyboardAccent(prefs.theme.accentColor.get()) {
        Color(activity.resources.getColor(android.R.color.system_accent1_500, activity.theme))
    }
    /** Rust's stylesheet becomes the workspace's file, which the extension editor's Save copies, and the draft with the
     * new meta is kept for the extension editor to take as it comes back. */
    private fun save(p: List<String>) {
        val (workspace, _) = component(p[1], p[2]) ?: return
        workspace.update { themeEditorSaved(workspace.extDir, this, p[2].toInt(), p) }
        runCatching { keepThemed(workspace) }
    }
    /** ThemeEditorScreen's `previewThemeInfo`, built from Rust's draft; a bare packet gives the keyboard its theme. */
    private fun preview(p: List<String>) {
        if (p.size < 4) { themeManager.previewThemeInfo.value = null; return }
        val (workspace, theme) = component(p[1], p[2]) ?: return
        val stylesheet = SnyggStylesheet.fromJson(p[3], PrettyPrintConfig).getOrNull() ?: return
        themeManager.previewThemeInfo.value = ThemeManager.ThemeInfo.DEFAULT.copy(
            name = extPreviewTheme(System.currentTimeMillis().toString()),
            config = runCatching { theme.build() }.getOrDefault(ThemeManager.ThemeInfo.DEFAULT.config),
            stylesheet = stylesheet,
            loadedDir = workspace.extDir,
        )
    }
}

/** Component [index] of [workspace]'s theme extension, or null when either is gone. */
internal fun themeComponent(workspace: CacheManager.ExtEditorWorkspace<ThemeExtensionEditor>?, index: String) =
    workspace?.editor?.themes?.getOrNull(index.toIntOrNull() ?: -1)?.let { workspace to it }

/** A `THEME_EDITOR` report: [outcome] for component [index] of workspace [uuid], [fields] given and the rest empty. */
internal fun themeEditorRow(outcome: String, uuid: String, index: String, vararg fields: Pair<Int, String>): Array<String> {
    val given = mapOf(SettingsUiThemeEditor.OUTCOME to outcome, SettingsUiThemeEditor.WORKSPACE to uuid,
        SettingsUiThemeEditor.COMPONENT to index) + fields
    return SettingsUiThemeEditor.row(*Array(SettingsUiThemeEditor.SIZE) { it to given[it].orEmpty() })
}

/** The accent the keyboard's palette is made from, FlorisImeTheme's `systemAccentOrDefault(prefs.theme.accentColor)`
 * outside composition: the [chosen] one, or the [system]'s when none is. */
internal fun keyboardAccent(chosen: Color, system: () -> Color) = chosen.takeUnless { it.isUnspecified } ?: system()

/** The report of a loaded component of extension [extension] unpacked in [extDir]: its stylesheet as a Compose editor
 * left it or as the file has it (empty when there is none, as loadThemeEditorStylesheet started empty), its meta with
 * the names its Material You flags can take, the [others]' ids, the font and image names, and the palette the keyboard
 * draws it in for accent [primary]. */
internal fun themeEditorReport(extDir: File, extension: String, theme: ThemeExtensionComponentEditor, others: List<String>,
    uuid: String, index: String, primary: Color): Array<String> {
    val file = extDir.subFile(theme.stylesheetPath())
    val stylesheet = theme.stylesheetEditor?.let { runCatching { it.build().toJson(PrettyPrintConfig).getOrThrow() }.getOrNull() }
        ?: file.takeIf { it.exists() }?.readText().orEmpty()
    val names = { dir: String -> extDir.subDir(dir).listFiles()?.map { it.name }?.sorted().orEmpty().joinToString("/") }
    val flags = theme.materialYouFlags
    val all = { entries: List<Enum<*>> -> entries.joinToString("/") { it.name } }
    val palette = snyggPalette(primary, flags)
    val colors = { part: List<Int> -> part.joinToString("/") { it.toUInt().toString() } }
    return themeEditorRow(SettingsUiThemeEditorOutcome.LOADED, uuid, index,
        SettingsUiThemeEditor.STYLESHEET to stylesheet, SettingsUiThemeEditor.EXTENSION to extension,
        SettingsUiThemeEditor.ID to theme.id, SettingsUiThemeEditor.LABEL to theme.label,
        SettingsUiThemeEditor.AUTHORS to theme.authors.joinToString("\n"),
        SettingsUiThemeEditor.NIGHT to WireFlag.encode(theme.isNightTheme), SettingsUiThemeEditor.PATH to theme.stylesheetPath,
        SettingsUiThemeEditor.FONTS to names("fonts"), SettingsUiThemeEditor.IMAGES to names("images"),
        SettingsUiThemeEditor.PALETTE to flags.paletteStyle.name, SettingsUiThemeEditor.CONTRAST to flags.contrastLevel.name,
        SettingsUiThemeEditor.SPEC to flags.specVersion.name, SettingsUiThemeEditor.PALETTES to all(PaletteStyle.entries),
        SettingsUiThemeEditor.CONTRASTS to all(Contrast.entries), SettingsUiThemeEditor.SPECS to all(ColorSpec.SpecVersion.entries),
        SettingsUiThemeEditor.OTHER_IDS to others.joinToString("/"),
        SettingsUiThemeEditor.COLOR_NAMES to palette.names.joinToString("/"),
        SettingsUiThemeEditor.LIGHT_COLORS to colors(palette.colors.take(palette.names.size)),
        SettingsUiThemeEditor.DARK_COLORS to colors(palette.colors.drop(palette.names.size)))
}

/** A `themeEditorSave` [p] put into component [index] of [editor], the extension unpacked in [extDir]: the meta a new
 * component in the old one's place, and the stylesheet Rust wrote the file where that component keeps it, which
 * `handleSave` copies when no Compose editor holds one. ThemeEditorScreen's edits were drawn as the extension editor
 * came back into composition; it stays composed under ThemeEditorActivity and skips a component it already drew, so
 * only a new one shows the change. A flag name no enum has keeps the old flag. */
internal fun themeEditorSaved(extDir: File, editor: ThemeExtensionEditor, index: Int, p: List<String>) {
    val theme = editor.themes[index]
    val old = theme.materialYouFlags
    val flags = MaterialYouFlags(PaletteStyle.entries.named(p[9]) ?: old.paletteStyle,
        Contrast.entries.named(p[10]) ?: old.contrastLevel, ColorSpec.SpecVersion.entries.named(p[11]) ?: old.specVersion)
    val authors = p[5].lines().filter { it.isNotEmpty() }
    val saved = ThemeExtensionComponentEditor(p[7], p[4], authors, WireFlag.decode(p[6]), flags, p[8])
        .also { it.stylesheetPathOnLoad = theme.stylesheetPathOnLoad }
    val file = extDir.subFile(saved.stylesheetPath())
    file.parentFile?.mkdirs()
    file.writeText(p[3])
    editor.themes[index] = saved
}

private fun <E : Enum<E>> List<E>.named(name: String) = firstOrNull { it.name == name }
