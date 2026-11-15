package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import kotlin.test.Test
import kotlin.test.assertEquals

/** The explicit resource maps are exactly what the native screens ask for: none missing, none stale. */
class SettingsResourcesTest {
    @Test fun stringIdsAreExactlyTheNativeKeyList() {
        KeyboardRenderer(emptyArray()).use { n ->
            assertEquals(n.settings(18)[2].split("\n").toSortedSet(), settingsStringIds.keys.toSortedSet())
        }
    }
    @Test fun pluralIdsAreExactlyTheNativePluralRequests() {
        KeyboardRenderer(emptyArray()).use { n ->
            assertEquals(n.settings(16)[2].split("\n").map { it.substringBefore("|") }.toSortedSet(), settingsPluralIds.keys.toSortedSet())
        }
    }
}
