package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.settings.SettingsCodecs
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class SettingsNumericMatchTest {
    @Test fun storedJsonIntegerAndExponentAdmissionMatchesTheOriginalLexer() {
        val random=Random(1714)
        val directed=listOf("0","-0","+1","1e2","1E-2","10e-1","1e","-e","1e+","1e--2","1e2+3", "1e+-1",
            "9223372036854775807","-9223372036854775808","9223372036854775808","18446744073709551617","1.0","١", "\\u0031")
        val numbers=directed+List(10000) {
            buildString {repeat(random.nextInt(0,30)) {append("0123456789eE+-.$".random(random))}}
        }+List(10000) { "${random.nextLong()}e${random.nextInt(-330,310)}" }
        for(number in numbers) for(quoted in listOf(false,true)) {
            val token=if(quoted) "\"$number\"" else number
            val raw="[{\"id\":$token,\"primaryLocale\":\"en-US\",\"secondaryLocales\":[],\"composer\":\"a:b\",\"currencySet\":\"a:b\",\"popupMapping\":\"a:b\",\"layoutMap\":{}}]"
            val expected=runCatching {ReferenceSettingsCodecs.subtypes(ReferenceSettingsCodecs.subtypes(raw))}
            val actual=runCatching {SettingsCodecs.subtypes(SettingsCodecs.subtypes(raw))}
            assertEquals(expected.isSuccess,actual.isSuccess,"synthetic numeric index ${numbers.indexOf(number)} quoted=$quoted")
            if(expected.isSuccess) assertEquals(expected.getOrThrow(),actual.getOrThrow())
        }
    }
}
