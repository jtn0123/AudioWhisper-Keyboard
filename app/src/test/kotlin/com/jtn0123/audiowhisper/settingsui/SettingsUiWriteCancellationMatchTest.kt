package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.SettingsRules
import com.jtn0123.audiowhisper.settings.SettingsPreference
import dev.patrickgold.florisboard.app.FlorisPreferenceModelImpl
import dev.patrickgold.florisboard.app.SettingsStoreBridge
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import dev.patrickgold.jetpref.datastore.model.PreferenceModel
import dev.patrickgold.jetpref.datastore.runtime.DataStoreWriter
import dev.patrickgold.jetpref.datastore.runtime.LoadStrategy
import dev.patrickgold.jetpref.datastore.runtime.PersistStrategy
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeoutOrNull
import java.util.concurrent.CopyOnWriteArrayList
import kotlin.test.*

/**
 * Jetpref's DataStore stops persisting for good when a write's caller is cancelled mid-write
 * ([SettingsPreference] says how). A plain Jetpref model shows the defect is real in the Jetpref
 * build we ship; the production model must come through it with its store working. If a Jetpref
 * update fixes the defect, the plain half fails and [SettingsPreference] can go.
 */
class SettingsUiWriteCancellationMatchTest {
    /** Two preferences declared as Jetpref declares them, with no production wrapper. */
    private class PlainModel : PreferenceModel() {
        val numberRow = boolean(key = "keyboard__number_row", default = false)
        val historyEnabled = boolean(key = "clipboard__history_enabled", default = false)
        override val declaredPreferenceEntries: Map<TypedKey, PreferenceData<*>> by lazy {
            listOf(numberRow, historyEnabled).associateBy { it.typedKey }
        }
    }

    /** Holds the first write once armed, so its caller can be cancelled while it is in flight. */
    private class HeldWriter : DataStoreWriter {
        @Volatile var armed = false
        val entered = CompletableDeferred<Unit>()
        val release = CompletableDeferred<Unit>()
        val writes = CopyOnWriteArrayList<String>()
        override suspend fun write(content: String) {
            if (armed && entered.complete(Unit)) release.await()
            writes += content
        }
    }

    private class Outcome(val callerLeftMidWrite: Boolean, val later: Result<Unit>, val last: String?)

    private fun cancelMidWrite(
        model: PreferenceModel, held: PreferenceData<Boolean>, later: PreferenceData<Boolean>,
        write: suspend PreferenceData<Boolean>.() -> Result<Unit>,
    ): Outcome = runBlocking {
        // Store calls run outside runBlocking's scope and are waited on, not cancelled: a write
        // that cannot be cancelled and never finishes must fail this test, not hang it.
        val outside = CoroutineScope(Dispatchers.Default)
        suspend fun bounded(block: suspend () -> Result<Unit>) = runCatching {
            checkNotNull(withTimeoutOrNull(5_000) { outside.async { block() }.await() }) { "write timed out" }.getOrThrow()
        }
        val writer = HeldWriter()
        SettingsStoreBridge.store(model).init(LoadStrategy.Disabled, PersistStrategy.UseWriter(writer)).getOrThrow()
        bounded { held.set(!held.default) }.getOrThrow() // not the default, so a reset has something to write
        writer.armed = true
        val caller = outside.launch { held.write() }
        checkNotNull(withTimeoutOrNull(5_000) { writer.entered.await() }) { "the held write never reached the writer" }
        caller.cancel()
        val left = withTimeoutOrNull(1_000) { caller.join() } != null
        writer.release.complete(Unit)
        withTimeoutOrNull(5_000) { caller.join() }
        Outcome(left, bounded { later.set(!later.get()) }, writer.writes.lastOrNull())
    }

    private fun check(write: suspend PreferenceData<Boolean>.() -> Result<Unit>) {
        val plain = PlainModel()
        val raw = cancelMidWrite(plain, plain.numberRow, plain.historyEnabled, write)
        assertTrue(raw.callerLeftMidWrite, "the plain caller is cancelled while its write is in flight")
        assertTrue(raw.later.isFailure, "the plain store no longer persists anything")

        val model = FlorisPreferenceModelImpl()
        val later = model.clipboard.historyEnabled
        val kept = cancelMidWrite(model, model.keyboard.numberRow, later, write)
        assertFalse(kept.callerLeftMidWrite, "a started write finishes before its caller is released")
        assertTrue(kept.later.isSuccess, "a later write still succeeds")
        assertContains(kept.last.orEmpty(), "${later.key};${later.get()}", message = "the later write reached the file")
    }

    @Test fun aCancelledSetLeavesTheStoreWriting() = check { set(!get()) }

    @Test fun aCancelledResetLeavesTheStoreWriting() = check { reset() }

    @Test fun everyDeclaredPreferenceFinishesItsWrites() {
        val entries = FlorisPreferenceModelImpl().declaredPreferenceEntries
        // A wrapper typed as itself is not declared at all: the store would drop it.
        assertEquals(SettingsRules.catalogue.keys, entries.values.map { it.key }.toSet())
        assertEquals(emptyList(), entries.values.filterNot { it is SettingsPreference<*> }.map { it.key })
    }
}
