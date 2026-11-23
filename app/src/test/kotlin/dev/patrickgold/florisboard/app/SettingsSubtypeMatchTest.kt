package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.SettingsRules
import com.jtn0123.audiowhisper.settings.SettingsCodecs
import com.jtn0123.audiowhisper.settings.SettingsSubtypes
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.core.SubtypeNlpProviderMap
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/** Pure extraction of the frozen manager's list and active-selection rules. */
private class ReferenceSubtypeSession {
    var list=emptyList<Subtype>();var active=Subtype.DEFAULT
    fun apply(op: String,item: Subtype,time: Long,target: Long,order: List<Int>): Boolean {
        var changed=false
        when(op) {
            "add" -> if(list.none {it.copy(id=0)==item.copy(id=0)}) {list=list+item.copy(id=time);changed=true}
            "modify" -> list.indexOfFirst {it.id==item.id}.takeIf {it>=0}?.let { index ->list=list.mapIndexed {i,s->if(i==index)item else s};changed=true}
            "remove" -> list.indexOf(item).takeIf {it>=0}?.let {index->list=list.filterIndexed {i,_->i!=index};changed=true}
            // No reorder API existed; the stored-list order is the cycle/fallback order.
            "reorder" -> if(order.sorted()==list.indices.toList()) {list=order.map {list[it]};changed=true}
            "load" -> {val supported=list.map {it.referenceSupportedInput()};changed=supported!=list;list=supported}
            "select" -> {list.find {it.id==target}?.let {active=it};return false}
            "next","previous" -> {
                var trigger=false;var chosen:Subtype?=null
                val items=if(op=="previous") list.reversed() else list
                for(s in items) {if(trigger) {trigger=false;chosen=s} else if(s==active) trigger=true}
                if(trigger) chosen=items.first()
                active=chosen ?: Subtype.DEFAULT;return false
            }
        }
        active=list.find {it.id==active.id} ?: list.firstOrNull() ?: Subtype.DEFAULT
        return changed
    }
}

class SettingsSubtypeMatchTest {
    @Test fun twoHundredSeedsOfSubtypeEditsLocaleChangesAndSelectionMatch() {
        var operations=0
        // The reference's answers also go on a golden tape (docs/phase6-kotlin-references.md).
        val tape=GoldenTape.ofReference("settings-subtype-sessions")
        repeat(200) {seed ->
            val random=Random(seed+17000);val reference=ReferenceSubtypeSession()
            var actual=emptyList<Subtype>();var active=Subtype.DEFAULT
            val locales=listOf("en-US","en-GB","fr-FR","de-DE","es-ES","vi-VN","pt-BR")
            val presets=locales.map {FlorisLocale.fromTag(it)}
            repeat(150) {step ->
                val locale=FlorisLocale.fromTag(locales.random(random))
                val candidate=if(actual.isNotEmpty() && random.nextBoolean()) actual.random(random) else Subtype.DEFAULT.copy(
                    id=random.nextLong(1,12),primaryLocale=locale,
                    layoutMap=Subtype.DEFAULT.layoutMap.copy(characters=ExtensionComponentName("org.florisboard.layouts",listOf("qwerty","azerty","korean").random(random))),
                    nlpProviders=if(random.nextInt(20)==0) SubtypeNlpProviderMap(spelling="org.florisboard.nlp.providers.han.shape") else Subtype.DEFAULT.nlpProviders)
                val op=listOf("add","add","remove","modify","reorder","select","load","next","previous").random(random)
                val order=if(random.nextInt(4)==0) listOf(-1) else actual.indices.shuffled(random)
                val target=if(actual.isNotEmpty() && random.nextBoolean()) actual.random(random).id else -500L
                val time=(seed*1000+step+1).toLong()
                val plan=SettingsSubtypes.plan(op,actual,active.id,candidate,time,target,active,order)
                val changed=reference.apply(op,candidate,time,target,order)
                tape.answer("changed",changed)
                tape.answer("list",ReferenceSettingsCodecs.subtypes(reference.list))
                tape.answer("active",reference.active.id)
                tape.answer("selected",reference.active)
                assertEquals(changed,plan.changed,"seed=$seed step=$step op=$op changed")
                assertEquals(reference.list,plan.list,"seed=$seed step=$step op=$op list")
                assertEquals(reference.active.id,plan.active,"seed=$seed step=$step op=$op active")
                // Select of an unknown id has no active-state effect.
                actual=plan.list;active=if(op=="select" && actual.none {it.id==target}) active else plan.selected
                assertEquals(reference.active,active,"seed=$seed step=$step op=$op selected")
                assertEquals(ReferenceSettingsCodecs.subtypes(reference.list),SettingsCodecs.subtypes(actual))
                val expected=presets.indexOfFirst {it==locale}.takeIf {it>=0} ?: presets.indexOfFirst {it.language==locale.language}
                assertEquals(expected,SettingsRules.preset(locale.languageTag(),locale.language,presets.map {it.languageTag() to it.language}))
                operations++
            }
        }
        tape.finish()
        assertEquals(30000,operations)
    }
    @Test fun activeRemovalUsesFirstRemainingThenFixedEnglishFallback() {
        val list=listOf(Subtype.DEFAULT.copy(id=9),Subtype.DEFAULT.copy(id=2),Subtype.DEFAULT.copy(id=7))
        val result=SettingsSubtypes.plan("remove",list,2,list[1])
        assertEquals(9,result.active)
        val empty=SettingsSubtypes.plan("remove",listOf(list[1]),2,list[1])
        assertEquals(Subtype.DEFAULT,empty.selected)
        assertEquals(-1,empty.active)
        assertEquals(-1,SettingsRules.preset("xx-ZZ","xx",listOf("en-US" to "en")))
        assertEquals(1,SettingsRules.preset("en-GB","en",listOf("en-US" to "en","en-GB" to "en")))
        assertEquals(0,SettingsRules.preset("en-AU","en",listOf("en-US" to "en","en-GB" to "en")))
    }
}
