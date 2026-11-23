package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.settings.SettingsCodecs
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.media.emoji.Emoji
import dev.patrickgold.florisboard.ime.media.emoji.EmojiHistory
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionArrangement
import dev.patrickgold.jetpref.datastore.runtime.*
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import kotlinx.coroutines.runBlocking
import java.io.File
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals

class SettingsStoredFileMatchTest {
    private fun <T : Any> encode(p: PreferenceData<T>) = p.serializer.serialize(p.get())
    @Test fun currentBuildDatastoreFilesRoundTripByteForByte() = runBlocking {
        val directory=File("../build/stage2-settings/stored-files").apply {mkdirs()}
        val tape=GoldenTape("settings-stored-files")
        repeat(20) {seed ->
            val written=FlorisPreferenceModelImpl()
            val baseline=SettingsStoreBridge.store(written)
            baseline.init(LoadStrategy.Disabled,PersistStrategy.Disabled).getOrThrow()
            written.keyboard.heightFactorPortrait.set(75+seed).getOrThrow()
            written.clipboard.historyEnabled.set(seed%2==0).getOrThrow()
            written.theme.dayThemeId.set(dev.patrickgold.florisboard.lib.ext.ExtensionComponentName("synthetic.theme","day-$seed")).getOrThrow()
            written.localization.subtypes.set(SettingsCodecs.subtypes(listOf(Subtype.DEFAULT.copy(id=seed.toLong()+1),Subtype.DEFAULT.copy(id=seed.toLong()+100,
                primaryLocale=dev.patrickgold.florisboard.lib.FlorisLocale.fromTag("fr-FR"))))).getOrThrow()
            written.localization.activeSubtypeId.set(seed.toLong()+100).getOrThrow()
            written.emoji.historyData.set(EmojiHistory(emptyList(),listOf(Emoji("😀","",emptyList())))).getOrThrow()
            written.smartbar.actionArrangement.set(QuickActionArrangement(QuickAction.InsertText("synthetic-$seed"),emptyList(),emptyList())).getOrThrow()
            val before=File(directory,"$seed-before.prefs")
            baseline.export(FileBasedStorage(before.absolutePath)).getOrThrow()
            tape.answer("before $seed",before.readText())
            // Loading calls the historical migrations, notably action arrangement additions.
            val model=FlorisPreferenceModelImpl()
            val upgraded=SettingsStoreBridge.store(model)
            upgraded.init(LoadStrategy.UseReader(FileBasedStorage(before.absolutePath)),PersistStrategy.Disabled).getOrThrow()
            for((key,pref) in model.declaredPreferenceEntries.entries.sortedBy { it.key.key }) tape.answer(key.key,encode(pref))
            // Canonical stored subtype JSON remains byte-identical.
            assertEquals(model.localization.subtypes.get(),SettingsCodecs.subtypes(SettingsCodecs.subtypes(model.localization.subtypes.get())))
            tape.answer("history",SettingsCodecs.history(model.emoji.historyData.get()))
            val actual=File(directory,"$seed-native.prefs")
            upgraded.export(FileBasedStorage(actual.absolutePath)).getOrThrow()
            tape.answer("loaded $seed",actual.readText())
            val round=FlorisPreferenceModelImpl();val roundStore=SettingsStoreBridge.store(round)
            roundStore.init(LoadStrategy.UseReader(FileBasedStorage(actual.absolutePath)),PersistStrategy.Disabled).getOrThrow()
            val again=File(directory,"$seed-roundtrip.prefs")
            roundStore.export(FileBasedStorage(again.absolutePath)).getOrThrow()
            assertContentEquals(actual.readBytes(),again.readBytes(),"native reload and export")
        }
        tape.finish()
    }
}
