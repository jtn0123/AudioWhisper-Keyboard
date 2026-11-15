package com.jtn0123.audiowhisper.settingsui

import androidx.compose.material3.Typography
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.sp
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * The Material 3 tracking the Rust settings renderer hardcodes, read from the library the app
 * resolves: settings_ui/layout.rs `letter_spacing` (row summaries and dialog descriptions are
 * bodyMedium, titles the app's bodyLarge) and dialog_buttons.rs (TextButton is labelLarge).
 * A Material upgrade that moves a token fails here instead of in a picture diff.
 */
class SettingsTypographyTokensTest {
    @Test fun rustTrackingConstantsAreTheResolvedMaterialTokens() {
        val material = Typography()
        assertEquals(0.2.sp, material.bodyMedium.letterSpacing)
        assertEquals(14.sp, material.bodyMedium.fontSize)
        assertEquals(0.1.sp, material.labelLarge.letterSpacing)
        assertEquals(14.sp, material.labelLarge.fontSize)
        val app = dev.patrickgold.florisboard.app.apptheme.Typography
        assertEquals(TextUnit.Unspecified, app.bodyLarge.letterSpacing)
        assertEquals(16.sp, app.bodyLarge.fontSize)
    }
}
