package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** The explicit resource maps are exactly what the native screens ask for: none missing, none stale. */
class SettingsResourcesTest {
    @Test fun stringIdsAreExactlyTheNativeKeyList() {
        KeyboardRenderer(emptyArray()).use { n ->
            assertEquals(n.settings(SettingsUiOp.STRING_KEYS)[2].split("\n").toSortedSet(), settingsStringIds.keys.toSortedSet())
        }
    }
    @Test fun pluralIdsAreExactlyTheNativePluralRequests() {
        KeyboardRenderer(emptyArray()).use { n ->
            assertEquals(n.settings(SettingsUiOp.PLURALS)[2].split("\n").map { it.substringBefore("|") }.toSortedSet(), settingsPluralIds.keys.toSortedSet())
        }
    }
    /** Every icon a native screen names has a vector in SettingsIcons; a missing one draws nothing. */
    @Test fun everyIconTheNativeScreensNameIsMapped() {
        // A call, on one line or as rustfmt splits it, a field (`icon: "add"`), or a list of names in a constant whose
        // name ends in ICONS.
        val named = Regex("""\.icon\(\s*"(\w+)"\s*\)|paint_icon\(\s*p,\s*"(\w+)"|icon: "(\w+)"|ICONS: \[&str; \d+\] = \[([^\]]*)\]""")
        val rust = File("../lib/rust/crates/render/src/settings_ui").walk().filter { it.extension == "rs" }
            .flatMap { file -> named.findAll(file.readText()).flatMap { m -> m.groupValues.drop(1).first { it.isNotEmpty() }.split(",") } }
            .map { it.trim().trim('"') }.filter { it.isNotEmpty() }.toSortedSet()
        val mapped = Regex(""""@icon:(\w+)"""")
            .findAll(File("src/main/kotlin/com/jtn0123/audiowhisper/settingsui/SettingsIcons.kt").readText())
            .map { it.groupValues[1] }.toSet()
        assertTrue(rust.containsAll(listOf("brightness_auto", "color_lens", "shop", "add", "arrow_upward", "input", "pageview")), rust.toString())
        assertEquals(emptySet(), rust - mapped)
    }
}
