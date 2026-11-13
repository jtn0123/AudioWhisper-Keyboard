package org.florisboard.lib.snygg

import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.sp
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import java.io.File
import kotlinx.serialization.json.*
import org.florisboard.lib.color.ColorPalette
import org.florisboard.lib.color.getColor
import org.florisboard.lib.snygg.value.SnyggSpSizeValue
import kotlin.test.Test
import kotlin.test.assertEquals

/** Only the reference supplies expected styles. The JNI calls the renderer's actual cached resolver. */
class RenderThemeMatchTest {
    private val root = File(requireNotNull(System.getProperty("snygg.repoRoot")))
    private val initial = SnyggSinglePropertySet(mapOf("font-size" to SnyggSpSizeValue(14.sp)))
    private val names = ColorPalette.colorNames
    private val colors = names.map { yellowLightScheme.getColor(it).toArgb() } + names.map { yellowDarkScheme.getColor(it).toArgb() }
    private val elements = listOf("window", "key", "key-hint", "smartbar", "smartbar-candidates-row",
        "smartbar-candidate-word", "smartbar-candidate-word-text", "smartbar-candidate-clip",
        "smartbar-candidate-clip-text", "key-popup-box", "key-popup-element")

    private fun expected(theme: ReferenceSnyggTheme, q: List<String>, multiplier: Float): SnyggSinglePropertySet {
        val name = q[0]; val mode = q[1]; val shift = q[2]; val code = q[3]
        val type = q[4]; val selector = q[5]; val auto = q[6]
        val attrs = mapOf("mode" to mode, "shiftstate" to shift, "code" to code.toInt(), "type" to type, "auto-commit" to auto.toInt())
        val state = SnyggSelector.entries[selector.toInt()]
        val parentName = when (name) {
            "root" -> null
            "window" -> "root"
            "key-hint" -> "key"
            "smartbar-candidates-row" -> "smartbar"
            "smartbar-candidate-word", "smartbar-candidate-clip" -> "smartbar-candidates-row"
            "smartbar-candidate-word-text" -> "smartbar-candidate-word"
            "smartbar-candidate-clip-text" -> "smartbar-candidate-clip"
            "key-popup-element" -> "key-popup-box"
            else -> "window"
        }
        val inheritSelector = name == "key-hint" || name.endsWith("-text")
        val parent = if (parentName == null) initial else expected(theme,
            listOf(parentName, mode, shift, code, type, if (inheritSelector) selector else "0", auto), multiplier)
        return theme.query(name, attrs, state, parent, yellowLightScheme, yellowDarkScheme, multiplier)
    }
    private fun queries(broad: Boolean): List<List<String>> = buildList {
        val codes = if (broad) listOf(97, 10, 32, -11, -201, -255) else listOf(97, 10, -11)
        for (mode in if (broad) listOf("characters", "symbols", "phone") else listOf("characters"))
            for (shift in if (broad) listOf("unshifted", "shifted_manual", "shifted_automatic", "caps_lock") else listOf("unshifted", "caps_lock"))
                for (code in codes) for (name in elements) for (state in if (broad) listOf(0, 1, 2, 4) else listOf(0, 1, 2)) {
                    // Vary candidate auto-commit and key type as well as code and shift.
                    add(listOf(name, mode, shift, code.toString(), if (code > 0) "character" else "function", state.toString(), if (code == 97) "1" else "0"))
                }
    }
    @Test fun `extension popup choices inherit their source key box across cache contexts`() {
        KeyboardRenderer(emptyArray()).close()
        val source = """{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key-popup-box[code=97]":{"font-size":"30sp","foreground":"#123456"},"key-popup-box[code=101]":{"font-size":"18sp","foreground":"#abcdef"}}"""
        val reference = ReferenceSnyggTheme.compileFrom(SnyggStylesheet.fromJson(source).getOrThrow())
        val queries = listOf(97, 101, 97).flatMap { code -> listOf(0, 2).map { state ->
            listOf("key-popup-element", "characters", "unshifted", "233", "character", state.toString(), "0", code.toString())
        } }
        val native = RenderThemeProbe.resolve(source, names.toTypedArray(), colors.toIntArray(),
            floatArrayOf(1f, 1f, 1f), queries.map { it.joinToString("\t") }.toTypedArray()).iterator()
        for (q in queries) {
            val parent = expected(reference, listOf("key-popup-box", q[1], q[2], q[7], q[4], "0", "0"), 1f)
            val expected = reference.query(q[0], mapOf("mode" to q[1], "shiftstate" to q[2], "code" to 233, "type" to q[4], "auto-commit" to 0),
                SnyggSelector.entries[q[5].toInt()], parent, yellowLightScheme, yellowDarkScheme, 1f)
            val actual = buildMap { repeat(native.next().toInt()) { put(native.next(), native.next()) } }
            assertEquals(expected.properties.mapValues { (_, v) -> v.encoder().serialize(v).getOrThrow() }, actual)
        }
    }
    @Test fun `renderer cache matches Kotlin reference for every bundled theme and drawing state`() {
        // Load the shared library using its normal wrapper before the test-only declaration.
        KeyboardRenderer(emptyArray()).close()
        val sheets = SnyggStylesheetMatchTest().sheets()
        assertEquals(12, sheets.size)
        var count = 0
        val fixture = buildJsonObject {
            put("names", JsonArray(names.map(::JsonPrimitive)))
            put("colors", JsonArray(colors.map(::JsonPrimitive)))
            put("themes", buildJsonObject {
                for ((path, source) in sheets) {
                    val reference = ReferenceSnyggTheme.compileFrom(SnyggStylesheet.fromJson(source).getOrThrow())
                    for (multiplier in listOf(1f, 1.35f)) {
                        val queries = queries(true)
                        val native = RenderThemeProbe.resolve(source, names.toTypedArray(), colors.toIntArray(),
                            floatArrayOf(2.625f, 1.1f, multiplier), queries.map { it.joinToString("\t") }.toTypedArray()).iterator()
                        for (q in queries) {
                            val actual = buildMap { repeat(native.next().toInt()) { put(native.next(), native.next()) } }
                            val expected = expected(reference, q, multiplier).properties.mapValues { (_, v) -> v.encoder().serialize(v).getOrThrow() }
                            assertEquals(expected, actual, "$path $q x$multiplier")
                            count++
                        }
                        check(!native.hasNext())
                    }
                    put(path, buildJsonArray {
                        for (q in queries(false)) add(buildJsonObject {
                            put("query", JsonArray(q.map(::JsonPrimitive)))
                            put("expected", buildJsonObject { expected(reference, q, 1f).properties.forEach { (k, v) ->
                                put(k, v.encoder().serialize(v).getOrThrow())
                            } })
                        })
                    })
                }
            })
        }
        val output = File(root, "build/stage3b/renderer-themes.json")
        checkNotNull(output.parentFile).mkdirs(); output.writeText(fixture.toString())
        val frozen = File(root, "lib/rust/crates/render/tests/themes.json")
        if (frozen.exists()) assertEquals(Json.parseToJsonElement(frozen.readText()), fixture)
        assertEquals(76032, count)
    }
}

internal object RenderThemeProbe {
    external fun resolve(json: String, names: Array<String>, colors: IntArray, scale: FloatArray, queries: Array<String>): Array<String>
}
