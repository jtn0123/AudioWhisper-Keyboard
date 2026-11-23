package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.settings.SettingsCodecs
import dev.patrickgold.florisboard.ime.core.*
import dev.patrickgold.florisboard.ime.keyboard.*
import dev.patrickgold.florisboard.ime.text.keyboard.*
import dev.patrickgold.florisboard.ime.media.emoji.*
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.smartbar.quickaction.*
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.lib.ext.ExtensionJsonConfig
import kotlinx.serialization.json.Json
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class SettingsCodecMatchTest {
    private fun text(random: Random) = buildString {repeat(random.nextInt(0,24)) {append(random.nextInt(65536).toChar())}}
    /** A history as the codecs compare it: emoji are equal by their value alone. */
    private fun values(h: EmojiHistory) = "pinned ${h.pinned.map { it.value }} recent ${h.recent.map { it.value }}"
    /** The canonical bytes [codec] gives for [raw], or its refusal, on [tape]. */
    private fun check(tape: GoldenTape, raw: String, codec: (String)->String) =
        tape.record(runCatching {codec(raw)}.fold({ "ok $it" }, { "refused" }))
    private fun subtypes(raw: String) = SettingsCodecs.subtypes(SettingsCodecs.subtypes(raw))
    private fun arrangement(raw: String) = SettingsCodecs.arrangement(SettingsCodecs.arrangement(raw))
    @Test fun subtypeStoredFormatAndBundledPresetsMatch() {
        val random=Random(1701)
        val tape=GoldenTape("settings-codec-subtypes")
        val presets=java.io.File("src/main/assets").walkTopDown().filter {it.isFile && it.name.endsWith(".json") && it.readText().contains("\"preferred\"")}
            .sortedBy { it.invariantSeparatorsPath }.toList()
        assertEquals(true,presets.isNotEmpty())
        // Real bundled subtype presets use the same nested layout/provider/component defaults.
        for(file in presets) {
            val node=Json.parseToJsonElement(file.readText())
            fun walk(v:kotlinx.serialization.json.JsonElement) {
                when(v) {
                    is kotlinx.serialization.json.JsonArray -> v.forEach(::walk)
                    is kotlinx.serialization.json.JsonObject -> {
                        if("preferred" in v && "languageTag" in v) {
                            val preset=ExtensionJsonConfig.decodeFromString<SubtypePreset>(v.toString())
                            tape.record(SettingsCodecs.subtypes(listOf(preset.toSubtype())))
                        } else v.values.forEach(::walk)
                    }
                    else -> Unit
                }
            }
            walk(node)
        }
        repeat(2000) {
            val s=Subtype.DEFAULT.copy(id=random.nextLong(),primaryLocale=FlorisLocale.fromTag(listOf("en-US","fr-FR","de","vi-VN","xx-ZZ").random(random)),
                secondaryLocales=List(random.nextInt(4)) {FlorisLocale.fromTag(listOf("es","en-GB","pt-BR").random(random))},
                composer=ExtensionComponentName("addon.synthetic",text(random)),
                layoutMap=Subtype.DEFAULT.layoutMap.copy(characters=ExtensionComponentName("addon.synthetic",text(random))))
            val old=SettingsCodecs.subtypes(listOf(s))
            tape.record(old)
            check(tape,old,::subtypes)
            check(tape,old.replace("\"composer\":","\"unknown\":[nonsense,{}],\"composer\":"),::subtypes)
        }
        for(raw in listOf("[]","[{}]","null","[","[null]","[1]","[{},]")) check(tape,raw,::subtypes)
        assertEquals(emptyList(),SettingsCodecs.subtypes(" \t\n"))
        // The nested defaults, and the default subtype's stored form.
        tape.record(Subtype.DEFAULT.layoutMap.toString())
        tape.record(SettingsCodecs.subtype(Subtype.DEFAULT))
        val minimum="[{\"id\":1,\"primaryLocale\":\"en-US\",\"secondaryLocales\":[],\"composer\":\"a:b\",\"currencySet\":\"a:b\",\"popupMapping\":\"a:b\",\"layoutMap\":{}}]"
        check(tape,minimum,::subtypes)
        for(first in listOf("2", "null", "\"bad\"", "{}")) check(tape,minimum.replace("\"id\":1", "\"id\":$first,\"id\":1"),::subtypes)
        tape.finish()
    }
    @Test fun arrangementsAllKeyVariantsPopupsAndCorruptInputsMatch() {
        val random=Random(1702)
        val tape=GoldenTape("settings-codec-arrangements")
        val keys=listOf<KeyData>(TextKeyData(),TextKeyData.DELETE,AutoTextKeyData(code=65),MultiTextKeyData(codePoints=intArrayOf(0x1f600,65)),
            TextKeyData(code=65,popup=PopupSet(CaseSelector(TextKeyData(code=97),TextKeyData(code=65)),listOf(
                ShiftStateSelector(shifted=TextKeyData.SPACE),VariationSelector(normal=TextKeyData.DELETE),
                LayoutDirectionSelector(TextKeyData.SPACE,TextKeyData.DELETE),CharWidthSelector(null,TextKeyData.SPACE)))))
        repeat(2000) {
            val actions=List(random.nextInt(0,12)) {if(random.nextBoolean()) QuickAction.InsertKey(keys.random(random)) else QuickAction.InsertText(text(random))}
            val a=QuickActionArrangement(actions.firstOrNull(),actions,actions.reversed())
            val old=SettingsCodecs.arrangement(a)
            tape.record(old)
            check(tape,old,::arrangement)
        }
        val raw=SettingsCodecs.arrangement(QuickActionArrangement.Default)
        tape.record(raw)
        val cases=listOf(raw,raw.replace("insert_key","future_action"),raw.replace("text_key","future_key"),raw.replace("\"stickyAction\":","\"future\":{invalid},\"stickyAction\":"),
            raw.replace("\"code\":-233","\"code\":\"-233\""),raw.dropLast(1),"{}","null","[]",raw.replace("\"code\":-233","\"code\":2147483648"),
            raw.replace("\"type\":\"system_gui\"","\"type\":\"system_guı\""),
            raw.replace("\"code\":-233","\"code\":+1"),raw.replace("\"code\":-233","\"code\":\"+1\""))
        val duplicates=listOf(raw.replace("\"code\":-233","\"code\":null,\"code\":-233"),
            raw.replace("\"type\":\"system_gui\"","\"type\":\"bad\",\"type\":\"system_gui\""))
        for(input in cases+duplicates) check(tape,input,::arrangement)
        tape.finish()
    }
    @Test fun emojiHistoriesAndEveryUtf16UnitMatch() {
        val random=Random(1703)
        val tape=GoldenTape("settings-codec-history")
        val inputs=(0..65535).map {it.toChar().toString()}.chunked(128)+List(1000) {List(random.nextInt(10)) {text(random)}}
        for(strings in inputs) {
            val h=EmojiHistory(strings.map {Emoji(it,"",emptyList())},strings.reversed().map {Emoji(it,"",emptyList())})
            val raw=SettingsCodecs.history(h)
            tape.record(raw)
            assertEquals(h,SettingsCodecs.history(raw))
        }
        for(raw in listOf("{}","null","[]","{\"pinned\":[],\"recent\":[],\"unknown\":1}","{\"pinned\":[null],\"recent\":[]}","{\"pinned\":[],\"recent\":[]} trailing", "{\"pinned\":null,\"pinned\":[\"😀\"],\"recent\":[]}"))
            tape.record(values(SettingsCodecs.history(raw)))
        tape.finish()
    }
}
