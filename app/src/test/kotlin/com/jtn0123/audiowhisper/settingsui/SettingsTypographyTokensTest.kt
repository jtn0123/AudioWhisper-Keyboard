package com.jtn0123.audiowhisper.settingsui

import androidx.compose.material3.Typography
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.sp
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * The Material 3 tracking the Rust settings renderer hardcodes, read from the library the app
 * resolves: settings_ui/layout.rs `letter_spacing` (row summaries and dialog descriptions are
 * bodyMedium, titles the app's bodyLarge), dialog_buttons.rs (TextButton is labelLarge) and
 * cards.rs (FlorisSimpleCard's titleMedium and titleSmall).
 * A Material upgrade that moves a token fails here instead of in a picture diff.
 */
class SettingsTypographyTokensTest {
    @Test fun rustTrackingConstantsAreTheResolvedMaterialTokens() {
        val material = Typography()
        assertEquals(0.2.sp, material.bodyMedium.letterSpacing)
        assertEquals(14.sp, material.bodyMedium.fontSize)
        assertEquals(0.1.sp, material.labelLarge.letterSpacing)
        assertEquals(14.sp, material.labelLarge.fontSize)
        assertEquals(listOf(16.sp, 24.sp, 0.2.sp), material.titleMedium.let { listOf(it.fontSize, it.lineHeight, it.letterSpacing) })
        assertEquals(listOf(14.sp, 20.sp, 0.1.sp), material.titleSmall.let { listOf(it.fontSize, it.lineHeight, it.letterSpacing) })
        val app = dev.patrickgold.florisboard.app.apptheme.Typography
        assertEquals(TextUnit.Unspecified, app.bodyLarge.letterSpacing)
        assertEquals(16.sp, app.bodyLarge.fontSize)
    }
}
