package dev.patrickgold.florisboard.ime.core

import com.jtn0123.audiowhisper.settings.SettingsSubtypes

internal fun Subtype.withSupportedInput(): Subtype {
    val plan = SettingsSubtypes.plan("load", listOf(this), id)
    return if (plan.changed) plan.list.single() else this
}
