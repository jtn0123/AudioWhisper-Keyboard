package dev.patrickgold.florisboard.app.settings.advanced

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.DocumentsContract
import androidx.core.app.ShareCompat
import androidx.core.content.FileProvider
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.BuildConfig
import dev.patrickgold.florisboard.cacheManager
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.lib.cache.CacheManager
import dev.patrickgold.florisboard.lib.io.FileRegistry
import dev.patrickgold.florisboard.lib.io.ZipUtils
import dev.patrickgold.jetpref.datastore.runtime.ImportStrategy
import java.io.File
import java.text.DateFormat
import java.util.Date
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import org.florisboard.lib.android.readToFile
import org.florisboard.lib.android.writeFromFile
import org.florisboard.lib.kotlin.io.readJson
import org.florisboard.lib.kotlin.io.writeJson

object Backup {
    /** A backup's `backup_metadata.json`, as every version has written it. */
    @Serializable
    data class Metadata(
        @SerialName("package")
        val packageName: String,
        val versionCode: Int,
        val versionName: String,
        val timestamp: Long,
    )
}

/** The archive steps, with no Android in them, done as Compose's Back up and Restore did them. */
internal object BackupFiles {
    /** Writes [metadata] beside the files in [input] and zips them into [output]: the archive. */
    fun pack(input: File, output: File, metadata: Backup.Metadata): File {
        File(input, BackupPolicy.metadataName).writeJson(metadata)
        return File(output, BackupPolicy.fileName(metadata.packageName, metadata.versionCode, metadata.timestamp))
            .also { ZipUtils.zip(input, it) }
    }

    /** Unpacks [archive] into [output] and says what it holds: a `RESTORE_READ` report, with each
     * group's bit in `PRESENT`, and in `DAMAGED` each clipboard file that will not decode. */
    fun inspect(archive: File, output: File, preferences: String, date: (Long) -> String): Array<String> {
        val job = SettingsUiJob.RESTORE_READ
        if (runCatching { ZipUtils.unzip(archive, output, BackupPolicy::archive) }.isFailure) {
            return done(job, SettingsUiJobResult.UNREADABLE)
        }
        val file = File(output, BackupPolicy.metadataName)
        if (!file.isFile) return done(job, SettingsUiJobResult.NOT_BACKUP)
        val metadata = runCatching { file.readJson<Backup.Metadata>() }.getOrNull()
            ?: return done(job, SettingsUiJobResult.BAD_METADATA)
        var present = 0
        var damaged = 0
        for ((group, path) in BackupPolicy.layout(preferences).withIndex()) {
            val entry = File(output, path)
            if (!entry.exists()) continue
            present = present or (1 shl group)
            if (group >= BackupLayout.CLIPBOARD_TEXT && runCatching { entry.readJson<List<ClipboardItem>>() }.isFailure) {
                damaged = damaged or (1 shl group)
            }
        }
        return done(job, SettingsUiJobResult.OK, SettingsUiDone.PACKAGE to metadata.packageName,
            SettingsUiDone.VERSION_CODE to "${metadata.versionCode}", SettingsUiDone.VERSION_NAME to metadata.versionName,
            SettingsUiDone.DATE to date(metadata.timestamp), SettingsUiDone.PRESENT to "$present", SettingsUiDone.DAMAGED to "$damaged")
    }

    /** A `DONE` report: [job] ended with [result]; fields not given are empty. */
    fun done(job: String, result: String, vararg fields: Pair<Int, String>): Array<String> {
        val given = fields.toMap()
        return Array(SettingsUiDone.SIZE) { given[it].orEmpty() }.also {
            it[SettingsUiDone.REPORT] = SettingsUiFiles.DONE
            it[SettingsUiDone.JOB] = job
            it[SettingsUiDone.RESULT] = result
        }
    }
}

/** Back up's and Restore's file work, as Rust asks for it. Backing up and restoring run on the main
 * thread as Compose ran them; copying, zipping and unzipping run on the IO dispatcher. */
internal object BackupWork {
    private const val AUTHORITY = "${BuildConfig.APPLICATION_ID}.provider.file"
    /** A backup made and waiting for a place, one a share target may still be reading, one read. */
    private var made: CacheManager.BackupAndRestoreWorkspace? = null
    private var shared: CacheManager.BackupAndRestoreWorkspace? = null
    private var read: CacheManager.BackupAndRestoreWorkspace? = null

    /** Runs the `work` packet [p]; [running] is the job already running, for `DISCARD`. */
    suspend fun run(context: Context, p: List<String>, running: String?, document: (String) -> Uri?,
        start: (Intent) -> Unit): Array<String> {
        val job = p[SettingsUiWork.JOB]
        return when (job) {
            SettingsUiJob.BACKUP_PREPARE -> prepare(context, p[SettingsUiWork.MASK].toInt())
            SettingsUiJob.BACKUP_WRITE -> write(context, document(p[SettingsUiWork.DOCUMENT]))
            SettingsUiJob.BACKUP_SHARE -> share(context, start)
            SettingsUiJob.RESTORE_READ -> read(context, document(p[SettingsUiWork.DOCUMENT]))
            SettingsUiJob.RESTORE_APPLY -> apply(context, p[SettingsUiWork.MASK].toInt(), WireFlag.decode(p[SettingsUiWork.ERASE]))
            else -> discard(running)
        }
    }

