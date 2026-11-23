package com.jtn0123.audiowhisper.render

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.test.platform.app.InstrumentationRegistry
import com.materialkolor.Contrast
import com.materialkolor.PaletteStyle
import com.materialkolor.dynamiccolor.ColorSpec
import java.io.File
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong
import kotlin.random.Random
import org.florisboard.lib.color.ColorPalette
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.color.getColor
import org.florisboard.lib.snygg.ui.materialYouScheme
import org.junit.Test

/**
 * Rust's Material You palette (crates/palette, through [themePalette]) against materialkolor run on ART, as main's
 * keyboard made it on a device. PaletteMatchTest compares with a desktop JVM; ART's `Math` calls bionic's libm. The
 * crate ports bionic's `sin` and `cos`, but takes `pow`, `exp`, `cbrt`, `hypot` and `atan2` from fdlibm, as the
 * JVM's StrictMath does, so on ART those match only as far as this measures. scripts/palette-art.sh runs it in an app
 * built with -PpaletteFixture=true, whose palette-fixture.pro keeps the classes it calls by name, and reads the counts
 * it writes to the cache: the seeded accents are PaletteMatchTest's, under every style, contrast and spec, light and
 * dark.
 */
class PaletteArtDiff {
    @Test fun rustAgainstMaterialKolorOnArt() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val seeds = InstrumentationRegistry.getArguments().getString("seeds")?.toInt() ?: 1000
        val random = Random(20261123)
        val accents = List(seeds) { random.nextInt() or (0xFF shl 24) }
        val flags = PaletteStyle.entries.flatMap { style ->
            Contrast.entries.flatMap { contrast -> ColorSpec.SpecVersion.entries.map { MaterialYouFlags(style, contrast, it) } }
        }
        val names = ColorPalette.colorNames
        val (schemes, exact) = AtomicLong() to AtomicLong()
        val differing = ConcurrentHashMap<String, Int>()
        accents.parallelStream().forEach { accent ->
            for (f in flags) {
                val rust = themePalette(accent, 0, f)
                for ((half, dark) in listOf(0 to false, 1 to true)) {
                    val art = materialYouScheme(Color(accent), dark, f)
                    val wrong = names.indices.filter { rust.colors[half * names.size + it] != art.getColor(names[it]).toArgb() }
                    schemes.incrementAndGet()
                    if (wrong.isEmpty()) exact.incrementAndGet()
                    for (i in wrong) {
                        val mode = if (dark) "dark" else "light"
                        differing.merge("${f.paletteStyle}/${f.contrastLevel}/${f.specVersion}\t$mode\t${names[i]}", 1, Int::plus)
                    }
                }
            }
        }
        val report = StringBuilder("accents\t$seeds\nflag sets\t${flags.size}\nschemes\t$schemes\nexact\t$exact\n")
        for ((key, count) in differing.toSortedMap()) report.append("differs\t$key\t$count\n")
        File(instrumentation.targetContext.cacheDir, "palette-art.tsv").writeText(report.toString())
    }
}
