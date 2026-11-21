package dev.patrickgold.florisboard.app

import androidx.compose.ui.graphics.Color
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.settings.SettingsSelection
import com.jtn0123.audiowhisper.settings.SettingsTimeSerializer
import dev.patrickgold.florisboard.app.settings.theme.ColorPreferenceSerializer
import dev.patrickgold.florisboard.app.settings.localization.SubtypeEditorState
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.core.SubtypeNlpProviderMap
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.jetpref.datastore.model.LocalTime
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.random.Random

class SettingsDerivationMatchTest {
    @Test fun editorValidationAndResourceSelectionMatch() {
        val random=Random(1712)
        val values=listOf("valid", "00:00", "00-00", "", "00", "synthetic:a", "😀")
        repeat(5000) {
            val old=ReferenceSubtypeEditorState(Subtype.DEFAULT)
            val new=SubtypeEditorState(Subtype.DEFAULT)
            val locale=FlorisLocale.fromTag(listOf("en-US","fr-FR","00-00","00","zz").random(random))
            val provider=SubtypeNlpProviderMap(values.random(random),values.random(random))
            val component=ExtensionComponentName("00",if(random.nextBoolean()) "00" else "valid")
            old.primaryLocale.value=locale;new.primaryLocale.value=locale
            old.nlpProviders.value=provider;new.nlpProviders.value=provider
            val field=random.nextInt(7)
            when(field) {
                0 -> {old.composer.value=component;new.composer.value=component}
                1 -> {old.currencySet.value=component;new.currencySet.value=component}
                2 -> {old.punctuationRule.value=component;new.punctuationRule.value=component}
                3 -> {old.popupMapping.value=component;new.popupMapping.value=component}
                4 -> {old.layoutMap.value=old.layoutMap.value.copy(characters=component);new.layoutMap.value=old.layoutMap.value}
                5 -> {old.layoutMap.value=old.layoutMap.value.copy(phone2=component);new.layoutMap.value=old.layoutMap.value}
            }
            val expected=old.toSubtype();val actual=new.toSubtype()
            assertEquals(expected.isSuccess,actual.isSuccess)
            if(expected.isSuccess) assertEquals(expected.getOrThrow(),actual.getOrThrow())
            val options=List(random.nextInt(10)) {values.random(random)};val target=values.random(random)
            assertEquals(options.indexOf(target).coerceAtLeast(0),SettingsSelection.index(target,options))
        }
        for(number in listOf(false,true)) for(symbol in listOf(false,true)) for(mode in listOf("DISABLED","SMART_PRIORITY","ACCENT_PRIORITY")) {
            val modes=SettingsRules.call(SettingsOp.KEY_HINTS,*with(SettingsKeyHints) {
                row(NUMBER_ENABLED to WireFlag.encode(number),NUMBER to mode,SYMBOLS_ENABLED to WireFlag.encode(symbol),SYMBOLS to mode) })
            assertEquals(if(number)mode else "DISABLED",modes[SettingsKeyHintModes.NUMBER])
            assertEquals(if(symbol)mode else "DISABLED",modes[SettingsKeyHintModes.SYMBOLS])
        }
    }
    @Test fun colorsAndTimeEncodingRangesAndMalformedValuesMatch() {
        val random=Random(1713)
        val colors=List(5000) {random.nextLong().toULong().toString(16)}+listOf("","+1","-1","0x1","0","f","ABCDEF","00000000000000000","\ud800")
        for(raw in colors) {
            val old=runCatching {ReferenceColorSerializer.serialize(ReferenceColorSerializer.deserialize(raw))}
            val new=runCatching {ColorPreferenceSerializer.serialize(ColorPreferenceSerializer.deserialize(raw))}
            assertEquals(old.isSuccess,new.isSuccess,"synthetic color index ${colors.indexOf(raw)}")
            if(old.isSuccess) assertEquals(old.getOrThrow(),new.getOrThrow())
        }
        for(hour in -1..24) for(minute in listOf(-1,0,1,59,60)) for(second in listOf(-1,0,59,60)) for(ms in listOf(-1,0,999,1000)) {
            val raw="%02d:%02d:%02d.%03d".format(java.util.Locale.ROOT,hour,minute,second,ms)
            assertEquals(LocalTime.parse(raw).getOrNull(),SettingsTimeSerializer.deserialize(raw))
        }
        repeat(5000) {
            val time=LocalTime(random.nextInt(24),random.nextInt(60),random.nextInt(60),random.nextInt(1000))
            assertEquals(time.toString(),SettingsTimeSerializer.serialize(time))
        }
    }
}
