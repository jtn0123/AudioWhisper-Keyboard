package com.jtn0123.audiowhisper.settingsui

import androidx.compose.animation.core.CubicBezierEasing

/** Frozen from lib/compose/.../ScrollableModifiers.kt: initial 1850ms, fade 950ms. */
internal object ReferenceSettingsScrollbar {
    private val easing = CubicBezierEasing(1f, 0f, 0.82f, -0.13f)
    fun alpha(elapsedMs: Int, initial: Boolean): Float {
        val time = (elapsedMs - if (initial) 1850 else 0).coerceAtLeast(0)
        return (1f - easing.transform((time / 950f).coerceIn(0f, 1f))).coerceIn(0f, 1f)
    }
}
