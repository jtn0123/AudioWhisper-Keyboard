package com.jtn0123.audiowhisper.settingsui

import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.provider.OpenableColumns
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.result.ActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.settings.advanced.BackupFiles
import dev.patrickgold.florisboard.app.settings.advanced.BackupWork
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** The system file picker and file work for the native settings screens. Rust asks; this opens the
 * picker or starts the work and reports how it ended. Rust decides everything after. A picked
 * document is kept under a numbered handle, so its address never reaches Rust or the log. */
internal class SettingsFiles(private val activity: ComponentActivity, private val report: (Array<String>) -> Unit) {
    private val launcher = activity.activityResultRegistry.register(KEY, ActivityResultContracts.StartActivityForResult(), ::picked)
    init { settingsFileWork.attach(report) }
    /** A `pick`, `work` or `toast` effect. */
    fun execute(p: List<String>) {
        when (p[0]) {
            SettingsUiFileEffect.PICK -> try { launcher.launch(PickRequest.of(p).intent()) } catch (e: ActivityNotFoundException) {
                settingsFileWork.send(pickedReport(SettingsUiPickOutcome.NO_PICKER))
            }
            SettingsUiFileEffect.WORK -> {
                val context = activity.applicationContext
                val job = p[SettingsUiWork.JOB]
                settingsFileWork.run(job) { running ->
                    if (job in WordFileWork.JOBS) WordFileWork.run(context, p, settingsFileWork::take)
                    else if (job in ExtensionFileWork.JOBS) ExtensionFileWork.run(context, p, settingsFileWork::take)
                    else if (job in ExtensionEditorWork.JOBS) ExtensionEditorWork.run(context, p, settingsFileWork::take)
                    else BackupWork.run(context, p, running, settingsFileWork::take, activity::startActivity)
                }
            }
            SettingsUiFileEffect.TOAST -> Toast.makeText(activity, p[1], Toast.LENGTH_LONG).show()
        }
    }
    private fun picked(result: ActivityResult) {
        val data = result.data
        val uris = when {
            result.resultCode != Activity.RESULT_OK || data == null -> emptyList()
            data.clipData != null -> data.clipData!!.let { clips -> (0 until clips.itemCount).map { clips.getItemAt(it).uri } }
            else -> listOfNotNull(data.data)
        }
        val resolver = activity.applicationContext.contentResolver
        settingsFileWork.answer {
            val names = withContext(Dispatchers.IO) { uris.map { displayName(resolver, it) } }
            if (uris.isEmpty()) pickedReport(SettingsUiPickOutcome.CANCELLED)
            else pickedReport(SettingsUiPickOutcome.PICKED, uris.zip(names) { uri, name -> settingsFileWork.keep(uri) to name })
        }
    }
    private fun displayName(resolver: android.content.ContentResolver, uri: Uri): String = runCatching {
        resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { if (it.moveToFirst()) it.getString(0) else null }
    }.getOrNull().orEmpty()
    fun release() { launcher.unregister(); settingsFileWork.detach(report) }
    private companion object { const val KEY = "aw-settings-file" }
}

/** The intent androidx's CreateDocument, GetContent and GetMultipleContents build for a `pick`. */
internal data class PickRequest(val action: String, val type: String, val openable: Boolean, val title: String?,
    val types: List<String>?, val multiple: Boolean) {
    fun intent(): Intent = Intent(action).setType(type).apply {
        if (openable) addCategory(Intent.CATEGORY_OPENABLE)
        title?.let { putExtra(Intent.EXTRA_TITLE, it) }
        types?.let { putExtra(Intent.EXTRA_MIME_TYPES, it.toTypedArray()) }
        if (multiple) putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true)
    }
    companion object {
        fun of(p: List<String>): PickRequest {
            val mime = p[SettingsUiPick.MIME].split('|')
            return if (p[SettingsUiPick.MODE] == SettingsUiPickMode.CREATE) {
                PickRequest(Intent.ACTION_CREATE_DOCUMENT, mime[0], false, p[SettingsUiPick.NAME], null, false)
            } else {
                PickRequest(Intent.ACTION_GET_CONTENT, if (mime.size > 1) "*/*" else mime[0], true, null,
                    mime.takeIf { it.size > 1 }, p[SettingsUiPick.MODE] == SettingsUiPickMode.OPEN_MANY)
            }
        }
    }
}

/** A `PICKED` report: how the picker closed, then each document's handle and name. */
internal fun pickedReport(outcome: String, documents: List<Pair<String, String>> = emptyList()): Array<String> =
    SettingsUiPicked.row(SettingsUiPicked.REPORT to SettingsUiFiles.PICKED, SettingsUiPicked.OUTCOME to outcome) +
        documents.flatMap { (handle, name) -> SettingsUiDocument.row(SettingsUiDocument.HANDLE to handle, SettingsUiDocument.NAME to name).asList() }

/** Runs one job at a time, and hands each report to the screen attached last when it arrives, queueing it while none
 * is, so work and its report outlive the activity. A screen over another, as the theme editor's activity is over the
 * extension editor's, hands the reports back as it leaves. [D] is a document. */
internal class FileWorkQueue<D>(private val scope: CoroutineScope) {
    private val hosts = mutableListOf<(Array<String>) -> Unit>()
    private val queued = ArrayDeque<Array<String>>()
    private val documents = mutableMapOf<String, D>()
    private var next = 0
    /** The job running, if any. */
    var running: String? = null
        private set
    /** Delivers what waited, then `IDLE` when no work runs: Rust then knows any it waited for was lost. */
    fun attach(report: (Array<String>) -> Unit) {
        hosts.remove(report)
        hosts.add(report)
        while (queued.isNotEmpty()) report(queued.removeFirst())
        if (running == null) report(arrayOf(SettingsUiFiles.IDLE))
    }
    /** The screen [report] left; the one it covered takes the reports again, told when no work runs. */
    fun detach(report: (Array<String>) -> Unit) {
        val top = hosts.lastOrNull() === report
        hosts.remove(report)
        val uncovered = hosts.lastOrNull()
        if (top && uncovered != null && running == null) uncovered(arrayOf(SettingsUiFiles.IDLE))
    }
    fun send(packet: Array<String>) { val to = hosts.lastOrNull(); if (to != null) to(packet) else queued.addLast(packet) }
    fun keep(document: D): String = "${next++}".also { documents[it] = document }
    /** The document under [handle], which is then forgotten. */
    fun take(handle: String): D? = documents.remove(handle)
    fun answer(block: suspend () -> Array<String>) { scope.launch { send(block()) } }
    /** Runs [work], given the job already running; `DISCARD` runs beside one and reports nothing. */
    fun run(job: String, work: suspend (running: String?) -> Array<String>) {
        if (job == SettingsUiJob.DISCARD) { scope.launch { work(running) }; return }
        if (running != null) return send(BackupFiles.done(job, SettingsUiJobResult.FAILED))
        running = job
        scope.launch {
            val report = runCatching { work(null) }.getOrElse { BackupFiles.done(job, SettingsUiJobResult.FAILED) }
            running = null
            send(report)
        }
    }
}

/** The process's queue: one per process, so a recreated activity finds the work still running. */
internal val settingsFileWork by lazy { FileWorkQueue<Uri>(CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)) }
