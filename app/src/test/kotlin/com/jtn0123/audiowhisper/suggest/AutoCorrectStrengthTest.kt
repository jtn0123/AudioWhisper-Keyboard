package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.*
import kotlin.test.Test
import kotlin.test.assertEquals

/** A strength crosses JNI as its ordinal, so Kotlin's order must be Rust's, and the setting's. */
class AutoCorrectStrengthTest {
    @Test
    fun theEnumIsRustsStrengthsInRustsOrder() {
        assertEquals(SuggestRules.strengthNames, AutoCorrectStrength.entries.map { it.name })
        assertEquals(AutoCorrectStrength.NORMAL.ordinal, SuggestRules.defaultStrength)
    }

    @Test
    fun theSettingListsEveryStrengthAndDefaultsToNormal() {
        assertEquals(AutoCorrectStrength.NORMAL.name, SettingsRules.default("correction__auto_correct"))
        val choices = SettingsRules.catalogue.getValue("correction__auto_correct")[SettingsCatalogue.CHOICES]
        assertEquals(AutoCorrectStrength.entries.map { it.name }, choices.split('|'))
    }
}
