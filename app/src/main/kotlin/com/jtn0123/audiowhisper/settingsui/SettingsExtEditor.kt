package com.jtn0123.audiowhisper.settingsui

import androidx.activity.ComponentActivity
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.lifecycleScope
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.ThemeEditorActivity
import dev.patrickgold.florisboard.cacheManager
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.ime.theme.ThemeExtension
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentImpl
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionEditor
import dev.patrickgold.florisboard.lib.cache.CacheManager
import dev.patrickgold.florisboard.lib.ext.Extension
import dev.patrickgold.florisboard.lib.ext.ExtensionJsonConfig
import dev.patrickgold.florisboard.lib.io.ZipUtils
import dev.patrickgold.florisboard.themeManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.serialization.encodeToString
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.kotlin.io.subDir
import org.florisboard.lib.kotlin.io.subFile

internal typealias EditorWorkspace = CacheManager.ExtEditorWorkspace<ThemeExtensionEditor>

/** Rust's copy of the draft, as the last `extEditorSync` sent it. */
private const val DRAFT = "draft.json"
/** The theme editor's copy, once it saved a component, until Rust takes it. */
private const val THEMED = "themed.json"
/** One writer for the whole process, so the copies land in the order Rust sent them, a turn of the screen between. */
private val writer = Dispatchers.IO.limitedParallelism(1)
private val copies = CoroutineScope(SupervisorJob() + writer)

/** What the native extension editor needs that only Kotlin reaches: its workspace in the app's CacheManager, the
 * installed extension unpacked into it, the files it holds, and a copy of the draft there for the theme editor, which
 * reads and changes `workspace.editor`; and the theme editor itself, its own activity over this one. Rust holds the
 * draft and decides every screen, check and message. */
internal class SettingsExtEditor(private val activity: ComponentActivity, private val report: (Array<String>) -> Unit) {
    private val cacheManager by activity.cacheManager()
    /** Both made here, on the main thread: each observes or sets LiveData as it is made, which throws anywhere else, and
     * after the process ended the first to read them may be [ready], on the file thread. */
    private val extensionManager = activity.extensionManager().value
    private val themeManager = activity.themeManager().value
    /** The workspace Rust has open and the extension it edits, reported again when the theme editor changed it. */
    private var open: Pair<String, String>? = null
    private val resumed = LifecycleEventObserver { _, event -> if (event == Lifecycle.Event.ON_RESUME) returned() }
    init { activity.lifecycle.addObserver(resumed) }

    /** A `SettingsUiExtEditorEffect`. */
    fun execute(p: List<String>) {
        when (p[0]) {
            SettingsUiExtEditorEffect.OPEN ->
                open(p[SettingsUiExtEditorOpen.EXTENSION], p[SettingsUiExtEditorOpen.WORKSPACE], p[SettingsUiExtEditorOpen.MANIFEST])
            SettingsUiExtEditorEffect.SYNC -> sync(p[1], p[2])
            SettingsUiExtEditorEffect.CLOSE -> close(p[1])
            SettingsUiExtEditorEffect.THEME -> ThemeEditorActivity.open(activity, p[1], p[2])
        }
    }
    fun release() = activity.lifecycle.removeObserver(resumed)

