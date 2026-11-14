package dev.patrickgold.florisboard.app.settings.advanced

import android.content.ContentUris
import android.content.Context
import com.jtn0123.audiowhisper.rust.BackupPolicy
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
        for ((group, path) in BackupPolicy.manifest(mask, preferences)) {
            val destination = File(root, path)
            when (group) {
                "0" -> FlorisPreferenceStore.export(FileBasedStorage(destination.path)).getOrThrow()
                "1", "2" -> File(context.filesDir, path.removePrefix("files/"))
                    .copyRecursively(destination)
                else -> {
                    destination.parentFile?.mkdirs()
                    val history = clipboardHistory ?: context.clipboardManager().value.currentHistory.all.also {
                        clipboardHistory = it
                    }
                    val kind = mapOf("3" to "TEXT", "4" to "IMAGE", "5" to "VIDEO").getValue(group)
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
        for ((operation, path) in plan) {
            val source = File(root, path)
            when (operation) {
                "preferences-merge", "preferences-erase" -> FlorisPreferenceStore.import(
                    if (operation == "preferences-erase") ImportStrategy.Erase else ImportStrategy.Merge,
                    FileBasedStorage(source.path),
                ).getOrThrow()
                "clear-directory" -> File(context.filesDir, path.removePrefix("files/")).deleteContentsRecursively()
                "copy-directory" -> source.copyRecursively(File(context.filesDir, path.removePrefix("files/")), overwrite = true)
                "clear-clipboard" -> {
                    clipboard.clearFullHistory()
                    ClipboardFileStorage.resetClipboardFileStorage(context)
                }
                "text", "image", "video" -> {
                    val items = source.readJson<List<ClipboardItem>>()
                    val kind = mapOf("text" to "TEXT", "image" to "IMAGE", "video" to "VIDEO").getValue(operation)
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
