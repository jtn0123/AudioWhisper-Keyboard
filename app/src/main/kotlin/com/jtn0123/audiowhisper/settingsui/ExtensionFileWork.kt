package com.jtn0123.audiowhisper.settingsui

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import android.text.format.Formatter
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.settings.advanced.BackupFiles
import dev.patrickgold.florisboard.cacheManager
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.lib.cache.CacheManager
import dev.patrickgold.florisboard.lib.io.FileRegistry
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** The extension importer's and Export's file work, as Rust asks for it, off the main thread where Compose ran it on
 * it: reading picked documents into an import workspace, importing the files Rust chose from one, and writing an
 * extension into a picked document. No address, name or error text reaches Rust or the log. */
internal object ExtensionFileWork {
    val JOBS = setOf(SettingsUiJob.EXT_READ, SettingsUiJob.EXT_IMPORT, SettingsUiJob.EXT_EXPORT)

    suspend fun run(context: Context, p: List<String>, document: (String) -> Uri?): Array<String> = withContext(Dispatchers.IO) {
        when (p[SettingsUiWork.JOB]) {
            SettingsUiJob.EXT_READ -> read(context, p[SettingsUiWork.DOCUMENT].split('|').map(document))
            SettingsUiJob.EXT_IMPORT -> import(context, p[SettingsUiWork.EXTENSION], p[SettingsUiWork.TEXT])
            else -> export(context, p[SettingsUiWork.EXTENSION], document(p[SettingsUiWork.DOCUMENT]))
        }
    }
    /** readFromUriIntoCache, which also reads each archive's manifest; the workspace's id. */
    private suspend fun read(context: Context, uris: List<Uri?>): Array<String> {
        val job = SettingsUiJob.EXT_READ
        if (null in uris) return BackupFiles.done(job, SettingsUiJobResult.GONE)
        val cache by context.cacheManager()
        return runCatching { cache.readFromUriIntoCache(uris.filterNotNull()) }.fold(
            { BackupFiles.done(job, SettingsUiJobResult.OK, SettingsUiDone.NAME to it.uuid) },
            { BackupFiles.done(job, SettingsUiJobResult.UNREADABLE) })
    }
    /** The files at [indexes], in order, as Import imported them, then the workspace closed. */
    private fun import(context: Context, uuid: String, indexes: String): Array<String> {
        val job = SettingsUiJob.EXT_IMPORT
        val cache by context.cacheManager()
        val extensions by context.extensionManager()
        val workspace = cache.importer.getWorkspaceByUuid(uuid) ?: return BackupFiles.done(job, SettingsUiJobResult.GONE)
        val imported = runCatching {
            for (i in indexes.split(',')) extensions.import(requireNotNull(workspace.inputFileInfos[i.toInt()].ext))
        }
        if (imported.isFailure) return BackupFiles.done(job, SettingsUiJobResult.FAILED)
        workspace.close()
        return BackupFiles.done(job, SettingsUiJobResult.OK)
    }
    /** The extension written into [uri] as ExtensionExportScreen wrote it. A document it could not write is deleted,
     * so no broken archive is left behind. */
    private fun export(context: Context, id: String, uri: Uri?): Array<String> {
        val job = SettingsUiJob.EXT_EXPORT
        uri ?: return BackupFiles.done(job, SettingsUiJobResult.GONE)
        val extensions by context.extensionManager()
        val ext = extensions.getExtensionById(id) ?: return BackupFiles.done(job, SettingsUiJobResult.FAILED)
        val written = writeOrDelete({ extensions.export(ext, uri) }) { DocumentsContract.deleteDocument(context.contentResolver, uri) }
        return BackupFiles.done(job, if (written) SettingsUiJobResult.OK else SettingsUiJobResult.UNWRITABLE)
    }
}

/** Whether [write] finished; when it threw, [delete] removes what it half wrote, and a failure to delete is ignored. */
internal fun writeOrDelete(write: () -> Unit, delete: () -> Unit): Boolean {
    if (runCatching(write).isSuccess) return true
    runCatching(delete)
    return false
}

/** The `EXTENSION_FILES` report on workspace [uuid] holding [files], null when it is closed: its id, then each file it
 * read, what Android calls its [size], whether it is typed as a `.flex`, and the archive read from it, which is [core]
 * when the installed extension with its id ships with the app. */
internal fun importRows(uuid: String, files: List<CacheManager.FileInfo>?, size: (Long) -> String,
    core: (String) -> Boolean): Array<String> {
    val out = mutableListOf(importRow(SettingsUiImportKind.WORKSPACE, extension = uuid, flag = files != null))
    for (info in files.orEmpty()) {
        val flex = FileRegistry.matchesFileFilter(info, listOf(FileRegistry.FlexExtension))
        out += importRow(SettingsUiImportKind.FILE, label = info.file.name, detail = info.mediaType.orEmpty(),
            text = size(info.size), flag = flex)
        val ext = info.ext ?: continue
        val meta = ext.meta
        out += importRow(SettingsUiImportKind.ARCHIVE, meta.id, ext.serialType(), meta.title, meta.version, flag = core(meta.id))
        meta.description?.let { out += importRow(SettingsUiImportKind.DESCRIPTION, text = it) }
        for (m in meta.maintainers) out += importRow(SettingsUiImportKind.MAINTAINER, label = m.name)
        for (c in ext.components()) out += importRow(SettingsUiImportKind.COMPONENT, id = c.id)
    }
    return out.flatMap { it.asList() }.toTypedArray()
}

/** As [importRows], from the app's own workspaces and extensions. */
internal fun importRows(context: Context, uuid: String): Array<String> {
    val cache by context.cacheManager()
    val extensions by context.extensionManager()
    val files = cache.importer.getWorkspaceByUuid(uuid)?.inputFileInfos
    return importRows(uuid, files, { Formatter.formatShortFileSize(context, it) }) { id ->
        extensions.getExtensionById(id)?.sourceRef?.isAssets == true
    }
}

private fun importRow(kind: String, extension: String = "", id: String = "", label: String = "", detail: String = "",
    text: String = "", flag: Boolean = false) = SettingsUiExtension.row(SettingsUiExtension.KIND to kind,
    SettingsUiExtension.EXTENSION to extension, SettingsUiExtension.ID to id, SettingsUiExtension.LABEL to label,
    SettingsUiExtension.DETAIL to detail, SettingsUiExtension.TEXT to text, SettingsUiExtension.FLAG to WireFlag.encode(flag))
