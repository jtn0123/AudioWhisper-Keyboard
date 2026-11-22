package org.florisboard.lib.snygg

import androidx.compose.material3.ColorScheme
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.sp
import com.jtn0123.audiowhisper.rust.ImeTheme
import com.jtn0123.audiowhisper.rust.ImeThemeChip
import com.jtn0123.audiowhisper.rust.ImeThemeInline
import com.jtn0123.audiowhisper.rust.ImeThemeRule
import com.materialkolor.Contrast
import com.materialkolor.PaletteStyle
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import org.florisboard.lib.color.ColorPalette
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.color.getColor
import org.florisboard.lib.snygg.ui.materialYouScheme
import org.florisboard.lib.snygg.value.SnyggSpSizeValue

/**
 * Seeded traces of the inline autofill chips' colours: main queried `inline-autofill-chip` in
 * Compose, inside the keyboard's window, below the theme's root, and styled the chips with its
 * static background (else white) and foreground (else black). Here the frozen Kotlin query
 * ([ReferenceSnyggTheme]) walks that chain and Rust (crates/imetheme's `inline`) answers from the
 * stylesheet's JSON, as the theme manager now asks, over the bundled stylesheets and synthetic
 * ones, every keyboard mode and shift state, and several palettes. They must match exactly.
 */
class InlineChipMatchTest {
    private val names = ColorPalette.colorNames
    private val initial = SnyggSinglePropertySet(mapOf("font-size" to SnyggSpSizeValue(14.sp)))
    private val palettes: List<Pair<ColorScheme, ColorScheme>> = listOf(yellowLightScheme to yellowDarkScheme) +
        listOf(0xFF6750A4, 0xFF006A6A, 0xFFB3261E, 0xFF000000).flatMap { primary ->
            listOf(MaterialYouFlags(), MaterialYouFlags(PaletteStyle.Vibrant, Contrast.High)).map { flags ->
                materialYouScheme(Color(primary), false, flags) to materialYouScheme(Color(primary), true, flags)
            }
        }

    @Test fun mainsChipQueryAndRustGiveTheSameColours() {
        val bundled = SnyggStylesheetMatchTest().sheets().map { it.second }
            .mapNotNull { SnyggStylesheet.fromJson(it).getOrNull() }
        assertTrue(bundled.size >= 12)
        for (seed in SEEDS) {
            val random = Random(seed)
            val seen = mutableMapOf<String, Int>()
            var rejected = 0
            val made = generateSequence { synthetic(random) }
                .mapNotNull { source -> SnyggStylesheet.fromJson(source).getOrNull().also { if (it == null) rejected++ }?.let { source to it } }
                .take(SYNTHETIC).toList()
            val sources = bundled.map { "" } + made.map { it.first }
            val pool = bundled + made.map { it.second }
            val compiled = pool.map { ReferenceSnyggTheme.compileFrom(it) to it.toJson().getOrThrow() }
            assertTrue(made.count { "[mode=" in it.first || "[shiftstate=" in it.first } > SYNTHETIC / 2, "seed $seed: few attributes")
            repeat(STATES) {
                val at = if (random.nextInt(10) < 3) random.nextInt(bundled.size) else random.nextInt(bundled.size, compiled.size)
                val (theme, json) = compiled[at]
                val (mode, shift) = MODES.random(random) to SHIFTS.random(random)
                val (light, dark) = palettes.random(random)
                val expected = expected(theme, mode, shift, light, dark, random.nextFloat() + 0.5f)
                assertEquals(expected, actual(json, mode, shift, light, dark), "seed $seed sheet $at $mode $shift")
                fun see(what: String) { seen[what] = (seen[what] ?: 0) + 1 }
                see(if (at < bundled.size) "bundled" else "synthetic")
                see(if (expected.first == Color.White.toArgb()) "default background" else "themed background")
                see(if (expected.second == Color.Black.toArgb()) "default foreground" else "themed foreground")
                if ("`$mode`" in sources[at] || "`$shift`" in sources[at]) see("names the state")
            }
            println("Inline chip trace seed $seed: $STATES states identical over ${pool.size} stylesheets " +
                "($rejected synthetic ones rejected by the parser); ${seen.toSortedMap()}")
            assertTrue(seen.size == 7 && seen.values.all { it > 1000 }, "seed $seed covered $seen")
        }
    }

