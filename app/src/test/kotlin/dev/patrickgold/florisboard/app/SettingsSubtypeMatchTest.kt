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

class SettingsSubtypeMatchTest {
    @Test fun twoHundredSeedsOfSubtypeEditsLocaleChangesAndSelectionMatch() {
        var operations=0
        val tape=GoldenTape("settings-subtype-sessions")
        repeat(200) {seed ->
            val random=Random(seed+17000)
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
                // Select of an unknown id has no active-state effect.
                actual=plan.list;active=if(op=="select" && actual.none {it.id==target}) active else plan.selected
                tape.answer("changed",plan.changed)
                tape.answer("list",SettingsCodecs.subtypes(actual))
                tape.answer("active",plan.active)
                tape.answer("selected",active)
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
