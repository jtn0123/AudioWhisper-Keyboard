package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.settings.SettingsCodecs
import dev.patrickgold.florisboard.ime.core.*
import dev.patrickgold.florisboard.ime.keyboard.*
import dev.patrickgold.florisboard.ime.text.keyboard.*
import dev.patrickgold.florisboard.ime.media.emoji.*
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.smartbar.quickaction.*
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlinx.serialization.json.Json
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class SettingsCodecMatchTest {
    private fun text(random: Random) = buildString {repeat(random.nextInt(0,24)) {append(random.nextInt(65536).toChar())}}
    private fun compare(raw: String, old: (String)->String, new: (String)->String) {
        val expected=runCatching {old(raw)};val actual=runCatching {new(raw)}
        assertEquals(expected.isSuccess,actual.isSuccess,"codec admission")
        if(expected.isSuccess) assertEquals(expected.getOrThrow(),actual.getOrThrow(),"canonical bytes")
    }
    @Test fun subtypeStoredFormatAndBundledPresetsMatch() {
        val random=Random(1701)
        val presets=java.io.File("src/main/assets").walkTopDown().filter {it.isFile && it.name.endsWith(".json") && it.readText().contains("\"preferred\"")}.toList()
        assertEquals(true,presets.isNotEmpty())
        // Real bundled subtype presets use the same nested layout/provider/component defaults.
        for(file in presets) {
            val node=Json.parseToJsonElement(file.readText())
            fun walk(v:kotlinx.serialization.json.JsonElement) {
                when(v) {
                    is kotlinx.serialization.json.JsonArray -> v.forEach(::walk)
                    is kotlinx.serialization.json.JsonObject -> {
                        if("preferred" in v && "languageTag" in v) {
                            val preset=SubtypeJsonConfig.decodeFromString<SubtypePreset>(v.toString())
                            val s=preset.toSubtype()
                            assertEquals(ReferenceSettingsCodecs.subtypes(listOf(s)),SettingsCodecs.subtypes(listOf(s)))
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
            val old=ReferenceSettingsCodecs.subtypes(listOf(s))
            assertEquals(old,SettingsCodecs.subtypes(listOf(s)))
            compare(old,{ReferenceSettingsCodecs.subtypes(ReferenceSettingsCodecs.subtypes(it))},{SettingsCodecs.subtypes(SettingsCodecs.subtypes(it))})
            compare(old.replace("\"composer\":","\"unknown\":[nonsense,{}],\"composer\":"),
                {ReferenceSettingsCodecs.subtypes(ReferenceSettingsCodecs.subtypes(it))},{SettingsCodecs.subtypes(SettingsCodecs.subtypes(it))})
        }
        for(raw in listOf("[]","[{}]","null","[","[null]","[1]","[{},]")) compare(raw,
            {ReferenceSettingsCodecs.subtypes(ReferenceSettingsCodecs.subtypes(it))},{SettingsCodecs.subtypes(SettingsCodecs.subtypes(it))})
        assertEquals(emptyList(),SettingsCodecs.subtypes(" \t\n"))
        // Frozen Serializable models supply independent nested defaults.
        assertEquals(ReferenceSubtype.DEFAULT.layoutMap.toString(),Subtype.DEFAULT.layoutMap.toString())
        val legacy=SubtypeJsonConfig.encodeToString(ReferenceSubtype.DEFAULT)
        assertEquals(legacy,SettingsCodecs.subtype(Subtype.DEFAULT))
        val minimum="[{\"id\":1,\"primaryLocale\":\"en-US\",\"secondaryLocales\":[],\"composer\":\"a:b\",\"currencySet\":\"a:b\",\"popupMapping\":\"a:b\",\"layoutMap\":{}}]"
        compare(minimum,{ReferenceSettingsCodecs.subtypes(ReferenceSettingsCodecs.subtypes(it))},{SettingsCodecs.subtypes(SettingsCodecs.subtypes(it))})
    }
    @Test fun arrangementsAllKeyVariantsPopupsAndCorruptInputsMatch() {
        val random=Random(1702)
        val keys=listOf<KeyData>(TextKeyData(),TextKeyData.DELETE,AutoTextKeyData(code=65),MultiTextKeyData(codePoints=intArrayOf(0x1f600,65)),
            TextKeyData(code=65,popup=PopupSet(CaseSelector(TextKeyData(code=97),TextKeyData(code=65)),listOf(
                ShiftStateSelector(shifted=TextKeyData.SPACE),VariationSelector(normal=TextKeyData.DELETE),
                LayoutDirectionSelector(TextKeyData.SPACE,TextKeyData.DELETE),CharWidthSelector(null,TextKeyData.SPACE)))))
        repeat(2000) {
            val actions=List(random.nextInt(0,12)) {if(random.nextBoolean()) QuickAction.InsertKey(keys.random(random)) else QuickAction.InsertText(text(random))}
            val a=QuickActionArrangement(actions.firstOrNull(),actions,actions.reversed())
            val old=ReferenceSettingsCodecs.arrangement(a)
            assertEquals(old,SettingsCodecs.arrangement(a))
            compare(old,{ReferenceSettingsCodecs.arrangement(ReferenceSettingsCodecs.arrangement(it))},{SettingsCodecs.arrangement(SettingsCodecs.arrangement(it))})
        }
        val raw=ReferenceSettingsCodecs.arrangement(ReferenceQuickActionDefaults.Default)
        assertEquals(raw,SettingsCodecs.arrangement(QuickActionArrangement.Default))
        val cases=listOf(raw,raw.replace("insert_key","future_action"),raw.replace("text_key","future_key"),raw.replace("\"stickyAction\":","\"future\":{invalid},\"stickyAction\":"),
            raw.replace("\"code\":-233","\"code\":\"-233\""),raw.dropLast(1),"{}","null","[]",raw.replace("\"code\":-233","\"code\":2147483648"),
            raw.replace("\"type\":\"system_gui\"","\"type\":\"system_guı\""),
            raw.replace("\"code\":-233","\"code\":+1"),raw.replace("\"code\":-233","\"code\":\"+1\""))
        for(input in cases) compare(input,{ReferenceSettingsCodecs.arrangement(ReferenceSettingsCodecs.arrangement(it))},{SettingsCodecs.arrangement(SettingsCodecs.arrangement(it))})
    }
    @Test fun emojiHistoriesAndEveryUtf16UnitMatch() {
        val random=Random(1703)
        val inputs=(0..65535).map {it.toChar().toString()}.chunked(128)+List(1000) {List(random.nextInt(10)) {text(random)}}
        for(strings in inputs) {
            val h=EmojiHistory(strings.map {Emoji(it,"",emptyList())},strings.reversed().map {Emoji(it,"",emptyList())})
            val raw=ReferenceSettingsCodecs.history(h)
            assertEquals(raw,SettingsCodecs.history(h))
            assertEquals(h,SettingsCodecs.history(raw))
        }
        for(raw in listOf("{}","null","[]","{\"pinned\":[],\"recent\":[],\"unknown\":1}","{\"pinned\":[null],\"recent\":[]}","{\"pinned\":[],\"recent\":[]} trailing"))
            assertEquals(ReferenceSettingsCodecs.history(raw),SettingsCodecs.history(raw))
    }
}