    private fun expected(theme: ReferenceSnyggTheme, mode: String, shift: String, light: ColorScheme, dark: ColorScheme,
        multiplier: Float): Pair<Int, Int> {
        val attributes = mapOf("mode" to mode, "shiftstate" to shift)
        var style = initial
        for ((element, query) in listOf("root" to attributes, "window" to attributes, "inline-autofill-chip" to emptyMap())) {
            style = theme.query(element, query, SnyggSelector.NONE, style, light, dark, multiplier)
        }
        return style.background(Color.White).toArgb() to style.foreground(Color.Black).toArgb()
    }

    private fun actual(json: String, mode: String, shift: String, light: ColorScheme, dark: ColorScheme): Pair<Int, Int> {
        val row = ImeThemeInline.row(ImeThemeInline.STYLESHEET to json, ImeThemeInline.MODE to mode, ImeThemeInline.SHIFT to shift)
        val colors = names.map { light.getColor(it).toArgb().toString() } + names.map { dark.getColor(it).toArgb().toString() }
        val chip = ImeTheme.rules(ImeThemeRule.INLINE, *row, *names.toTypedArray(), *colors.toTypedArray())
        return chip[ImeThemeChip.BACKGROUND].toInt() to chip[ImeThemeChip.FOREGROUND].toInt()
    }

    /** A stylesheet styling the chip, the window and the root with colours of every kind, under attributes and selectors. */
    private fun synthetic(random: Random): String {
        fun color(): String = when (random.nextInt(8)) {
            0 -> "#%06x".format(random.nextInt(0x1000000))
            1 -> "#%08x".format(random.nextInt())
            2 -> "rgba(${random.nextInt(256)},${random.nextInt(256)},${random.nextInt(256)},${random.nextInt(11) / 10.0})"
            3 -> "transparent"
            4 -> "dynamic-light-color(${names.random(random)})"
            5 -> "dynamic-dark-color(${names.random(random)})"
            6 -> "var(--c${random.nextInt(3)})"
            else -> listOf("undefined", "inherit", "nonsense").random(random)
        }
        val rules = buildList {
            add("\"@defines\":{" + (0..2).joinToString(",") { "\"--c$it\":\"${color()}\"" } + "}")
            repeat(random.nextInt(1, 7)) {
                val element = ELEMENTS.random(random)
                val attribute = when (random.nextInt(4)) {
                    0 -> "[mode=`${MODES.random(random)}`]"
                    1 -> "[shiftstate=`${SHIFTS.random(random)}`]"
                    2 -> "[mode=`${MODES.random(random)}`,`${MODES.random(random)}`]"
                    else -> ""
                }
                val selector = if (random.nextInt(5) == 0) ":pressed" else ""
                val properties = listOf("background", "foreground").filter { random.nextInt(3) != 0 }
                    .joinToString(",") { "\"$it\":\"${color()}\"" }
                add("\"$element$attribute$selector\":{$properties}")
            }
        }.distinctBy { it.substringBefore("\":") }
        return "{\"\$schema\":\"${SnyggStylesheet.SCHEMA_V2}\"," + rules.joinToString(",") + "}"
    }

    private companion object {
        const val STATES = 50_000
        const val SYNTHETIC = 400
        val SEEDS = listOf(20261122L, 7L, 424242L)
        val ELEMENTS = listOf("root", "window", "inline-autofill-chip", "inline-autofill-chip", "inline-autofill-chip", "key")
        val MODES = listOf("characters", "edit", "symbols", "symbols2", "numeric", "numeric_advanced", "phone", "phone2")
        val SHIFTS = listOf("unshifted", "shifted_manual", "shifted_automatic", "caps_lock")
    }
}
