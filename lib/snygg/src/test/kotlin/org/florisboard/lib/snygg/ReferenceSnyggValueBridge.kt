package org.florisboard.lib.snygg

import org.florisboard.lib.snygg.reference.*
import org.florisboard.lib.snygg.value.*

internal val referenceValueEncoders = listOf(
    ReferenceSnyggInheritValue,
    ReferenceSnyggUndefinedValue,
    ReferenceSnyggYesValue,
    ReferenceSnyggNoValue,
    ReferenceSnyggDpSizeValue,
    ReferenceSnyggSpSizeValue,
    ReferenceSnyggPercentageSizeValue,
    ReferenceSnyggStaticColorValue,
    ReferenceSnyggDynamicLightColorValue,
    ReferenceSnyggDynamicDarkColorValue,
    ReferenceSnyggGenericFontFamilyValue,
    ReferenceSnyggCustomFontFamilyValue,
    ReferenceSnyggFontStyleValue,
    ReferenceSnyggFontWeightValue,
    ReferenceSnyggPaddingValue,
    ReferenceSnyggRectangleShapeValue,
    ReferenceSnyggCircleShapeValue,
    ReferenceSnyggCutCornerDpShapeValue,
    ReferenceSnyggCutCornerPercentShapeValue,
    ReferenceSnyggRoundedCornerDpShapeValue,
    ReferenceSnyggRoundedCornerPercentShapeValue,
    ReferenceSnyggTextAlignValue,
    ReferenceSnyggTextDecorationLineValue,
    ReferenceSnyggTextMaxLinesValue,
    ReferenceSnyggTextOverflowValue,
    ReferenceSnyggUriValue,
    ReferenceSnyggContentScaleValue,
    ReferenceSnyggDefinedVarValue,
)

/** Field-only bridge; reference results never pass through the production codec. */
internal fun ReferenceSnyggValue.production(): SnyggValue = when (this) {
    ReferenceSnyggInheritValue -> SnyggInheritValue
    ReferenceSnyggUndefinedValue -> SnyggUndefinedValue
    ReferenceSnyggYesValue -> SnyggYesValue
    ReferenceSnyggNoValue -> SnyggNoValue
    is ReferenceSnyggDpSizeValue -> SnyggDpSizeValue(dp)
    is ReferenceSnyggSpSizeValue -> SnyggSpSizeValue(sp)
    is ReferenceSnyggPercentageSizeValue -> SnyggPercentageSizeValue(percentage)
    is ReferenceSnyggStaticColorValue -> SnyggStaticColorValue(color)
    is ReferenceSnyggDynamicLightColorValue -> SnyggDynamicLightColorValue(colorName)
    is ReferenceSnyggDynamicDarkColorValue -> SnyggDynamicDarkColorValue(colorName)
    is ReferenceSnyggGenericFontFamilyValue -> SnyggGenericFontFamilyValue(fontFamily)
    is ReferenceSnyggCustomFontFamilyValue -> SnyggCustomFontFamilyValue(fontName)
    is ReferenceSnyggFontStyleValue -> SnyggFontStyleValue(fontStyle)
    is ReferenceSnyggFontWeightValue -> SnyggFontWeightValue(fontWeight)
    is ReferenceSnyggPaddingValue -> SnyggPaddingValue(values)
    is ReferenceSnyggRectangleShapeValue -> SnyggRectangleShapeValue(shape)
    is ReferenceSnyggCircleShapeValue -> SnyggCircleShapeValue(shape)
    is ReferenceSnyggCutCornerDpShapeValue -> SnyggCutCornerDpShapeValue(topStart, topEnd, bottomEnd, bottomStart)
    is ReferenceSnyggCutCornerPercentShapeValue -> SnyggCutCornerPercentShapeValue(topStart, topEnd, bottomEnd, bottomStart)
    is ReferenceSnyggRoundedCornerDpShapeValue -> SnyggRoundedCornerDpShapeValue(topStart, topEnd, bottomEnd, bottomStart)
    is ReferenceSnyggRoundedCornerPercentShapeValue -> SnyggRoundedCornerPercentShapeValue(topStart, topEnd, bottomEnd, bottomStart)
    is ReferenceSnyggTextAlignValue -> SnyggTextAlignValue(textAlign)
    is ReferenceSnyggTextDecorationLineValue -> SnyggTextDecorationLineValue(textDecoration)
    is ReferenceSnyggTextMaxLinesValue -> SnyggTextMaxLinesValue(maxLines)
    is ReferenceSnyggTextOverflowValue -> SnyggTextOverflowValue(textOverflow)
    is ReferenceSnyggUriValue -> SnyggUriValue(uri)
    is ReferenceSnyggContentScaleValue -> SnyggContentScaleValue(contentScale)
    is ReferenceSnyggDefinedVarValue -> SnyggDefinedVarValue(key)
    else -> error("Unknown reference value")
}
