package com.jtn0123.audiowhisper.settingsui

import android.content.Intent
import com.jtn0123.audiowhisper.rust.*
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlin.test.*

/** The file picker adapter's own logic: the intent each `pick` asks for, the `PICKED` report, and
 * the queue that runs one job at a time and holds reports for whichever screen is attached. */
class SettingsFilesTest {
    private fun pick(how: String, mime: String, name: String = "") =
        SettingsUiPick.row(SettingsUiPick.EFFECT to SettingsUiFileEffect.PICK, SettingsUiPick.MODE to how,
            SettingsUiPick.MIME to mime, SettingsUiPick.NAME to name).asList()

    @Test fun eachPickAsksForTheIntentAndroidxBuilt() {
        // CreateDocument("application/zip").launch(name), as BackupScreen launched it.
        assertEquals(PickRequest(Intent.ACTION_CREATE_DOCUMENT, "application/zip", false, "b.zip", null, false),
            PickRequest.of(pick(SettingsUiPickMode.CREATE, "application/zip", "b.zip")))
        // GetContent().launch("*/*"), as RestoreScreen launched it.
        assertEquals(PickRequest(Intent.ACTION_GET_CONTENT, "*/*", true, null, null, false),
            PickRequest.of(pick(SettingsUiPickMode.OPEN, "*/*")))
        // Several types, and several documents, for dictionary and extension import.
        assertEquals(PickRequest(Intent.ACTION_GET_CONTENT, "*/*", true, null, listOf("application/zip", "text/plain"), true),
            PickRequest.of(pick(SettingsUiPickMode.OPEN_MANY, "application/zip|text/plain")))
        assertEquals(PickRequest(Intent.ACTION_GET_CONTENT, "text/plain", true, null, null, false),
            PickRequest.of(pick(SettingsUiPickMode.OPEN, "text/plain")))
    }

    @Test fun aPickedReportCarriesHandlesAndNamesNeverAddresses() {
        assertEquals(listOf(SettingsUiFiles.PICKED, SettingsUiPickOutcome.CANCELLED),
            pickedReport(SettingsUiPickOutcome.CANCELLED).asList())
        val report = pickedReport(SettingsUiPickOutcome.PICKED, listOf("0" to "a.zip", "1" to ""))
        assertEquals(SettingsUiPicked.SIZE + 2 * SettingsUiDocument.SIZE, report.size)
        assertEquals(listOf(SettingsUiFiles.PICKED, SettingsUiPickOutcome.PICKED, "0", "a.zip", "1", ""), report.asList())
    }

    private class Host {
        val got = mutableListOf<List<String>>()
        val report: (Array<String>) -> Unit = { got += it.asList() }
    }
    private fun queue() = FileWorkQueue<String>(CoroutineScope(Dispatchers.Unconfined))
    private fun done(job: String, result: String = SettingsUiJobResult.OK) =
        dev.patrickgold.florisboard.app.settings.advanced.BackupFiles.done(job, result).asList()
    private val idle = listOf(SettingsUiFiles.IDLE)

    @Test fun reportsWaitForAScreenAndIdleSaysNothingRuns() {
        val q = queue()
        q.send(arrayOf("first"))
        q.answer { arrayOf("second") }
        val host = Host()
        q.attach(host.report)
        assertEquals(listOf(listOf("first"), listOf("second"), idle), host.got)
        q.send(arrayOf("third"))
        assertEquals(listOf("third"), host.got.last())
        // A screen that left already does not take the next one's reports away.
        val next = Host()
        q.attach(next.report)
        q.detach(host.report)
        q.send(arrayOf("fourth"))
        assertEquals(listOf("fourth"), next.got.last())
        q.detach(next.report)
        q.send(arrayOf("fifth"))
        assertEquals(listOf("fourth"), next.got.last())
    }

    @Test fun aCoveredScreenTakesTheReportsBackWhenTheOneOverItLeaves() {
        val q = queue()
        val under = Host()
        q.attach(under.report)
        // The theme editor's activity opens over the extension editor's, and closes.
        val over = Host()
        q.attach(over.report)
        under.got.clear()
        q.detach(over.report)
        assertEquals(listOf(idle), under.got)
        // Save's report, sent once it is back, reaches it: it used to wait for a screen forever.
        q.send(arrayOf("saved"))
        assertEquals(listOf("saved"), under.got.last())
        // A screen under another that leaves first changes nothing for the one on top.
        val top = Host()
        q.attach(top.report)
        under.got.clear(); top.got.clear()
        q.detach(under.report)
        assertEquals(emptyList(), top.got)
        q.send(arrayOf("next"))
        assertEquals(listOf(listOf("next")), top.got)
        assertEquals(emptyList(), under.got)
    }

    @Test fun aCoveredScreenIsNotToldIdleWhileWorkRuns() {
        val q = queue()
        val under = Host()
        q.attach(under.report)
        val over = Host()
        q.attach(over.report)
        under.got.clear()
        val gate = CompletableDeferred<Unit>()
        q.run(SettingsUiJob.EXT_SAVE) { gate.await(); done(SettingsUiJob.EXT_SAVE).toTypedArray() }
        q.detach(over.report)
        assertEquals(emptyList(), under.got)
        gate.complete(Unit)
        assertEquals(listOf(done(SettingsUiJob.EXT_SAVE)), under.got)
    }

    @Test fun oneJobRunsAtATimeAndItsReportOutlivesTheScreen() {
        val q = queue()
        val host = Host()
        q.attach(host.report)
        host.got.clear()
        val gate = CompletableDeferred<Unit>()
        var discardSaw: String? = "never ran"
        q.run(SettingsUiJob.RESTORE_APPLY) { gate.await(); done(SettingsUiJob.RESTORE_APPLY).toTypedArray() }
        assertEquals(SettingsUiJob.RESTORE_APPLY, q.running)
        // A second job is refused at once; the first runs on.
        q.run(SettingsUiJob.RESTORE_READ) { fail("ran beside another job") }
        assertEquals(listOf(done(SettingsUiJob.RESTORE_READ, SettingsUiJobResult.FAILED)), host.got)
        // Discarding runs beside it, knows what runs, and reports nothing.
        q.run(SettingsUiJob.DISCARD) { running -> discardSaw = running; done(SettingsUiJob.DISCARD).toTypedArray() }
        assertEquals(SettingsUiJob.RESTORE_APPLY, discardSaw)
        assertEquals(1, host.got.size)
        // The activity goes; a new one attaches while the work runs: no IDLE, then the report.
        q.detach(host.report)
        val next = Host()
        q.attach(next.report)
        assertEquals(emptyList(), next.got)
        gate.complete(Unit)
        assertEquals(listOf(done(SettingsUiJob.RESTORE_APPLY)), next.got)
        assertNull(q.running)
    }

    @Test fun workThatThrowsReportsFailedAndFreesTheQueue() {
        val q = queue()
        val host = Host()
        q.attach(host.report)
        host.got.clear()
        q.run(SettingsUiJob.BACKUP_PREPARE) { error("disk full") }
        assertEquals(listOf(done(SettingsUiJob.BACKUP_PREPARE, SettingsUiJobResult.FAILED)), host.got)
        assertNull(q.running)
    }

    @Test fun aHandleGivesItsDocumentOnce() {
        val q = queue()
        val first = q.keep("content://one")
        val second = q.keep("content://two")
        assertNotEquals(first, second)
        assertEquals("content://two", q.take(second))
        assertNull(q.take(second))
        assertEquals("content://one", q.take(first))
        assertNull(q.take("99"))
    }
}
