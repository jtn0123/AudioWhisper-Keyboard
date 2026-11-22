package com.jtn0123.audiowhisper.settingsui

import androidx.activity.ComponentActivity
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.settings.theme.PrettyPrintConfig
import dev.patrickgold.florisboard.cacheManager
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentEditor
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionEditor
import dev.patrickgold.florisboard.ime.theme.ThemeManager
import dev.patrickgold.florisboard.ime.theme.extPreviewTheme
import dev.patrickgold.florisboard.lib.cache.CacheManager
import dev.patrickgold.florisboard.themeManager
import org.florisboard.lib.kotlin.io.subDir
import org.florisboard.lib.kotlin.io.subFile
import org.florisboard.lib.snygg.SnyggStylesheet

/** What the native theme editor needs from the extension editor's workspace, which lives in the app's CacheManager
 * and only Kotlin reaches: the component's stylesheet text, meta and file names as one `THEME_EDITOR` report; on Save,
 * the stylesheet Rust wrote put back into the workspace with the meta, as ThemeEditorScreen's edits were for
 * ExtensionEditScreen's `handleSave` to zip; and the draft on the keyboard through `previewThemeInfo`. */
internal class SettingsThemeEditor(activity: ComponentActivity, private val report: (Array<String>) -> Unit) {
    private val cacheManager by activity.cacheManager()
    private val themeManager by activity.themeManager()

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
        themeComponent(cacheManager.themeExtEditor.getWorkspaceByUuid(uuid)?.takeIf { it.isOpen() }, index)

    private fun load(uuid: String, index: String): Array<String> {
        val found = component(uuid, index)
            ?: return themeEditorRow(SettingsUiThemeEditorOutcome.MISSING, uuid, index)
        val (workspace, theme) = found
        return themeEditorReport(workspace, theme, uuid, index)
    }
    /** Rust's stylesheet becomes the workspace's file, which `handleSave` copies when no Compose editor holds one. */
    private fun save(p: List<String>) {
        val (workspace, theme) = component(p[1], p[2]) ?: return
        val file = workspace.extDir.subFile(theme.stylesheetPath())
        file.parentFile?.mkdirs()
        file.writeText(p[3])
        workspace.update {
            theme.stylesheetEditor = null
            theme.label = p[4]
            theme.authors = p[5].lines().filter { it.isNotEmpty() }
            theme.isNightTheme = WireFlag.decode(p[6])
        }
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

/** The report of a loaded component: its stylesheet as a Compose editor left it or as the workspace's file has it
 * (empty when there is none, as loadThemeEditorStylesheet started empty), its meta, and the workspace's files. */
internal fun themeEditorReport(workspace: CacheManager.ExtEditorWorkspace<ThemeExtensionEditor>,
    theme: ThemeExtensionComponentEditor, uuid: String, index: String): Array<String> {
    val path = theme.stylesheetPath()
    val file = workspace.extDir.subFile(path)
    val stylesheet = theme.stylesheetEditor?.let { runCatching { it.build().toJson(PrettyPrintConfig).getOrThrow() }.getOrNull() }
        ?: file.takeIf { it.exists() }?.readText().orEmpty()
    val names = { dir: String -> workspace.extDir.subDir(dir).listFiles()?.map { it.name }?.sorted().orEmpty().joinToString("/") }
    return themeEditorRow(SettingsUiThemeEditorOutcome.LOADED, uuid, index,
        SettingsUiThemeEditor.STYLESHEET to stylesheet, SettingsUiThemeEditor.EXTENSION to workspace.editor?.meta?.id.orEmpty(),
        SettingsUiThemeEditor.ID to theme.id, SettingsUiThemeEditor.LABEL to theme.label,
        SettingsUiThemeEditor.AUTHORS to theme.authors.joinToString("\n"),
        SettingsUiThemeEditor.NIGHT to WireFlag.encode(theme.isNightTheme), SettingsUiThemeEditor.PATH to path,
        SettingsUiThemeEditor.FONTS to names("fonts"), SettingsUiThemeEditor.IMAGES to names("images"))
}
