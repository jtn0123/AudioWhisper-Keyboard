package org.florisboard.lib.snygg

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.jtn0123.audiowhisper.rust.SnyggValues
import org.florisboard.lib.snygg.reference.*
import org.florisboard.lib.snygg.value.*
import org.junit.jupiter.api.Test
import kotlin.random.Random
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SnyggValueMatchTest {
    private val encoders get() = NativeSnyggValues.encoders
    internal fun check(id: Int, input: String) {
        val reference = referenceValueEncoders[id]
        val native = encoders[id]
        val a = reference.deserialize(input)
        val b = native.deserialize(input)
        sameResult("encoder $id input $input", a, b, { it.production() }, { it })
        if (a.isSuccess && b.isSuccess) {
            assertEquals(native, b.getOrThrow().encoder(), "encoder choice $id")
            val serialized = reference.serialize(a.getOrThrow())
            val actual = native.serialize(b.getOrThrow())
            sameResult("serialization $id $input", serialized, actual, {it}, {it})
            if (serialized.isSuccess) {
                sameResult("round trip $id $input", reference.deserialize(serialized.getOrThrow()),
                    native.deserialize(actual.getOrThrow()), {it.production()}, {it})
            }
        }
    }
    internal fun all(input: String) {
        encoders.indices.forEach { check(it, input) }
        val selected = referenceValueEncoders.firstNotNullOfOrNull { it.deserialize(input).getOrNull() }
        val actual = SnyggValues.call(8, (listOf(encoders.size.toString()) + encoders.indices.map {it.toString()} + input).toTypedArray()).toList()
        assertEquals(selected?.production(), actual.takeIf {it.isNotEmpty()}?.let(NativeSnyggValues::value), "selected $input")
    }
    @Test fun `metadata defaults wrong encoder types and specs match`() {
        assertEquals(28, encoders.size)
        for (id in encoders.indices) {
            val a = referenceValueEncoders[id]; val b = encoders[id]
            assertEquals(a.defaultValue().production(), b.defaultValue(), "default $id")
            assertEquals(a.spec is ReferenceSnyggKeywordValueSpec, b.spec is SnyggKeywordValueSpec, "keyword schema type $id")
            val refSpecs = listOf(a.spec) + a.alternativeSpecs
            val specs = listOf(b.spec) + b.alternativeSpecs
            assertEquals(refSpecs.map { listOf(it.id, it.parsePattern.toString(), it.packTemplate) },
                specs.map { listOf(it.id, it.parsePattern.toString(), it.packTemplate) }, "specs $id")
            for (other in encoders.indices) {
                sameResult("wrong type $id $other", a.serialize(referenceValueEncoders[other].defaultValue()),
                    b.serialize(encoders[other].defaultValue()), {it}, {it})
            }
            for (i in specs.indices) {
                for (input in directed) {
                    val ma = referenceSnyggIdToValueMapOf("existing" to "kept")
                    val mb = snyggIdToValueMapOf("existing" to "kept")
                    sameResult("spec parse $id $i $input", runCatching {refSpecs[i].parse(input,ma);ma},
                        runCatching {specs[i].parse(input,mb);mb}, {it}, {it})
                    assertEquals(ma, mb)
                    sameResult("spec pack $id $i", runCatching {refSpecs[i].pack(ma)}, runCatching {specs[i].pack(mb)}, {it}, {it})
                }
            }
        }
    }
    private val directed = listOf("", " ", "inherit", "undefined", "defined", "yes", "no", "transparent", "TRANSPARENT",
        "0dp", "1dp", ".5dp", "1.dp", "-1dp", "+1dp", "01dp", "0DP", "1px", "1 dp", "NaNdp", "Infinitydp",
        "0sp", "1sp", ".1sp", "0.5sp", "10.sp", "1e2sp", "100%", "100.000%", "100.1%", "-1%", "0%", "99.999%",
        "rgb(0,255,123)", "rgba(0, 1, 2, .5)", "rgba(0,0,0,1.00)", "rgba(256,0,0,1)", "rgba(-1,0,0,1)", "rgba(0,0,0,1.1)",
        "#000000", "#ABCdef", "#ffffff00", "#01234567", "#123", "#1234567", "#gggggg", "#123456789", "rgb(1,2)", "rgba(1,2,3,)",
        "dynamic-light-color(primary)", "dynamic-dark-color(surfaceContainerHighest)", "dynamic-light-color(Primary)", "dynamic-dark-color(surface)",
        "normal", "italic", "bold", "400", "450", "system", "sans-serif", "`A B_2-`", "``", "`é`", "`A\nB`", "`A\u00a0B`",
        "0dp 1dp 2dp 3dp", "1dp 2dp", "1dp", "1dp\t2dp", "1dp  2dp", "1dp 2dp 3dp", "rectangle()", "circle( )",
        "rounded-corner(1.5dp,2dp,3dp,4dp)", "cut-corner(1.5dp,2dp,3dp,4dp)", "cut-corner(1dp,2dp,3dp,4dp)",
        "rounded-corner(0%,100%,50%,1%)", "cut-corner(101%,0%,0%,0%)", "rounded-corner(-1dp,0dp,0dp,0dp)",
        "left", "underline", "line-through", "none", "2147483647", "2147483648", "clip", "fill-bounds", "var(--missing)", "var(--)", "var(--a_b)",
        "uri(`flex:/a`)", "uri(`http:/a`)", "uri(`flex:/a b`)", "uri(`flex:/%20`)", "uri(`flex:/%ZZ`)", "uri(`flex:/a?x=y#z`)",
        "uri(`flex:/a#x#z`)", "uri(`flex:/a[0]`)", "uri(`flex://`)", "uri(`flex:/é`)", "uri(`flex:/\uD800`)", "uri(`flex:/a\tb`)")

    @Test fun `directed cases whitespace case and seeded mutations match`() {
        for (input in directed) {
            all(input); all(" \t$input\r\n"); all(input.uppercase()); all(input.lowercase())
        }
        val random = Random(0x24C0DE)
        repeat(7000) {
            val r = random.nextInt(-300,600); val g = random.nextInt(-300,600); val b = random.nextInt(-300,600)
            val number = listOf(random.nextFloat().toString(), random.nextInt().toString(), "0", "00", ".5", "1.", "NaN", "1e8", "9".repeat(80)).random(random)
            val input = when (it % 7) {
                0 -> number + listOf("dp","sp","%","px","DP"," dp").random(random)
                1 -> "rgba($r,$g,$b,$number)"
                2 -> "#" + List(random.nextInt(12)) { "0123456789abcdefXYZ".random(random) }.joinToString("")
                3 -> "${listOf("rounded-corner","cut-corner").random(random)}($number${listOf("dp","%").random(random)},0dp,0dp,0dp)"
                4 -> "var(--${List(random.nextInt(15)) { "abc-09_ AZ".random(random) }.joinToString("")})"
                5 -> directed.random(random).let { s -> if (s.isEmpty()) "?" else s.replaceRange(random.nextInt(s.length), random.nextInt(s.length+1).coerceAtLeast(s.length), "?") }
                else -> "uri(`flex:/" + List(random.nextInt(20)) { "abc%012[]?#@<>\\\t\u00a0".random(random) }.joinToString("") + "`)"
            }
            all(input)
        }
    }

    @Test fun `programmatic float and integer serialization edge cases match`() {
        val random = Random(0x24F10A7)
        val floats = listOf(0f, -0f, Float.NaN, Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY, Float.MIN_VALUE,
            Float.MAX_VALUE, java.lang.Float.MIN_NORMAL, 0.001f, 0.0001f, 10000000f, 99.99999f, 100f, -1f, 1f) + List(20000) {Float.fromBits(random.nextInt())}
        for (f in floats) {
            val pairs = listOf(4 to (ReferenceSnyggDpSizeValue(f.dp) to SnyggDpSizeValue(f.dp)),
                5 to (ReferenceSnyggSpSizeValue(f.sp) to SnyggSpSizeValue(f.sp)),
                6 to (ReferenceSnyggPercentageSizeValue(f) to SnyggPercentageSizeValue(f)),
                19 to (ReferenceSnyggRoundedCornerDpShapeValue(f.dp,0.dp,0.dp,0.dp) to SnyggRoundedCornerDpShapeValue(f.dp,0.dp,0.dp,0.dp)))
            for ((id, pair) in pairs) sameResult("float $id ${f.toRawBits()}", referenceValueEncoders[id].serialize(pair.first),encoders[id].serialize(pair.second),{it},{it})
        }
        for (n in listOf(Int.MIN_VALUE,-1,0,1,99,100,101,Int.MAX_VALUE)) {
            sameResult("lines $n", ReferenceSnyggTextMaxLinesValue.serialize(ReferenceSnyggTextMaxLinesValue(n)), SnyggTextMaxLinesValue.serialize(SnyggTextMaxLinesValue(n)),{it},{it})
            sameResult("corners $n", ReferenceSnyggRoundedCornerPercentShapeValue.serialize(ReferenceSnyggRoundedCornerPercentShapeValue(n,n,n,n, androidx.compose.foundation.shape.RoundedCornerShape(0.dp))), SnyggRoundedCornerPercentShapeValue.serialize(SnyggRoundedCornerPercentShapeValue(n,n,n,n, androidx.compose.foundation.shape.RoundedCornerShape(0.dp))),{it},{it})
        }
    }
}
