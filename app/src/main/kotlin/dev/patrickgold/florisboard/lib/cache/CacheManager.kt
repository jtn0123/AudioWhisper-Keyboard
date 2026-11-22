/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package dev.patrickgold.florisboard.lib.cache

import android.content.ContentResolver
import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionEditor
import dev.patrickgold.florisboard.lib.ext.Extension
import dev.patrickgold.florisboard.lib.ext.ExtensionDefaults
import dev.patrickgold.florisboard.lib.ext.ExtensionEditor
import dev.patrickgold.florisboard.lib.ext.ExtensionCodec
import dev.patrickgold.florisboard.lib.io.FileRegistry
import dev.patrickgold.florisboard.lib.io.ZipUtils
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.florisboard.lib.android.query
import org.florisboard.lib.android.readToFile
import org.florisboard.lib.kotlin.io.FsDir
import org.florisboard.lib.kotlin.io.FsFile
import org.florisboard.lib.kotlin.io.subDir
import org.florisboard.lib.kotlin.io.subFile
import java.io.Closeable
import java.io.File
import java.util.UUID

class CacheManager(context: Context) {
    companion object {
        private const val InputDirName = "input"
        private const val OutputDirName = "output"

        private const val ImporterDirName = "importer"
        private const val ExporterDirName = "exporter"
        private const val EditorDirName = "editor"
        private const val BackupAndRestoreDirName = "backup-and-restore"

        const val LoadedDirName = "loaded"
    }

    private val appContext by context.appContext()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    val importer = WorkspacesContainer(ImporterDirName) { ImporterWorkspace(it) }
    val exporter = WorkspacesContainer(ExporterDirName) { ExporterWorkspace(it) }
    /** Kept where Android does not clear it, unlike the cache, so an edit outlives the process (stage4-extensions.md,
     * part 3); the app's start wipes only the cache. */
    val themeExtEditor = WorkspacesContainer(EditorDirName, appContext.noBackupFilesDir) { ExtEditorWorkspace<ThemeExtensionEditor>(it) }
    val backupAndRestore = WorkspacesContainer(BackupAndRestoreDirName) { BackupAndRestoreWorkspace(it) }

    /** Reads [uriList] into a new import workspace, which can be found by its id once this returns. A read that fails
     * deletes what it read. */
    suspend fun readFromUriIntoCache(uriList: List<Uri>): ImporterWorkspace {
        val contentResolver = appContext.contentResolver ?: error("Content resolver is null.")
        val workspace = ImporterWorkspace(uuid = UUID.randomUUID().toString()).also { it.mkdirs() }
        workspace.inputFileInfos = runCatching { readInto(workspace, contentResolver, uriList) }
            .onFailure { workspace.close() }.getOrThrow()
        importer.add(workspace).join()
        return workspace
    }

    private fun readInto(workspace: ImporterWorkspace, contentResolver: ContentResolver, uriList: List<Uri>) =
        buildList {
            for (uri in uriList) {
                val info = contentResolver.query(uri)?.use { cursor ->
                    val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
                    cursor.moveToFirst()
                    val file = workspace.inputDir.subFile(cursor.getString(nameIndex))
                    contentResolver.readToFile(uri, file)
                    val ext = runCatching {
                        val extWorkingDir = workspace.outputDir.subDir(file.nameWithoutExtension)
                        ZipUtils.unzip(srcFile = file, dstDir = extWorkingDir)
                        val extJsonFile = extWorkingDir.subFile(ExtensionDefaults.MANIFEST_FILE_NAME)
                        ExtensionCodec.decode(extJsonFile.readText()).also { it.workingDir = extWorkingDir }
                    }
                    FileInfo(
                        file = file,
                        mediaType = FileRegistry.guessMediaType(file, contentResolver.getType(uri)),
                        size = cursor.getLong(sizeIndex),
                        ext = ext.getOrNull(),
                    )
                } ?: error("Unable to fetch info about one or more resources to be imported.")
                add(info)
            }
        }

