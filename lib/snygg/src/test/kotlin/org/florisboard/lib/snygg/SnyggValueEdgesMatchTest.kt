package org.florisboard.lib.snygg

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import org.florisboard.lib.snygg.reference.*
import org.florisboard.lib.snygg.value.*
import org.junit.jupiter.api.Test
import kotlin.random.Random
import kotlin.test.assertEquals

class SnyggValueEdgesMatchTest {
    private val values = SnyggValueMatchTest()
    @Test fun `every UTF16 unit in font names and URIs preserves parsing and error classes`() {
        for (u in Char.MIN_VALUE..Char.MAX_VALUE) {
            values.check(11,"`A${u}Z`")
            values.check(25,"uri(`flex:/a${u}z`)")
        }
        for (space in listOf(' ','\t','\r','\n','\u000b','\u000c','\u001c','\u0085','\u00a0','\u1680','\u2007','\u2028','\u202f','\u3000')) {
            for (id in NativeSnyggValues.encoders.indices) {
                val a = referenceValueEncoders[id].defaultValue()
                val serialized = referenceValueEncoders[id].serialize(a).getOrNull() ?: continue
                values.check(id,"$space$serialized$space")
            }
        }
        for (s in listOf("😀","\uD800\uD800","\uDC00\uDC00","\uD800x\uDC00","\uDBFF\uDFFF")) {
            values.check(11,"`$s`");values.check(25,"uri(`flex:/$s`)")
        }
    }
    @Test fun `valid generated colours dimensions padding corners and every dynamic name match`() {
        val random = Random(0x24C0107)
        val names = org.florisboard.lib.color.ColorPalette.colorNames
        for (name in names) for (id in listOf(8,9)) {
            val prefix = if (id==8) "dynamic-light-color" else "dynamic-dark-color"
            values.check(id,"$prefix($name)");values.check(id,"$prefix( $name )");values.check(id,"$prefix(${name.uppercase()})")
        }
        repeat(5000) {
            val r=random.nextInt(256);val g=random.nextInt(256);val b=random.nextInt(256);val a=random.nextFloat()
            values.check(7,"rgba($r,$g,$b,$a)")
            values.check(7,"rgb($r,$g,$b)")
            values.check(7,"#%02x%02x%02x%02x".format(r,g,b,random.nextInt(256)))
            val n=List(4) {random.nextInt(101)}
            values.check(14,"${n[0]}dp ${n[1]}dp ${n[2]}dp ${n[3]}dp")
            values.check(14,"${n[0]}dp ${n[1]}dp");values.check(14,"${n[0]}dp")
            for (id in listOf(17,18,19,20)) {
                val kind=if (id<19) "cut-corner" else "rounded-corner"
                val unit=if(id==18||id==20) "%" else "dp"
                values.check(id,n.joinToString(",", "$kind(", ")") {"$it$unit"})
            }
            values.check(4,"${random.nextFloat()*1000}dp")
            values.check(5,"${random.nextInt(1,1000)}.${random.nextInt(100)}sp")
            values.check(6,"${random.nextFloat()*100}%")
            values.check(23,random.nextInt(1,Int.MAX_VALUE).toString())
        }
        for (byte in 0..255) values.check(7,"rgba(1,2,3,${byte/255f})")
    }
    @Test fun `URI authority IPv6 escape query and fragment edges match`() {
        val authorities = listOf("", "localhost", "a-b.example:80", "user:pass@host:80", "host:port", "a_b", "@", ":", "a@@b", "a%20b",
            "[::1]", "[::]", "[2001:db8::1]:8080", "user@[::ffff:192.168.001.1]", "[::1%lo0]", "[::1%25en0]", "[::1%]",
            "[", "[::%x", "]%x", "]", "[]", "[gg]", "[1:2]", "[1:2:3:4:5:6:7:8]", "[1:2:3:4:5:6:7:8:9]", "[12345::]", "[1:::2]", "[::1]x",
            "[::1]:-1", "[::1]:2147483648", "[::1]:", "[::1]:a", "x[::1]", "a[b]@host", "[::ffff:999.1.1.1]")
        for (a in authorities) for (suffix in listOf("", "/a", "/a?x=[a]#f", "?q", "#f")) values.check(25,"uri(`flex://$a$suffix`)")
        for (path in listOf("a%20b","a%2","a%xx","a#x#z","a?x?z","a?x[y]","a#x[y]","/","a|b","a\\b","a^b","a{b}","a<b>")) values.check(25,"uri(`flex:/$path`)")
    }
    @Test fun `invalid programmatic text dynamic names padding colours and enum fields match`() {
        for (s in listOf("", "wrong", "primary", "Primary", "--ok", "--bad_name", "font Name", "\uD800", "flex:/%zz")) {
            val pairs = listOf(
                8 to (ReferenceSnyggDynamicLightColorValue(s) to SnyggDynamicLightColorValue(s)),
                9 to (ReferenceSnyggDynamicDarkColorValue(s) to SnyggDynamicDarkColorValue(s)),
                11 to (ReferenceSnyggCustomFontFamilyValue(s) to SnyggCustomFontFamilyValue(s)),
                25 to (ReferenceSnyggUriValue(s) to SnyggUriValue(s)),
                27 to (ReferenceSnyggDefinedVarValue(s) to SnyggDefinedVarValue(s)))
            for ((id,pair) in pairs) sameResult("field $id",referenceValueEncoders[id].serialize(pair.first),NativeSnyggValues.encoders[id].serialize(pair.second),{it},{it})
        }
        for (f in listOf(-1f,-0f,Float.NaN,Float.POSITIVE_INFINITY,0f,1f,Float.MAX_VALUE)) {
            val p=object : PaddingValues {
                override fun calculateLeftPadding(layoutDirection: androidx.compose.ui.unit.LayoutDirection) = f.dp
                override fun calculateTopPadding() = f.dp
                override fun calculateRightPadding(layoutDirection: androidx.compose.ui.unit.LayoutDirection) = f.dp
                override fun calculateBottomPadding() = f.dp
            }
            sameResult("padding $f",ReferenceSnyggPaddingValue.serialize(ReferenceSnyggPaddingValue(p)),SnyggPaddingValue.serialize(SnyggPaddingValue(p)),{it},{it})
        }
        for (color in listOf(Color.Unspecified,Color.Transparent,Color(0.123f,0.456f,0.789f,0.234f),Color(0.2f,0.3f,0.4f,0.5f,androidx.compose.ui.graphics.colorspace.ColorSpaces.DisplayP3))) {
            sameResult("color",ReferenceSnyggStaticColorValue.serialize(ReferenceSnyggStaticColorValue(color)),SnyggStaticColorValue.serialize(SnyggStaticColorValue(color)),{it},{it})
        }
        for (weight in listOf(1,99,100,450,900,1000)) {
            val w=androidx.compose.ui.text.font.FontWeight(weight)
            sameResult("weight $weight",ReferenceSnyggFontWeightValue.serialize(ReferenceSnyggFontWeightValue(w)),SnyggFontWeightValue.serialize(SnyggFontWeightValue(w)),{it},{it})
        }
    }
}
