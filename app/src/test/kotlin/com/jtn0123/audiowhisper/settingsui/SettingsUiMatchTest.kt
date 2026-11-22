package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import dev.patrickgold.florisboard.app.ReferenceSettingsModel
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import kotlinx.serialization.json.*
import kotlin.math.roundToInt
import kotlin.random.Random
import kotlin.test.*

class SettingsUiMatchTest {
    private val ref = ReferenceSettingsScreens
    private fun <T : Any> raw(p: PreferenceData<T>) = p.serializer.serialize(p.default).orEmpty()
    private fun defaults() = ReferenceSettingsModel().declaredPreferenceEntries.values.associate { it.key to raw(it) }
    private fun state(native: KeyboardRenderer, op: SettingsUiOp = SettingsUiOp.SAVE, vararg input: String) = Json.parseToJsonElement(native.settings(op, arrayOf(*input))[0]).jsonObject
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun prepare(n: KeyboardRenderer, values: Map<String,String>) {
        n.settings(SettingsUiOp.PREFERENCES, values.flatMap { listOf(it.key,it.value) }.toTypedArray())
        n.settings(SettingsUiOp.CONFIG, arrayOf("1080","8192","1","1","4294967295","4278190080","4283782485","4281558681","4291611852","4293848814","true","true"))
    }
    @Test fun allHomeRoutesMatchTheComposeSource() {
        KeyboardRenderer(emptyArray()).use { n ->
            prepare(n,defaults())
            // Groups 1 and 2 (Keyboard, Smartbar, Gestures, Clipboard, Media, Typing, About, Dictation
            // and the screens they push), Other and Theme, Back up and Restore, and the language and user dictionary screens
            // are drawn natively; the rest fall back to Compose, which is told every route it must hand back.
            val native=listOf("settings/home","settings/keyboard","settings/keyboard/input-feedback","settings/smartbar",
                "settings/gestures","settings/clipboard","settings/media","settings/typing","settings/dictionary",
                "settings/other/physical-keyboard","settings/about","settings/dictation",
                "settings/about/third-party-licenses","settings/other","settings/theme",
                "settings/other/backup","settings/other/restore") + languageRoutes + wordRoutes
            for (i in ref.home.indices) {
                if (ref.home[i] in native) {
                    assertEquals(ref.home[i],state(n,SettingsUiOp.ACTIVATE,(100+i).toString()).getValue("route").jsonPrimitive.content)
                    assertEquals("settings/home",state(n,SettingsUiOp.BACK,"3","0").getValue("route").jsonPrimitive.content)
                } else assertEquals(listOf(listOf("compose",ref.home[i],native.joinToString("|"))),effects(state(n,SettingsUiOp.ACTIVATE,(100+i).toString())))
            }
            val keyboard=state(n,SettingsUiOp.ACTIVATE,"102")
            assertEquals("settings/keyboard",keyboard.getValue("route").jsonPrimitive.content)
            state(n,SettingsUiOp.BACK,"0","0"); state(n,SettingsUiOp.BACK,"1","0.8"); val cancelled=state(n,SettingsUiOp.BACK,"2","0")
            assertEquals("settings/keyboard",cancelled.getValue("route").jsonPrimitive.content)
            assertEquals("settings/home",state(n,SettingsUiOp.BACK,"3","0").getValue("route").jsonPrimitive.content)
            assertEquals(listOf(listOf("picker")),effects(state(n,SettingsUiOp.ACTIVATE,"3")))
        }
    }
    @Test fun everyKeyboardVisibilityEnableStateAndControlRoleMatchesFrozenCompose() {
        repeat(64) { mask ->
            val p=defaults().toMutableMap()
            listOf("number_row","utility_key_enabled","one_handed_mode_enabled","hinted_number_row_enabled","hinted_symbols_enabled","popup_enabled")
                .forEachIndexed { i,key -> p["keyboard__$key"]=(mask and (1 shl i)!=0).toString() }
            KeyboardRenderer(emptyArray()).use { n ->
                prepare(n,p); val s=state(n,SettingsUiOp.NAVIGATE,"settings/keyboard")
                val actual=nodes(s).filter { it.getValue("id").jsonPrimitive.int in 1000..1022 }
                assertEquals(ref.rows.indices.filter { ref.visible(it,p) }.map { 1000+it },actual.map { it.getValue("id").jsonPrimitive.int })
                for (node in actual) {
                    val i=node.getValue("id").jsonPrimitive.int-1000
                    assertEquals(ref.enabled(i,p),node.getValue("enabled").jsonPrimitive.boolean, "enabled row $i mask $mask")
                    assertEquals(when(ref.rows[i].kind){"toggle"->"toggle";"heading"->"heading";else->"button"},node.getValue("role").jsonPrimitive.content)
                    if (!node.getValue("enabled").jsonPrimitive.boolean) assertTrue(node.getValue("actions").jsonArray.isEmpty(), "disabled row $i offers an action")
                    if(ref.rows[i].kind=="toggle") assertEquals(p.getValue(ref.rows[i].key).toBoolean(),node.getValue("checked").jsonPrimitive.boolean)
                }
            }
        }
    }
    @Test fun togglesListsSlidersConfirmCancelAndDefaultsMatchOriginalTransactions() {
        val random=Random(20261114)
        KeyboardRenderer(emptyArray()).use { n ->
            val defaults=defaults();val p=defaults.toMutableMap();prepare(n,p);state(n,SettingsUiOp.NAVIGATE,"settings/keyboard")
            repeat(1000) {
                val i=ref.rows.indices.filter { ref.visible(it,p)&&ref.enabled(it,p)&&ref.rows[it].kind !in listOf("heading","route") }.random(random)
                val row=ref.rows[i]
                var reply=state(n,SettingsUiOp.ACTIVATE,(1000+i).toString())
                if(row.kind=="toggle") {
                    p[row.key]=(!p.getValue(row.key).toBoolean()).toString()
                    assertEquals(listOf(listOf("write",row.key,p.getValue(row.key))),effects(reply))
                } else {
                    assertTrue(reply.getValue("popup").jsonPrimitive.boolean)
                    val reset=random.nextInt(5)==0;val cancel=random.nextInt(4)==0
                    val expected=mutableListOf<List<String>>()
                    val keys=if(row.kind=="choice") listOf(row.key,row.switch).filter { it.isNotEmpty() } else listOf(row.key,row.second).filter { it.isNotEmpty() }
                    var choice=""
                    val sliderValues=mutableMapOf<String,String>()
                    var switchValue=if(row.switch.isEmpty())true else p.getValue(row.switch).toBoolean()
                    if(row.kind=="choice") {
                        if(!switchValue) {state(n,SettingsUiOp.ACTIVATE,"8");switchValue=true}
                        val options=ref.choices(i);val at=random.nextInt(options.size)
                        reply=state(n,SettingsUiOp.ACTIVATE,(3000+at).toString());choice=options[at]
                        assertEquals(1,nodes(reply).count { it.getValue("role").jsonPrimitive.content=="radio" && it.getValue("selected").jsonPrimitive.boolean })
                        if(row.switch.isNotEmpty() && random.nextInt(3)==0){reply=state(n,SettingsUiOp.ACTIVATE,"8");switchValue=!switchValue
                            assertTrue(nodes(reply).filter { it.getValue("role").jsonPrimitive.content=="radio" }.all { it.getValue("enabled").jsonPrimitive.boolean==switchValue })}
                    } else {
                        val (min,max,step)=ref.bounds.getValue(i)
                        for((slot,key) in keys.withIndex()) {
                            val input=min-10f+random.nextFloat()*(max-min+20f)
                            val value=(min+((input.coerceIn(min,max)-min)/step).roundToInt()*step).coerceIn(min,max)
                            state(n,SettingsUiOp.SLIDER,(4000+slot).toString(),input.toString())
                            sliderValues[key]=if(i==15)value.toString() else value.toInt().toString()
                        }
                    }
                    if(reset) {
                        reply=state(n,SettingsUiOp.ACTIVATE,"5")
                        for(key in keys){p[key]=defaults.getValue(key);expected+=listOf("reset",key)}
                    } else {
                        if(!cancel) for(key in keys){
                            val value=if(row.kind=="choice") {if(key==row.key)choice else switchValue.toString()} else sliderValues.getValue(key)
                            p[key]=value;expected+=listOf("write",key,value)
                        }
                        reply=state(n,SettingsUiOp.ACTIVATE,if(cancel)"6"else"7")
                    }
                    assertFalse(reply.getValue("popup").jsonPrimitive.boolean)
                    assertEquals(expected,effects(reply),"transaction row $i")
                }
            }
        }
    }
    @Test fun defaultImmediatelyResetsAndClosesLikeTheOriginalJetprefDialog() {
        KeyboardRenderer(emptyArray()).use { n ->
            val p=defaults().toMutableMap().apply { put("keyboard__space_bar_display_mode", "NOTHING") }
            prepare(n,p);state(n,SettingsUiOp.NAVIGATE,"settings/keyboard");state(n,SettingsUiOp.ACTIVATE,"1006")
            val result=state(n,SettingsUiOp.ACTIVATE,"5")
            assertEquals(listOf(listOf("reset","keyboard__space_bar_display_mode")),effects(result))
            assertFalse(result.getValue("popup").jsonPrimitive.boolean)
        }
    }
    @Test fun dpadAndTabTraversalRevealEveryEligibleControlAtLargeFontScale() {
        val p=defaults()
        KeyboardRenderer(emptyArray()).use { n ->
            prepare(n,p)
            n.settings(SettingsUiOp.CONFIG,arrayOf("480","640","1","2","4294967295","4278190080","4283782485","4281558681","4291611852","4293848814","true","true"))
            state(n,SettingsUiOp.NAVIGATE,"settings/keyboard")
            val expected=buildList {
                add(1)
                for (i in ref.rows.indices) if(ref.visible(i,p)&&ref.enabled(i,p)&&ref.rows[i].kind!="heading") {
                    add(1000+i);if(ref.rows[i].switch.isNotEmpty())add(2000+i)
                }
                add(2);add(3)
            }
            var scrolled=false
            repeat(expected.size*2) { at ->
                val result=state(n,SettingsUiOp.KEY,"0","61","0","false")
                val id=result.getValue("focus").jsonPrimitive.int
                assertEquals(expected[at%expected.size],id)
                assertTrue(nodes(result).any { it.getValue("id").jsonPrimitive.int==id },"focus must be visible")
                scrolled=scrolled||result.getValue("scroll").jsonPrimitive.float>0f
            }
            assertTrue(scrolled)
        }
    }
    @Test fun repeatedDeepLinksKeepTheExistingDestinationAndBoundedBackStack() {
        KeyboardRenderer(emptyArray()).use { n ->
            prepare(n,defaults())
            repeat(100) {state(n,SettingsUiOp.NAVIGATE,"settings/home");state(n,SettingsUiOp.NAVIGATE,"settings/keyboard")}
            assertEquals("settings/home|settings/keyboard",n.settings(SettingsUiOp.SAVE)[4])
            assertEquals("settings/home",state(n,SettingsUiOp.BACK,"3","0").getValue("route").jsonPrimitive.content)
        }
    }
    @Test fun savedDraftsAndPredictiveCancellationDoNotCommitPreferences() {
        KeyboardRenderer(emptyArray()).use { n ->
            prepare(n,defaults());state(n,SettingsUiOp.NAVIGATE,"settings/keyboard");state(n,SettingsUiOp.ACTIVATE,"1008");state(n,SettingsUiOp.SLIDER,"4000","125")
            val before=n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
            state(n,SettingsUiOp.BACK,"0","0");state(n,SettingsUiOp.BACK,"1","0.75");assertTrue(effects(state(n,SettingsUiOp.BACK,"2","0")).isEmpty())
            assertContentEquals(before,n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray())
            KeyboardRenderer(emptyArray()).use { recreated ->
                prepare(recreated,defaults());state(recreated,SettingsUiOp.RESTORE,*before)
                assertContentEquals(before,recreated.settings(SettingsUiOp.SAVE).drop(3).toTypedArray())
                assertEquals("125",effects(state(recreated,SettingsUiOp.ACTIVATE,"7"))[0][2])
            }
            assertTrue(effects(state(n,SettingsUiOp.BACK,"3","0")).isEmpty())
        }
    }
}
