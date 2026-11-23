package com.jtn0123.audiowhisper.render

import androidx.compose.ui.graphics.Color
import com.materialkolor.Contrast
import com.materialkolor.PaletteStyle
import com.materialkolor.dynamiccolor.ColorSpec
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import org.florisboard.lib.color.MaterialYouFlags

/**
 * Material You palettes made in Rust (crates/palette, through `ImeThemeRule.PALETTE`) against main's, made by
 * materialkolor ([referenceSnyggPalette]): seeded accents under every palette style, contrast level and spec version
 * a theme's MaterialYouFlags can hold, light and dark. Every colour must match. The gate runs [SEEDS] accents;
 * `-PpaletteSeeds=50000` runs more (docs/phase3-compose-free-path.md records that run).
 */
class PaletteMatchTest {
    @Test fun rustMakesTheSamePaletteAsMaterialKolor() {
        val seeds = System.getProperty("palette.seeds")?.toInt() ?: SEEDS
        val random = Random(SEED)
        val accents = (SPECIALS + List(seeds - SPECIALS.size) { random.nextInt() or (0xFF shl 24) }).take(seeds)
        val flags = PaletteStyle.entries.flatMap { style ->
            Contrast.entries.flatMap { contrast -> ColorSpec.SpecVersion.entries.map { MaterialYouFlags(style, contrast, it) } }
        }
        val (schemes, exact) = AtomicLong() to AtomicLong()
        val differing = ConcurrentHashMap<String, Int>()
        accents.parallelStream().forEach { accent ->
            for (f in flags) {
                val rust = themePalette(accent, 0, f)
                val main = referenceSnyggPalette(Color(accent), f)
                assertEquals(main.names, rust.names)
                for ((half, dark) in listOf(0 to "light", 1 to "dark")) {
                    val at = half * main.names.size
                    val wrong = main.names.indices.filter { main.colors[at + it] != rust.colors[at + it] }
                    schemes.incrementAndGet()
                    if (wrong.isEmpty()) exact.incrementAndGet()
                    for (i in wrong) differing.merge("${f.paletteStyle}/${f.contrastLevel}/${f.specVersion} $dark ${main.names[i]}", 1, Int::plus)
                }
            }
        }
        println("Palette differential: ${accents.size} accents x ${flags.size} flag sets x light and dark: " +
            "$exact of $schemes schemes exact; differing roles $differing")
        assertEquals(emptyMap(), differing.toMap())
        assertEquals(accents.size.toLong() * flags.size * 2, exact.get())
    }

    private companion object {
        const val SEED = 20261123
        const val SEEDS = 200
        /** Black, white, greys, primaries, near-greys and the Material default, before the seeded ones. */
        val SPECIALS = listOf(0x000000, 0xFFFFFF, 0x808080, 0x7F7F7F, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0x00FFFF,
            0xFF00FF, 0x010101, 0xFEFEFE, 0x808081, 0x6750A4, 0x3F51B5, 0x00796B, 0xB3261E).map { it or (0xFF shl 24) }
    }
}
