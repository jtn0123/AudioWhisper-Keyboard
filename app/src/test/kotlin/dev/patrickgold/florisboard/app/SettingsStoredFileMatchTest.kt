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
        // The reference's answers also go on a golden tape (docs/phase6-kotlin-references.md).
        val tape=GoldenTape.ofReference("settings-stored-files")
        repeat(20) {seed ->
            val old=ReferenceSettingsModel()
            val baseline=SettingsStoreBridge.store(old)
            baseline.init(LoadStrategy.Disabled,PersistStrategy.Disabled).getOrThrow()
            old.keyboard.heightFactorPortrait.set(75+seed).getOrThrow()
            old.clipboard.historyEnabled.set(seed%2==0).getOrThrow()
            old.theme.dayThemeId.set(dev.patrickgold.florisboard.lib.ext.ExtensionComponentName("synthetic.theme","day-$seed")).getOrThrow()
            old.localization.subtypes.set(ReferenceSettingsCodecs.subtypes(listOf(Subtype.DEFAULT.copy(id=seed.toLong()+1),Subtype.DEFAULT.copy(id=seed.toLong()+100,
                primaryLocale=dev.patrickgold.florisboard.lib.FlorisLocale.fromTag("fr-FR"))))).getOrThrow()
            old.localization.activeSubtypeId.set(seed.toLong()+100).getOrThrow()
            old.emoji.historyData.set(EmojiHistory(emptyList(),listOf(Emoji("😀","",emptyList())))).getOrThrow()
            old.smartbar.actionArrangement.set(QuickActionArrangement(QuickAction.InsertText("synthetic-$seed"),emptyList(),emptyList())).getOrThrow()
            val before=File(directory,"$seed-before.prefs")
            baseline.export(FileBasedStorage(before.absolutePath)).getOrThrow()
            tape.answer("before $seed",before.readText())
            val model=FlorisPreferenceModelImpl()
            val upgraded=SettingsStoreBridge.store(model)
            upgraded.init(LoadStrategy.UseReader(FileBasedStorage(before.absolutePath)),PersistStrategy.Disabled).getOrThrow()
            // Loading calls historical migrations in both builds, notably action arrangement additions.
            val sameBuild=ReferenceSettingsModel()
            val original=SettingsStoreBridge.store(sameBuild)
            original.init(LoadStrategy.UseReader(FileBasedStorage(before.absolutePath)),PersistStrategy.Disabled).getOrThrow()
            for((key,pref) in sameBuild.declaredPreferenceEntries.entries.sortedBy { it.key.key }) {
                tape.answer(key.key,encode(pref))
                assertEquals(encode(pref),encode(model.declaredPreferenceEntries.getValue(key)),"stored field ${key.key}")
            }
            // Canonical stored subtype and history JSON remains byte-identical too.
            assertEquals(model.localization.subtypes.get(),SettingsCodecs.subtypes(SettingsCodecs.subtypes(model.localization.subtypes.get())))
            tape.answer("history",ReferenceSettingsCodecs.history(model.emoji.historyData.get()))
            assertEquals(ReferenceSettingsCodecs.history(model.emoji.historyData.get()),SettingsCodecs.history(model.emoji.historyData.get()))
            val expected=File(directory,"$seed-reference.prefs");val actual=File(directory,"$seed-native.prefs")
            original.export(FileBasedStorage(expected.absolutePath)).getOrThrow()
            upgraded.export(FileBasedStorage(actual.absolutePath)).getOrThrow()
            tape.answer("loaded $seed",expected.readText())
            assertContentEquals(expected.readBytes(),actual.readBytes(),"complete Jetpref stored file")
            val round=FlorisPreferenceModelImpl();val roundStore=SettingsStoreBridge.store(round)
            roundStore.init(LoadStrategy.UseReader(FileBasedStorage(actual.absolutePath)),PersistStrategy.Disabled).getOrThrow()
            val again=File(directory,"$seed-roundtrip.prefs")
            roundStore.export(FileBasedStorage(again.absolutePath)).getOrThrow()
            assertContentEquals(actual.readBytes(),again.readBytes(),"native reload and export")
        }
        tape.finish()
    }
}
