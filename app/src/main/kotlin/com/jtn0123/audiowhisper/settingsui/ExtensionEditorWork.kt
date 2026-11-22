package com.jtn0123.audiowhisper.settingsui

import android.content.Context
import android.net.Uri
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.settings.advanced.BackupFiles
import dev.patrickgold.florisboard.cacheManager
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.ime.theme.ThemeExtension
import dev.patrickgold.florisboard.lib.ext.ExtensionDefaults
import dev.patrickgold.florisboard.lib.ext.ExtensionManager
import dev.patrickgold.florisboard.lib.io.FlorisRef
import dev.patrickgold.florisboard.lib.io.ZipUtils
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.florisboard.lib.android.readToFile
import org.florisboard.lib.kotlin.io.subDir
import org.florisboard.lib.kotlin.io.subFile
import org.florisboard.lib.kotlin.mimeTypeFilterOf
import java.io.File

/** The extension editor's file work, as Rust asks for it, off the main thread where ExtensionEditScreen and
 * ExtensionEditFilesScreen ran it on it: Save, a folder's Add, Rename and Delete, and a copied component's stylesheet.
 * Every path stays inside the workspace. No name, address or error text reaches Rust's log or this one. */
internal object ExtensionEditorWork {
    val JOBS = setOf(SettingsUiJob.EXT_SAVE, SettingsUiJob.EXT_FILE_ADD, SettingsUiJob.EXT_FILE_RENAME,
        SettingsUiJob.EXT_FILE_DELETE, SettingsUiJob.EXT_COMPONENT_COPY)

    suspend fun run(context: Context, p: List<String>, document: (String) -> Uri?): Array<String> = withContext(Dispatchers.IO) {
        val job = p[SettingsUiWork.JOB]
        val cache by context.cacheManager()
        val uuid = p[SettingsUiWork.EXTENSION]
        val named = if (job == SettingsUiJob.EXT_SAVE) uuid else p[SettingsUiWork.NAME]
        val workspace = editorWorkspace(cache, uuid)
            ?: return@withContext BackupFiles.done(job, SettingsUiJobResult.GONE, SettingsUiDone.NAME to named)
        val result = runCatching {
            when (job) {
                SettingsUiJob.EXT_SAVE -> save(context, workspace, p)
                SettingsUiJob.EXT_FILE_ADD -> add(context, workspace, p, document)
                SettingsUiJob.EXT_FILE_RENAME -> rename(workspace, p[SettingsUiWork.TEXT], p[SettingsUiWork.NAME])
                SettingsUiJob.EXT_FILE_DELETE -> delete(workspace, p[SettingsUiWork.NAME])
                else -> copy(context, workspace, p)
            }
        }.getOrDefault(SettingsUiJobResult.FAILED)
        val from = if (job == SettingsUiJob.EXT_FILE_RENAME) p[SettingsUiWork.TEXT] else ""
        BackupFiles.done(job, result, SettingsUiDone.NAME to named, SettingsUiDone.TEXT to from)
    }

