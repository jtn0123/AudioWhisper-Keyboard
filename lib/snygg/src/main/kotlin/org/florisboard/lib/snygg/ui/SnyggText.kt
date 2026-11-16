/*
 * Copyright (C) 2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package org.florisboard.lib.snygg.ui

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextGeometricTransform
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.unit.isSpecified
import org.florisboard.lib.snygg.SnyggQueryAttributes
import org.florisboard.lib.snygg.SnyggSelector
import org.florisboard.lib.snygg.SnyggStylesheet

/**
 * Simple text composable, which displays the given [text].
 *
 * This composable infers its style from the current [SnyggTheme][org.florisboard.lib.snygg.SnyggTheme], which is
 * required to be provided by [ProvideSnyggTheme].
 *
 * @param elementName The name of this element. If `null` the style will be inherited from the parent element.
 * @param attributes The attributes of the element used to refine the query.
 * @param selector A specific SnyggSelector to query the style for.
 * @param modifier The modifier to be applied to the Text.
 * @param text The text of the element.
 * @param maxLines Overrides the style's line limit when not `null` (AudioWhisper).
 * @param overflow Overrides the style's overflow when not `null` (AudioWhisper).
 * @param fit Fits the text to its width when not `null` (AudioWhisper): given the width and a measure of
 *  the text's unsqueezed width at a font scale, returns its squeeze (text scale X, 1 for none) and
 *  font scale.
 *
 * @since 0.5.0-alpha01
 *
 * @see [Text]
 */
@Composable
fun SnyggText(
    modifier: Modifier = Modifier,
    elementName: String? = null,
    attributes: SnyggQueryAttributes = emptyMap(),
    selector: SnyggSelector? = null,
    text: String,
    maxLines: Int? = null,
    overflow: TextOverflow? = null,
    fit: SnyggTextFit? = null,
) {
    ProvideSnyggStyle(elementName, attributes, selector) { style ->
        val styled = modifier
            .snyggMargin(style)
            .snyggShadow(style)
            .snyggBorder(style)
            .snyggBackground(style, allowClip = false)
            .snyggPadding(style)
        if (fit != null) {
            val textStyle = TextStyle(
                color = style.foreground(),
                fontSize = style.fontSize(),
                fontStyle = style.fontStyle(),
                fontWeight = style.fontWeight(),
                fontFamily = style.fontFamily(LocalSnyggPreloadedCustomFontFamilies.current),
                letterSpacing = style.letterSpacing(),
                lineHeight = style.lineHeight(),
                textAlign = style.textAlign() ?: TextAlign.Unspecified,
                textDecoration = style.textDecorationLine(),
            )
            FittedText(styled, text, LocalTextStyle.current.merge(textStyle), fit,
                maxLines ?: style.textMaxLines(), overflow ?: style.textOverflow())
        } else {
            Text(
                modifier = styled,
                text = text,
                color = style.foreground(),
                fontSize = style.fontSize(),
                fontStyle = style.fontStyle(),
                fontWeight = style.fontWeight(),
                fontFamily = style.fontFamily(LocalSnyggPreloadedCustomFontFamilies.current),
                letterSpacing = style.letterSpacing(),
                lineHeight = style.lineHeight(),
                textAlign = style.textAlign(),
                textDecoration = style.textDecorationLine(),
                maxLines = maxLines ?: style.textMaxLines(),
                overflow = overflow ?: style.textOverflow(),
            )
        }
    }
}

/** AudioWhisper: how [SnyggText] fits a text to [available] pixels; see its `fit` parameter. */
fun interface SnyggTextFit {
    /** Returns (squeeze, font scale); [measure] gives the unsqueezed width at a font scale. */
    fun fit(available: Float, measure: (scale: Float) -> Float): Pair<Float, Float>
}

@Composable
private fun FittedText(
    modifier: Modifier,
    text: String,
    style: TextStyle,
    fit: SnyggTextFit,
    maxLines: Int,
    overflow: TextOverflow,
) {
    val measurer = rememberTextMeasurer()
    BoxWithConstraints(modifier) {
        val size = style.fontSize
        val (squeeze, scale) = if (constraints.hasBoundedWidth && size.isSpecified) {
            fit.fit(constraints.maxWidth.toFloat()) { scale ->
                measurer.measure(text, style.copy(fontSize = size * scale), softWrap = false, maxLines = 1)
                    .multiParagraph.intrinsics.maxIntrinsicWidth
            }
        } else {
            1f to 1f
        }
        Text(
            text = text,
            style = style.copy(
                fontSize = if (size.isSpecified) size * scale else size,
                textGeometricTransform = TextGeometricTransform(scaleX = squeeze),
            ),
            maxLines = maxLines,
            overflow = overflow,
        )
    }
}

@Preview
@Composable
private fun SimpleSnyggText() {
    val stylesheet = SnyggStylesheet.v2 {
        "preview-column" {
            fontSize = fontSize(20.sp)
            foreground = rgbaColor(0, 0, 255)
        }
        "preview-text" {
            background = rgbaColor(255, 255, 255)
            foreground = inherit()
            borderColor = rgbaColor(0, 0, 255)
            borderWidth = size(1.dp)
            shadowElevation = size(6.dp)
            shadowColor = rgbaColor(0, 255, 0)
            margin = padding(16.dp)
            padding = padding(6.dp)
        }
        "preview-text"("attr" to listOf(1)) {
            foreground = rgbaColor(255, 0, 0)
            borderWidth = size(0.dp)
            fontSize = fontSize(10.sp)
            fontStyle = fontStyle(FontStyle.Italic)
            fontWeight = fontWeight(FontWeight.Bold)
            letterSpacing = fontSize(4.sp)
            textDecorationLine = textDecorationLine(TextDecoration.LineThrough)
        }
        "preview-text"("long" to listOf(1)) {
            fontFamily = genericFontFamily(FontFamily.Serif)
            fontSize = fontSize(10.sp)
            textMaxLines = textMaxLines(1)
            textOverflow = textOverflow(TextOverflow.Ellipsis)
        }
    }
    val theme = rememberSnyggTheme(stylesheet)

    ProvideSnyggTheme(theme) {
        SnyggColumn(elementName = "preview-column", modifier = Modifier.widthIn(max = 150.dp)) {
            SnyggText(elementName = "preview-text", text = "black text")
            SnyggText(elementName = "preview-text", attributes = mapOf("attr" to 1), text = "red text")
            SnyggText(elementName = "preview-text", attributes = mapOf("long" to 1),
                text = "this is a very long paragraph that will definitely not fit")
        }
    }
}