    private suspend fun prepare(context: Context, mask: Int): Array<String> {
        val job = SettingsUiJob.BACKUP_PREPARE
        made?.close()
        made = null
        val workspace = context.cacheManager().value.backupAndRestore.new()
        return try {
            BackupIO.backup(context, workspace.inputDir, mask)
            val metadata = Backup.Metadata(BuildConfig.APPLICATION_ID, BuildConfig.VERSION_CODE,
                BuildConfig.VERSION_NAME, System.currentTimeMillis())
            workspace.zipFile = withContext(Dispatchers.IO) { BackupFiles.pack(workspace.inputDir, workspace.outputDir, metadata) }
            made = workspace
            BackupFiles.done(job, SettingsUiJobResult.OK, SettingsUiDone.NAME to workspace.zipFile.name)
        } catch (e: Exception) {
            workspace.close()
            BackupFiles.done(job, SettingsUiJobResult.FAILED)
        }
    }

    private suspend fun write(context: Context, uri: Uri?): Array<String> {
        val job = SettingsUiJob.BACKUP_WRITE
        val workspace = made
        if (workspace == null || uri == null) return BackupFiles.done(job, SettingsUiJobResult.GONE)
        return try {
            withContext(Dispatchers.IO) { context.contentResolver.writeFromFile(uri, workspace.zipFile) }
            workspace.close()
            made = null
            BackupFiles.done(job, SettingsUiJobResult.OK)
        } catch (e: Exception) {
            // The picker made the document empty; a copy that failed leaves no broken backup.
            runCatching { DocumentsContract.deleteDocument(context.contentResolver, uri) }
            BackupFiles.done(job, SettingsUiJobResult.UNWRITABLE)
        }
    }

    private fun share(context: Context, start: (Intent) -> Unit): Array<String> {
        val job = SettingsUiJob.BACKUP_SHARE
        val workspace = made ?: return BackupFiles.done(job, SettingsUiJobResult.GONE)
        return try {
            val uri = FileProvider.getUriForFile(context, AUTHORITY, workspace.zipFile)
            start(ShareCompat.IntentBuilder(context).setStream(uri).setType(FileRegistry.BackupArchive.mediaType)
                .createChooserIntent().addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION))
            shared?.close()
            shared = workspace
            made = null
            BackupFiles.done(job, SettingsUiJobResult.OK)
        } catch (e: ActivityNotFoundException) {
            BackupFiles.done(job, SettingsUiJobResult.NO_APP)
        }
    }

    private suspend fun read(context: Context, uri: Uri?): Array<String> {
        val job = SettingsUiJob.RESTORE_READ
        read?.close()
        read = null
        if (uri == null) return BackupFiles.done(job, SettingsUiJobResult.GONE)
        val workspace = context.cacheManager().value.backupAndRestore.new()
        workspace.zipFile = File(workspace.inputDir, BackupPolicy.inputArchiveName)
        val report = try {
            withContext(Dispatchers.IO) {
                context.contentResolver.readToFile(uri, workspace.zipFile)
                BackupFiles.inspect(workspace.zipFile, workspace.outputDir, BackupIO.preferences) {
                    DateFormat.getDateTimeInstance().format(Date(it))
                }
            }
        } catch (e: Exception) {
            BackupFiles.done(job, SettingsUiJobResult.UNREADABLE)
        }
        if (report[SettingsUiDone.RESULT] == SettingsUiJobResult.OK) read = workspace else workspace.close()
        return report
    }

    /** Restores the read backup; one that stops part-way says how many of its plan's steps ran,
     * and stays read so it can be tried again. */
    private suspend fun apply(context: Context, mask: Int, erase: Boolean): Array<String> {
        val job = SettingsUiJob.RESTORE_APPLY
        val workspace = read ?: return BackupFiles.done(job, SettingsUiJobResult.GONE)
        var steps = 0
        return try {
            BackupIO.restore(context, workspace, mask, if (erase) ImportStrategy.Erase else ImportStrategy.Merge) { steps++ }
            workspace.close()
            read = null
            BackupFiles.done(job, SettingsUiJobResult.OK)
        } catch (e: Exception) {
            BackupFiles.done(job, SettingsUiJobResult.FAILED, SettingsUiDone.STEPS to "$steps")
        }
    }

    /** Deletes the archives no screen needs, except one [running] work uses or a share target reads. */
    private fun discard(running: String?): Array<String> {
        if (running != SettingsUiJob.BACKUP_PREPARE && running != SettingsUiJob.BACKUP_WRITE) {
            made?.close()
            made = null
        }
        if (running != SettingsUiJob.RESTORE_READ && running != SettingsUiJob.RESTORE_APPLY) {
            read?.close()
            read = null
        }
        return BackupFiles.done(SettingsUiJob.DISCARD, SettingsUiJobResult.OK)
    }
}
