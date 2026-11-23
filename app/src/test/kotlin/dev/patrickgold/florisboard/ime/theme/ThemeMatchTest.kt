package dev.patrickgold.florisboard.ime.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import com.jtn0123.audiowhisper.render.inlineChipColors
import com.jtn0123.audiowhisper.render.themePalette
import com.jtn0123.audiowhisper.rust.ImeTheme
import com.jtn0123.audiowhisper.rust.ImeThemeRule
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.snygg.SnyggStylesheet

/**
 * Seeded traces of theme updates, each run by main's theme manager (frozen in
 * ReferenceThemeManager.kt) and by Rust's (lib/rust/crates/imetheme, through ThemeUpdates) side by
 * side: the theme shown after every event, and what the renderer is handed for it, must match
 * exactly. See docs/phase3-theme-subtype.md.
 */
class ThemeMatchTest {
    @Test fun mainsThemeManagerAndRustShowTheSameThemes() {
        for (seed in SEEDS) {
            val trace = ThemeTrace(seed)
            trace.run(STATES)
            println("Theme trace seed $seed: $STATES states identical; ${trace.seen.toSortedMap()}")
            for (mode in ThemeMode.entries) assertTrue((trace.seen["mode $mode"] ?: 0) > 0, "seed $seed never set $mode")
            for (what in COVERED) assertTrue((trace.seen[what] ?: 0) > 0, "seed $seed never saw $what")
        }
    }

    @Test fun theBaseStyleRustHoldsIsMainsKotlinBaseStyle() {
        val base = ImeTheme.rules(ImeThemeRule.BASE).single()
        assertEquals(ReferenceFlorisImeThemeBaseStyle.toJson().getOrThrow(), base)
        assertEquals(ReferenceFlorisImeThemeBaseStyle, ThemeManager.ThemeInfo.DEFAULT.stylesheet)
        assertEquals(base, ThemeManager.ThemeInfo.DEFAULT.stylesheetJson)
        assertEquals(ReferenceThemeManager.DEFAULT, ThemeManager.ThemeInfo.DEFAULT)
    }

    /** The chips' adapter sends the stylesheet, the keyboard's state and the palette as Rust reads them. */
    @Test fun theChipsAreStyledFromTheWholePaletteInTheKeyboardsState() {
        val palette = themePalette(0xFF6750A4.toInt(), 0, MaterialYouFlags())
        val names = palette.names
        val sheet = """{"${'$'}schema":"${SnyggStylesheet.SCHEMA_V2}",
            "window":{"foreground":"dynamic-dark-color(secondary)"},
            "window[mode=`symbols`]":{"foreground":"#ff0000"},
            "inline-autofill-chip":{"background":"dynamic-light-color(primary)"}}"""
        val info = ThemeManager.ThemeInfo.DEFAULT.copy(stylesheet = SnyggStylesheet.fromJson(sheet).getOrThrow())
        val primary = palette.colors[names.indexOf("primary")]
        val secondary = palette.colors[names.size + names.indexOf("secondary")]
        assertEquals(primary to secondary, inlineChipColors(info, "characters", "unshifted", palette))
        assertEquals(primary to Color.Red.toArgb(), inlineChipColors(info, "symbols", "unshifted", palette))
        val unstyled = ThemeManager.ThemeInfo.DEFAULT.copy(stylesheet = SnyggStylesheet.fromJson(
            """{"${'$'}schema":"${SnyggStylesheet.SCHEMA_V2}","root[shiftstate=`caps_lock`]":{"foreground":"#00ff00"}}""").getOrThrow())
        assertEquals(Color.White.toArgb() to Color.Black.toArgb(), inlineChipColors(unstyled, "characters", "unshifted", palette))
        assertEquals(Color.White.toArgb() to Color.Green.toArgb(), inlineChipColors(unstyled, "characters", "caps_lock", palette))
        assertEquals(0xFF424242.toInt() to Color.White.toArgb(),
            inlineChipColors(ThemeManager.ThemeInfo.DEFAULT, "characters", "unshifted", palette))
    }

    private companion object {
        const val STATES = 50_000
        val SEEDS = listOf(20261122L, 7L, 424242L)
        val COVERED = listOf("load", "kept", "cached", "recreated", "preview id", "preview info", "clock", "keyboards", "file",
            "extension removed", "extension added", "extensions reordered", "source toggled", "extensions reset",
            "cancelled before the lock", "showing preview", "showing cancelled load", "showing unreadable stylesheet",
            "showing unparsable stylesheet", "showing base", "showing loaded", "system accent", "settings accent")
    }
}
