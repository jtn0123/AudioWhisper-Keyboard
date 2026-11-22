package com.jtn0123.audiowhisper.settingsui

import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import org.florisboard.lib.snygg.Snygg
import org.florisboard.lib.snygg.SnyggRule
import org.florisboard.lib.snygg.SnyggStylesheetEditor

/**
 * Frozen Compose theme editor decisions the match tests still hold Rust to, transcribed from
 * ThemeEditorComponents.kt (`CustomRuleComparator`) and ThemeTranslations.kt (`PropertyNameMap`)
 * as they stood when compose-removal step 9 deleted them. No production code.
 */
internal object ReferenceThemeEditor {
    /** The order ThemeEditorScreen's editor kept rules in. */
    object CustomRuleComparator : Comparator<SnyggRule> {
        override fun compare(a: SnyggRule, b: SnyggRule): Int =
            SnyggStylesheetEditor.compareRules(a, b, FlorisImeUi.elementNames)
    }

    /** The properties and variables `translatePropertyName` named, with their strings. */
    val PropertyNameMap = mapOf(
        Snygg.Background to R.string.snygg__property_name__background,
        Snygg.Foreground to R.string.snygg__property_name__foreground,
        Snygg.BackgroundImage to R.string.snygg__property_name__background_image,
        Snygg.ContentScale to R.string.snygg__property_name__content_scale,
        Snygg.BorderColor to R.string.snygg__property_name__border_color,
        Snygg.BorderStyle to R.string.snygg__property_name__border_style,
        Snygg.BorderWidth to R.string.snygg__property_name__border_width,
        Snygg.FontFamily to R.string.snygg__property_name__font_family,
        Snygg.FontSize to R.string.snygg__property_name__font_size,
        Snygg.FontStyle to R.string.snygg__property_name__font_style,
        Snygg.FontWeight to R.string.snygg__property_name__font_weight,
        Snygg.LetterSpacing to R.string.snygg__property_name__letter_spacing,
        Snygg.LineHeight to R.string.snygg__property_name__line_height,
        Snygg.Margin to R.string.snygg__property_name__margin,
        Snygg.Padding to R.string.snygg__property_name__padding,
        Snygg.ShadowColor to R.string.snygg__property_name__shadow_color,
        Snygg.ShadowElevation to R.string.snygg__property_name__shadow_elevation,
        Snygg.Shape to R.string.snygg__property_name__shape,
        Snygg.Clip to R.string.snygg__property_name__clip,
        Snygg.Src to R.string.snygg__property_name__src,
        Snygg.TextAlign to R.string.snygg__property_name__text_align,
        Snygg.TextDecorationLine to R.string.snygg__property_name__text_decoration_line,
        Snygg.TextMaxLines to R.string.snygg__property_name__text_max_lines,
        Snygg.TextOverflow to R.string.snygg__property_name__text_overflow,
        "--primary" to R.string.snygg__property_name__var_primary,
        "--primary-variant" to R.string.snygg__property_name__var_primary_variant,
        "--secondary" to R.string.snygg__property_name__var_secondary,
        "--secondary-variant" to R.string.snygg__property_name__var_secondary_variant,
        "--background" to R.string.snygg__property_name__var_background,
        "--surface" to R.string.snygg__property_name__var_surface,
        "--surface-variant" to R.string.snygg__property_name__var_surface_variant,
        "--on-primary" to R.string.snygg__property_name__var_on_primary,
        "--on-secondary" to R.string.snygg__property_name__var_on_secondary,
        "--on-background" to R.string.snygg__property_name__var_on_background,
        "--on-surface" to R.string.snygg__property_name__var_on_surface,
        "--on-surface-variant" to R.string.snygg__property_name__var_on_surface_variant,
        "--shape" to R.string.snygg__property_name__var_shape,
        "--shape-variant" to R.string.snygg__property_name__var_shape_variant,
    )
}