    /** handleSave made whole: the manifest Rust wrote, the folders and each theme's stylesheet zipped in the workspace,
     * then moved over the installed archive, or into the theme folder for a new extension, in one step, so a save cut
     * short leaves the old archive as it was. A new one never replaces another's archive. The workspace then closes. */
    private fun save(context: Context, workspace: EditorWorkspace, p: List<String>): String {
        val name = p[SettingsUiWork.NAME]
        val target = p[SettingsUiWork.TARGET]
        val destination = destination(context, target, name) ?: return SettingsUiJobResult.FAILED
        if (target.isEmpty() && destination.exists()) return SettingsUiJobResult.TAKEN
        pack(workspace, p[SettingsUiWork.TEXT], decodeDraft(p[SettingsUiWork.TEXT]))
        val archive = workspace.dir.subFile(name)
        ZipUtils.zip(workspace.saverDir, archive)
        destination.parentFile?.mkdirs()
        if (!archive.renameTo(destination)) return SettingsUiJobResult.FAILED
        workspace.close()
        return SettingsUiJobResult.OK
    }
    /** Where Save puts archive [name]: the installed extension [target]'s own file, or the theme folder. */
    private fun destination(context: Context, target: String, name: String): File? {
        if (File(name).name != name || !name.endsWith(".${ExtensionDefaults.FILE_EXTENSION}")) return null
        if (target.isEmpty()) return FlorisRef.internal(ExtensionManager.IME_THEME_PATH).subRef(name).absoluteFile(context)
        val extensions by context.extensionManager()
        return extensions.getExtensionById(target)?.sourceRef?.takeIf { it.isInternal }?.absoluteFile(context)
    }
    /** The saver folder as handleSave filled it: the manifest, the fonts and images, and each theme's stylesheet. */
    private fun pack(workspace: EditorWorkspace, manifest: String, ext: ThemeExtension) {
        val saver = workspace.saverDir
        saver.deleteRecursively()
        saver.mkdirs()
        saver.subFile(ExtensionDefaults.MANIFEST_FILE_NAME).writeText(manifest)
        for (folder in listOf(SettingsUiExtEditorFolder.FONTS, SettingsUiExtEditorFolder.IMAGES)) {
            workspace.extDir.subDir(folder).takeIf { it.exists() }?.copyRecursively(saver.subDir(folder), overwrite = true)
        }
        for (theme in ext.themes) {
            val from = inside(workspace.extDir, theme.stylesheetPath())?.takeIf { it.exists() } ?: continue
            from.copyTo(saver.subFile(theme.stylesheetPath()), overwrite = true)
        }
    }
    /** A folder's Add: the picked document copied in when its media type is one the folder takes, as the files
     * screen's filter matched it; never over a file already there. */
    private fun add(context: Context, workspace: EditorWorkspace, p: List<String>, document: (String) -> Uri?): String {
        val uri = document(p[SettingsUiWork.DOCUMENT]) ?: return SettingsUiJobResult.GONE
        val file = inside(workspace.extDir, p[SettingsUiWork.NAME]) ?: return SettingsUiJobResult.FAILED
        val types = p[SettingsUiWork.TEXT].split('|').toTypedArray()
        if (!mimeTypeFilterOf(*types).matches(context.contentResolver.getType(uri))) return SettingsUiJobResult.WRONG_TYPE
        if (file.exists()) return SettingsUiJobResult.TAKEN
        val part = workspace.dir.subFile("added.part")
        context.contentResolver.readToFile(uri, part)
        file.parentFile?.mkdirs()
        return if (part.renameTo(file)) SettingsUiJobResult.OK else SettingsUiJobResult.FAILED.also { part.delete() }
    }
    private fun rename(workspace: EditorWorkspace, from: String, to: String): String {
        val old = inside(workspace.extDir, from) ?: return SettingsUiJobResult.FAILED
        val new = inside(workspace.extDir, to) ?: return SettingsUiJobResult.FAILED
        return when {
            !old.exists() -> SettingsUiJobResult.GONE
            new.exists() -> SettingsUiJobResult.TAKEN
            old.renameTo(new) -> SettingsUiJobResult.OK
            else -> SettingsUiJobResult.FAILED
        }
    }
    /** One already gone is done. */
    private fun delete(workspace: EditorWorkspace, name: String): String {
        val file = inside(workspace.extDir, name) ?: return SettingsUiJobResult.FAILED
        return if (!file.exists() || file.delete()) SettingsUiJobResult.OK else SettingsUiJobResult.FAILED
    }
    /** Create component's copy: the stylesheet in the installed extension `TARGET`'s archive, or in the workspace,
     * written where the new component keeps its own. One that is not there copies as none, where Compose's copy from
     * the same extension threw. */
    private fun copy(context: Context, workspace: EditorWorkspace, p: List<String>): String {
        val to = inside(workspace.extDir, p[SettingsUiWork.NAME]) ?: return SettingsUiJobResult.FAILED
        val path = p[SettingsUiWork.TEXT]
        val target = p[SettingsUiWork.TARGET]
        val text = if (target.isEmpty()) {
            inside(workspace.extDir, path)?.takeIf { it.exists() }?.readText()
        } else {
            val extensions by context.extensionManager()
            val ref = extensions.getExtensionById(target)?.sourceRef ?: return SettingsUiJobResult.FAILED
            ZipUtils.readFileFromArchive(context, ref, path).getOrNull()
        }
        if (text != null) {
            to.parentFile?.mkdirs()
            to.writeText(text)
        }
        return SettingsUiJobResult.OK
    }
}

/** [path] under [dir], or null when it names a place outside it. */
internal fun inside(dir: File, path: String): File? {
    val root = dir.canonicalFile
    val file = File(root, path).canonicalFile
    return file.takeIf { it.path.startsWith(root.path + File.separator) }
}