    /** The theme editor reads the draft at once; the copy on disk, for after the process ends, follows. */
    private fun sync(uuid: String, text: String) {
        val workspace = cacheManager.themeExtEditor.getWorkspaceByUuid(uuid) ?: return
        workspace.editor = runCatching { decodeDraft(text).edit() }.getOrElse { return }
        copies.launch { runCatching { workspace.dir.subFile(DRAFT).writeText(text) } }
    }
    private fun open(id: String, uuid: String, manifest: String) = activity.lifecycleScope.launch {
        val rows = withContext(writer) {
            runCatching { if (uuid.isEmpty()) fresh(id) else takeUp(id, uuid, manifest) }
                .getOrElse { listOf(editorRow(SettingsUiExtEditorKind.WORKSPACE, uuid, id, SettingsUiExtEditorOutcome.FAILED)) }
        }
        val head = rows.first()
        if (head[SettingsUiExtEditor.ID] == SettingsUiExtEditorOutcome.READY) open = head[SettingsUiExtEditor.WORKSPACE] to id
        report(rows.flatMap { it.asList() }.toTypedArray())
    }
    /** A new workspace, with the installed theme extension [id] unpacked into it, or empty to create one, as
     * ExtensionEditScreen's `getOrCreateWorkspace` made it. Workspaces a lost process left are deleted first. */
    private fun fresh(id: String): List<Array<String>> {
        val container = cacheManager.themeExtEditor
        container.deleteOthers(keep = "")
        val ext = if (id.isEmpty()) null else extensionManager.getExtensionById(id) as? ThemeExtension
            ?: return listOf(editorRow(SettingsUiExtEditorKind.WORKSPACE, "", id, SettingsUiExtEditorOutcome.MISSING))
        val workspace = container.new()
        val unpacked = ext == null || runCatching {
            ZipUtils.unzip(activity, requireNotNull(ext.sourceRef), workspace.extDir).getOrThrow()
        }.isSuccess
        if (!unpacked) {
            workspace.close()
            return listOf(editorRow(SettingsUiExtEditorKind.WORKSPACE, "", id, SettingsUiExtEditorOutcome.FAILED))
        }
        val text = ext?.let { ExtensionJsonConfig.encodeToString<Extension>(it) }.orEmpty()
        if (ext != null) keepDraft(workspace, text)
        return ready(workspace, id, text, changed = false)
    }
    /** The workspace [uuid] a saved state names, taken up again: Rust's [manifest] stays the draft unless the theme
     * editor saved a component since, when its copy is the draft. */
    private fun takeUp(id: String, uuid: String, manifest: String): List<Array<String>> {
        val workspace = editorWorkspace(cacheManager, uuid)
            ?: return listOf(editorRow(SettingsUiExtEditorKind.WORKSPACE, uuid, id, SettingsUiExtEditorOutcome.GONE))
        val themed = takeThemed(workspace)
        if (themed == null) keepDraft(workspace, manifest)
        return ready(workspace, id, themed ?: manifest, changed = themed != null)
    }
    /** Back from the theme editor: a component it saved changes Rust's draft. */
    private fun returned() {
        val (uuid, id) = open ?: return
        activity.lifecycleScope.launch {
            val rows = withContext(writer) {
                val workspace = editorWorkspace(cacheManager, uuid) ?: return@withContext null
                runCatching { takeThemed(workspace)?.let { ready(workspace, id, it, changed = true) } }.getOrNull()
            } ?: return@launch
            report(rows.flatMap { it.asList() }.toTypedArray())
        }
    }
    private fun close(uuid: String) {
        if (open?.first == uuid) open = null
        copies.launch { cacheManager.themeExtEditor.adopt(uuid)?.close() }
    }
    private fun ready(workspace: EditorWorkspace, id: String, text: String, changed: Boolean): List<Array<String>> {
        val uuid = workspace.uuid
        val head = editorRow(SettingsUiExtEditorKind.WORKSPACE, uuid, id, SettingsUiExtEditorOutcome.READY,
            label = System.currentTimeMillis().toString(), detail = MaterialYouFlags().names(), text = text, flag = changed)
        val files = listOf(SettingsUiExtEditorFolder.FONTS, SettingsUiExtEditorFolder.IMAGES).flatMap { folder ->
            val names = workspace.extDir.subDir(folder).listFiles().orEmpty().filter { it.isFile }.map { it.name }.sorted()
            names.map { editorRow(SettingsUiExtEditorKind.FILE, uuid, id = folder, label = it) }
        }
        val themes = themeManager.indexedThemeConfigs.value.first.map { (name, theme) ->
            editorRow(SettingsUiExtEditorKind.THEME, uuid, name.extensionId, name.componentId, theme.label,
                theme.materialYouFlags.names(), theme.authors.joinToString("\n"), theme.isNightTheme, theme.stylesheetPath())
        }
        val all = listOf(extensionManager.keyboardExtensions, extensionManager.themes, extensionManager.languagePacks)
        val installed = all.flatMap { it.value.orEmpty() }.map { editorRow(SettingsUiExtEditorKind.INSTALLED, uuid, it.meta.id) }
        return listOf(head) + files + themes + installed
    }
}

/** Editor workspace [uuid], taken up from its folder when the process that opened it is gone, with the copy of the
 * draft it keeps as its editor; null when it is closed. */
internal fun editorWorkspace(cache: CacheManager, uuid: String): EditorWorkspace? {
    val workspace = cache.themeExtEditor.adopt(uuid)?.takeIf { it.isOpen() } ?: return null
    if (workspace.editor == null) {
        val copy = listOf(THEMED, DRAFT).map { workspace.dir.subFile(it) }.firstOrNull { it.exists() } ?: return workspace
        workspace.editor = runCatching { decodeDraft(copy.readText()).edit() }.getOrNull()
    }
    return workspace
}

/** Rust's draft [text] becomes the workspace's editor, for the theme editor, and its copy on disk. */
private fun keepDraft(workspace: EditorWorkspace, text: String) {
    workspace.editor = decodeDraft(text).edit()
    workspace.dir.subFile(DRAFT).writeText(text)
}

/** The theme editor saved a component of [workspace]: its draft is kept for Rust to take as it comes back. */
internal fun keepThemed(workspace: EditorWorkspace) {
    val editor = workspace.editor ?: return
    workspace.dir.subFile(THEMED).writeText(rawDraft(editor))
}

/** The theme editor's copy, which becomes Rust's; null when it saved nothing since Rust last took it. */
private fun takeThemed(workspace: EditorWorkspace): String? {
    val themed = workspace.dir.subFile(THEMED).takeIf { it.exists() } ?: return null
    val text = themed.readText()
    themed.renameTo(workspace.dir.subFile(DRAFT))
    return text
}

internal fun decodeDraft(text: String) = ExtensionJsonConfig.decodeFromString<Extension>(text) as ThemeExtension

/** [editor] as kotlinx encodes it with nothing trimmed or dropped, as Rust's copy is: a component as it was typed. */
internal fun rawDraft(editor: ThemeExtensionEditor): String = ExtensionJsonConfig.encodeToString<Extension>(ThemeExtension(
    editor.meta, editor.dependencies.takeUnless { it.isEmpty() }?.toList(), editor.themes.map {
        ThemeExtensionComponentImpl(it.id, it.label, it.authors, it.isNightTheme, it.materialYouFlags,
            it.stylesheetPath.takeUnless(String::isEmpty))
    }))

private fun MaterialYouFlags.names() = "${paletteStyle.name}/${contrastLevel.name}/${specVersion.name}"

/** One `SettingsUiExtEditor` row of a report. */
internal fun editorRow(kind: String, workspace: String, extension: String = "", id: String = "", label: String = "",
    detail: String = "", text: String = "", flag: Boolean = false, path: String = "") = SettingsUiExtEditor.row(
    SettingsUiExtEditor.KIND to kind, SettingsUiExtEditor.WORKSPACE to workspace, SettingsUiExtEditor.EXTENSION to extension,
    SettingsUiExtEditor.ID to id, SettingsUiExtEditor.LABEL to label, SettingsUiExtEditor.DETAIL to detail,
    SettingsUiExtEditor.TEXT to text, SettingsUiExtEditor.FLAG to WireFlag.encode(flag), SettingsUiExtEditor.PATH to path)