    open inner class WorkspacesContainer<T : Workspace> internal constructor(
        val dirName: String,
        root: FsDir = appContext.cacheDir,
        val factory: (uuid: String) -> T,
    ) {
        private val workspacesGuard = Mutex(locked = false)
        private val workspaces = mutableListOf<T>()

        val dir: FsDir = root.subDir(dirName)

        /** A new workspace, which [getWorkspaceByUuid] finds as soon as this returns. */
        fun new(uuid: String = UUID.randomUUID().toString()): T {
            return factory(uuid).also { it.mkdirs(); runBlocking { add(it).join() } }
        }

        /** The workspace [uuid] whose folder outlived the process that made it, or null when it is gone. */
        fun adopt(uuid: String): T? {
            getWorkspaceByUuid(uuid)?.let { return it }
            if (uuid.isEmpty() || !dir.subDir(uuid).isDirectory) return null
            return factory(uuid).also { runBlocking { add(it).join() } }
        }

        /** Deletes every workspace but [keep], with the folders a lost process left behind. */
        fun deleteOthers(keep: String) {
            for (folder in dir.listFiles().orEmpty()) {
                if (folder.name == keep) continue
                getWorkspaceByUuid(folder.name)?.close()
                folder.deleteRecursively()
            }
        }

        internal fun add(workspace: T) = scope.launch {
            workspacesGuard.withLock {
                workspaces.add(workspace)
            }
        }

        internal fun remove(workspace: Workspace) = scope.launch {
            workspacesGuard.withLock {
                workspaces.removeAll { it === workspace }
            }
        }

        fun getWorkspaceByUuid(uuid: String) = runBlocking { getWorkspaceByUuidAsync(uuid).await() }

        fun getWorkspaceByUuidAsync(uuid: String): Deferred<T?> = scope.async {
            workspacesGuard.withLock {
                workspaces.find { it.uuid == uuid }
            }
        }
    }

    abstract inner class Workspace(val uuid: String) : Closeable {
        abstract val dir: FsDir

        open fun mkdirs() {
            dir.mkdirs()
        }

        fun isOpen() = dir.exists()

        fun isClosed() = !dir.exists()

        override fun close() {
            dir.deleteRecursively()
        }
    }

    inner class ImporterWorkspace(uuid: String) : Workspace(uuid) {
        override val dir: FsDir = importer.dir.subDir(uuid)

        val inputDir: FsDir = dir.subDir(InputDirName)
        val outputDir: FsDir = dir.subDir(OutputDirName)

        var inputFileInfos = emptyList<FileInfo>()

        override fun mkdirs() {
            super.mkdirs()
            inputDir.mkdirs()
            outputDir.mkdirs()
        }

        override fun close() {
            super.close()
            importer.remove(this)
        }
    }

    inner class ExporterWorkspace(uuid: String) : Workspace(uuid) {
        override val dir: FsDir = exporter.dir.subDir(uuid)
    }

    inner class ExtEditorWorkspace<T : ExtensionEditor>(uuid: String) : Workspace(uuid) {
        override val dir: FsDir = themeExtEditor.dir.subDir(uuid)

        val extDir: FsDir = dir.subDir("ext")
        val saverDir: FsDir = dir.subDir("saver")

        /** The draft the theme editor reads and saves into: Rust's copy, as `extEditorSync` last sent it. */
        @Volatile var editor: T? = null

        override fun mkdirs() {
            super.mkdirs()
            extDir.mkdirs()
            saverDir.mkdirs()
        }

        override fun close() {
            super.close()
            themeExtEditor.remove(this)
        }

        /** Runs [block] on the draft, which must be there. */
        inline fun <R> update(block: T.() -> R): R = block(checkNotNull(editor))
    }

    inner class BackupAndRestoreWorkspace(uuid: String) : Workspace(uuid) {
        override val dir: FsDir = backupAndRestore.dir.subDir(uuid)

        val inputDir: FsDir = dir.subDir(InputDirName)
        val outputDir: FsDir = dir.subDir(OutputDirName)

        lateinit var zipFile: FsFile

        override fun mkdirs() {
            super.mkdirs()
            inputDir.mkdirs()
            outputDir.mkdirs()
        }

        override fun close() {
            super.close()
            backupAndRestore.remove(this)
        }
    }

    data class FileInfo(
        val file: FsFile,
        val mediaType: String?,
        val size: Long,
        val ext: Extension?,
    )
}
