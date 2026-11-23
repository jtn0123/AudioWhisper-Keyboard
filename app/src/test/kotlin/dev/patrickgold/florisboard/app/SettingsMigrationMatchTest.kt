package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.settings.SettingsCodecs
import dev.patrickgold.florisboard.ime.smartbar.quickaction.*
import dev.patrickgold.florisboard.ime.text.keyboard.*
import dev.patrickgold.florisboard.ime.popup.PopupSet
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class SettingsMigrationMatchTest {
    private val historicalFamilies = listOf(
        listOf("media__emoji_recently_used","media__emoji_recently_used_max_size"),
        listOf("advanced__settings_theme","advanced__accent_color","advanced__settings_language","other__settings_language",
            "advanced__show_app_icon","other__show_app_icon","advanced__incognito_mode","advanced__force_incognito_mode_from_dynamic",
            "other__rust_settings_prototype"),
        listOf("suggestion__clipboard_content_enabled","suggestion__clipboard_content_timeout","keyboard__one_handed_mode","smartbar__action_arrangement"),
        listOf("theme__editor_display_colors_as","clipboard__sync_to_floris","clipboard__sync_to_system",
            "clipboard__num_history_grid_columns_portrait","clipboard__num_history_grid_columns_landscape","clipboard__clean_up_old","clipboard__clean_up_after",
            "clipboard__auto_clean_sensitive","clipboard__auto_clean_sensitive_after","clipboard__limit_history_size","clipboard__max_history_size",
            "clipboard__clear_primary_clip_deletes_last_item"),
        listOf("devtools__enabled","devtools__unknown","future__key","suggestion__enabled","localization__subtypes"),
    )
    /** An outcome as the golden tapes record it: the migrated entry's four fields, or a refusal. */
    private fun answer(outcome: Result<Array<String>>) = outcome.fold({ "migrated ${it.toList()}" }, { "refused" })
    private fun check(tape: GoldenTape,type: String,key: String,raw: String) =
        tape.record(answer(runCatching {SettingsStoreBridge.snapshot(migrateAppPreference(SettingsStoreBridge.entry(type,key,raw)))}))
    @Test fun everyHistoricalKeyFamilyCorruptionAndUnknownEntriesMatch() {
        val random=Random(1710)
        var states=0
        val tape=GoldenTape("settings-migration-families")
        for(family in historicalFamilies) repeat(200) {
            // Each synthetic old stored state contains every key of that family and unknown data.
            for(key in family+listOf("future__unknown","devtools__future")) {
                val raw=listOf("true","false","OFF","END","RGBA","HEX","",";","😀;é;","{broken",random.nextInt().toString(),"synthetic\ud800").random(random)
                check(tape,listOf("b","s","i","l","f").random(random),key,raw)
            }
            states++
        }
        tape.finish()
        assertEquals(1000,states)
    }
    @Test fun fullArrangementMigrationReplacementAdditionAndIdentityDedupMatch() {
        val random=Random(1711)
        val actions=listOf(QuickAction.InsertKey(TextKeyData.COMPACT_LAYOUT_TO_RIGHT),QuickAction.InsertKey(TextKeyData.LANGUAGE_SWITCH),
            QuickAction.InsertKey(TextKeyData.FORWARD_DELETE),QuickAction.InsertKey(TextKeyData.IME_HIDE_UI),
            QuickAction.InsertKey(AutoTextKeyData(code=65)),QuickAction.InsertKey(MultiTextKeyData(codePoints=intArrayOf(65))),
            QuickAction.InsertKey(TextKeyData(code=65,popup=PopupSet(TextKeyData.SPACE))),QuickAction.InsertText("synthetic😀\ud800"))
        val tape=GoldenTape("settings-migration-arrangements")
        repeat(2000) {
            val arrangement=QuickActionArrangement(if(random.nextBoolean()) actions.random(random) else null,
                List(random.nextInt(15)) {actions.random(random)},List(random.nextInt(15)) {actions.random(random)})
            check(tape,"s","smartbar__action_arrangement",SettingsCodecs.arrangement(arrangement))
        }
        tape.finish()
    }
}
