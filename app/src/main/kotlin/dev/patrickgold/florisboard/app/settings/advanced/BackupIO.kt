package dev.patrickgold.florisboard.app.settings.advanced

import android.content.ContentUris
import android.content.Context
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.FlorisPreferenceModel
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardFileStorage
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.lib.cache.CacheManager
import dev.patrickgold.jetpref.datastore.runtime.AndroidAppDataStorage
import dev.patrickgold.jetpref.datastore.runtime.FileBasedStorage
import dev.patrickgold.jetpref.datastore.runtime.ImportStrategy
import java.io.File
import org.florisboard.lib.kotlin.io.deleteContentsRecursively
import org.florisboard.lib.kotlin.io.readJson
import org.florisboard.lib.kotlin.io.writeJson

/** Android/file effects for a complete Rust manifest or restore plan. */
internal object BackupIO {
    private val preferences get() = "${AndroidAppDataStorage.JETPREF_DIR_NAME}/${FlorisPreferenceModel.NAME}.${AndroidAppDataStorage.JETPREF_FILE_EXT}"

    suspend fun backup(context: Context, root: File, mask: Int) {
        var clipboardHistory: List<ClipboardItem>? = null
        for ((group, path) in BackupPolicy.manifest(mask, preferences).map { it[BackupManifestRow.GROUP].toInt() to it[BackupManifestRow.PATH] }) {
            val destination = File(root, path)
            when (group) {
                BackupLayout.PREFERENCES -> FlorisPreferenceStore.export(FileBasedStorage(destination.path)).getOrThrow()
                BackupLayout.KEYBOARDS, BackupLayout.THEMES -> File(context.filesDir, path.removePrefix("files/"))
                    .copyRecursively(destination)
                else -> {
                    destination.parentFile?.mkdirs()
                    val history = clipboardHistory ?: context.clipboardManager().value.currentHistory.all.also {
                        clipboardHistory = it
                    }
                    val kind = mapOf(BackupLayout.CLIPBOARD_TEXT to "TEXT", BackupLayout.CLIPBOARD_IMAGES to "IMAGE", BackupLayout.CLIPBOARD_VIDEO to "VIDEO").getValue(group)
                    val items = BackupPolicy.items(kind, history.map { it.type.name }).map { history[it] }
                    destination.writeJson(items)
                    for (index in BackupPolicy.items("IMAGE", items.map { it.type.name }) +
                        BackupPolicy.items("VIDEO", items.map { it.type.name })) {
                        val id = ContentUris.parseId(items[index].uri!!)
                        ClipboardFileStorage.getFileForId(context, id).copyTo(
                            File(root, "${BackupPolicy.mediaDirectory}/$id"))
                    }
                }
            }
        }
    }

    suspend fun restore(context: Context, workspace: CacheManager.BackupAndRestoreWorkspace, mask: Int, strategy: ImportStrategy) {
        val root = workspace.outputDir
        val present = BackupPolicy.layout(preferences).map { File(root, it).exists() }
        val plan = BackupPolicy.restore(mask, strategy == ImportStrategy.Erase, preferences, present)
        val clipboard = context.clipboardManager().value
        for ((operation, path) in plan.map { it[BackupEffectRow.EFFECT] to it[BackupEffectRow.PATH] }) {
            val source = File(root, path)
            when (operation) {
                BackupEffect.PREFERENCES_MERGE, BackupEffect.PREFERENCES_ERASE -> FlorisPreferenceStore.import(
                    if (operation == BackupEffect.PREFERENCES_ERASE) ImportStrategy.Erase else ImportStrategy.Merge,
                    FileBasedStorage(source.path),
                ).getOrThrow()
                BackupEffect.CLEAR_DIRECTORY -> File(context.filesDir, path.removePrefix("files/")).deleteContentsRecursively()
                BackupEffect.COPY_DIRECTORY -> source.copyRecursively(File(context.filesDir, path.removePrefix("files/")), overwrite = true)
                BackupEffect.CLEAR_CLIPBOARD -> {
                    clipboard.clearFullHistory()
                    ClipboardFileStorage.resetClipboardFileStorage(context)
                }
                BackupEffect.RESTORE_TEXT, BackupEffect.RESTORE_IMAGES, BackupEffect.RESTORE_VIDEO -> {
                    val items = source.readJson<List<ClipboardItem>>()
                    val kind = mapOf(BackupEffect.RESTORE_TEXT to "TEXT", BackupEffect.RESTORE_IMAGES to "IMAGE", BackupEffect.RESTORE_VIDEO to "VIDEO").getValue(operation)
                    val selected = BackupPolicy.items(kind, items.map { it.type.name }).map { items[it] }
                    for (index in BackupPolicy.items("IMAGE", selected.map { it.type.name }) +
                        BackupPolicy.items("VIDEO", selected.map { it.type.name })) {
                        val name = selected[index].uri!!.path!!.split('/').last()
                        ClipboardFileStorage.insertFileFromBackupIfNotExisting(context,
                            File(root, "${BackupPolicy.mediaDirectory}/$name"))
                    }
                    clipboard.restoreHistory(selected)
                }
                else -> error("Unknown backup effect")
            }
        }
    }
}
