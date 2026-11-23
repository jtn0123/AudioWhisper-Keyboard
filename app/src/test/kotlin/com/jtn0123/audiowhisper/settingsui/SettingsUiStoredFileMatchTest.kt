package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import dev.patrickgold.florisboard.app.FlorisPreferenceModelImpl
import dev.patrickgold.florisboard.app.SettingsStoreBridge
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import dev.patrickgold.jetpref.datastore.model.PreferenceModel
import dev.patrickgold.jetpref.datastore.runtime.*
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.*
import java.io.File
import kotlin.test.*

/** Host datastore proof; this does not replace the required run-as/UI device comparison. */
class SettingsUiStoredFileMatchTest {
    private fun <T : Any> raw(p: PreferenceData<T>) = p.serializer.serialize(p.get()).orEmpty()
    private suspend fun <T : Any> set(p: PreferenceData<T>, value: String) = p.set(requireNotNull(p.serializer.deserialize(value))).getOrThrow()
    private fun pref(model: PreferenceModel, key: String) = model.declaredPreferenceEntries.values.single { it.key == key }
    @Test fun nativeWidgetPlansProduceIdenticalRealJetprefExportBytes() = runBlocking {
        val directory = File("../build/stage4-settings/host-stored").apply { mkdirs() }
        // The files the Compose screens' writes left, as the same taps leave them through Rust's.
        val tape = GoldenTape("settings-ui-stored-files")
        repeat(20) { seed ->
            val current = FlorisPreferenceModelImpl()
            val newStore = SettingsStoreBridge.store(current)
            newStore.init(LoadStrategy.Disabled, PersistStrategy.Disabled).getOrThrow()
            current.keyboard.fontSizeMultiplierLandscape.set(75 + seed).getOrThrow()
            KeyboardRenderer(emptyArray()).use { n ->
                n.settings(SettingsUiOp.PREFERENCES, current.declaredPreferenceEntries.values.flatMap { listOf(it.key, raw(it)) }.toTypedArray())
                n.settings(SettingsUiOp.CONFIG, arrayOf("1080","8192","1","1","4294967295","4278190080","4283782485","4281558681","4291611852","4293848814","true","true"))
                n.settings(SettingsUiOp.NAVIGATE, arrayOf("settings/keyboard"))
                suspend fun call(op: SettingsUiOp, vararg args: String) {
                    val reply = Json.parseToJsonElement(n.settings(op, arrayOf(*args))[0]).jsonObject
                    for (effect in reply.getValue("effects").jsonArray) {
                        val p = effect.jsonArray.map { it.jsonPrimitive.content }
                        if (p[0] == "write") set(pref(current, p[1]), p[2])
                        else if (p[0] == "reset") pref(current, p[1]).reset().getOrThrow()
                        else error("unexpected platform effect in fixture")
                    }
                }
                call(SettingsUiOp.ACTIVATE,"1001")
                call(SettingsUiOp.ACTIVATE,"1006");call(SettingsUiOp.ACTIVATE,"3000");call(SettingsUiOp.ACTIVATE,"7")
                call(SettingsUiOp.ACTIVATE,"1008");call(SettingsUiOp.SLIDER,"4000","50");call(SettingsUiOp.ACTIVATE,"6") // cancellation writes nothing
                call(SettingsUiOp.ACTIVATE,"1008");call(SettingsUiOp.SLIDER,"4000","50");call(SettingsUiOp.ACTIVATE,"7")
                call(SettingsUiOp.ACTIVATE,"1006");call(SettingsUiOp.ACTIVATE,"5")
                call(SettingsUiOp.ACTIVATE,"1011");call(SettingsUiOp.ACTIVATE,"8");call(SettingsUiOp.ACTIVATE,"3000");call(SettingsUiOp.ACTIVATE,"7")
                call(SettingsUiOp.ACTIVATE,"1019")
                val actual=File(directory,"$seed-rust.jetpref")
                newStore.export(FileBasedStorage(actual.absolutePath)).getOrThrow()
                tape.answer("seed $seed", actual.readText())
            }
        }
        tape.finish()
    }
}
