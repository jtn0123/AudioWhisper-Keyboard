package com.jtn0123.audiowhisper.render

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyboard
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import dev.patrickgold.florisboard.nlpManager
import org.florisboard.lib.snygg.SnyggSelector
import org.florisboard.lib.snygg.ui.rememberSnyggThemeQuery
import org.florisboard.lib.snygg.value.SnyggDpSizeValue

/** Milestone A: bundled theme values still resolve through Snygg; only plain values cross JNI. */
@Composable
fun RustKeyboardLayout(evaluator: ComputingEvaluator) = with(LocalDensity.current) {
    val context = LocalContext.current
    val prefs by FlorisPreferenceStore
    val nlp by context.nlpManager()
    val candidates by nlp.activeCandidatesFlow.collectAsState()
    val keyboard = evaluator.keyboard as TextKeyboard
    val keys = keyboard.keys().asSequence().toList()
    val window = rememberSnyggThemeQuery("window")
    val candidate = rememberSnyggThemeQuery(FlorisImeUi.SmartbarCandidateWord.elementName)
    val popup = rememberSnyggThemeQuery(FlorisImeUi.KeyPopupBox.elementName)
    val focus = rememberSnyggThemeQuery(FlorisImeUi.KeyPopupElement.elementName, selector = SnyggSelector.FOCUS)
    val density = this
    val styles = keys.map { key ->
        val attributes = mapOf(FlorisImeUi.Attr.Code to key.computedData.code,
            FlorisImeUi.Attr.Mode to keyboard.mode.toString(),
            FlorisImeUi.Attr.ShiftState to evaluator.state.inputShiftState.toString())
        val normal = rememberSnyggThemeQuery(FlorisImeUi.Key.elementName, attributes,
            if (key.isEnabled) SnyggSelector.NONE else SnyggSelector.DISABLED)
        val pressed = rememberSnyggThemeQuery(FlorisImeUi.Key.elementName, attributes, SnyggSelector.PRESSED)
        val hint = rememberSnyggThemeQuery(FlorisImeUi.KeyHint.elementName, attributes, SnyggSelector.NONE)
        val hintPressed = rememberSnyggThemeQuery(FlorisImeUi.KeyHint.elementName, attributes, SnyggSelector.PRESSED)
        ResolvedKeyStyle(
            intArrayOf(normal.background(Color.Transparent).toArgb(), normal.foreground(Color.Black).toArgb(),
                pressed.background(Color.Transparent).toArgb(), pressed.foreground(Color.Black).toArgb(),
                hint.foreground(Color.Gray).toArgb(), hintPressed.foreground(Color.Gray).toArgb()),
            normal.fontSize(22.sp).toPx(),
            hint.fontSize(12.sp).toPx(),
            (normal.shadowElevation as? SnyggDpSizeValue)?.dp?.toPx() ?: 0f,
            radius = { size ->
                when (val outline = normal.shape().createOutline(size, LayoutDirection.Ltr, density)) {
                    is Outline.Rounded -> outline.roundRect.topLeftCornerRadius.x
                    else -> 0f
                }
            },
        )
    }
    val keyboardHeight = FlorisImeSizing.keyboardUiHeight()
    val stripHeight = FlorisImeSizing.smartbarHeight
    val rowHeight = FlorisImeSizing.keyboardRowBaseHeight.toPx()
    val marginH = prefs.keyboard.keySpacingHorizontal.get() * density.density
    val marginV = prefs.keyboard.keySpacingVertical.get() * density.density
    val colors = intArrayOf(window.background(Color.White).toArgb(), candidate.foreground(Color.Black).toArgb(),
        popup.background(Color.LightGray).toArgb(), popup.foreground(Color.Black).toArgb(), focus.background(Color.Gray).toArgb())
    val heightPx = keyboardHeight.toPx()
    val stripPx = stripHeight.toPx()
    val candidateSize = candidate.fontSize(14.sp).toPx()

    AndroidView(
        modifier = Modifier.fillMaxWidth().height(keyboardHeight + stripHeight),
        factory = { RustKeyboardView(it) },
        onRelease = { it.release() },
        update = { view ->
            view.update(KeyboardSnapshot(keyboard, keys, styles, candidates.take(3), colors, candidateSize,
                heightPx, stripPx, rowHeight, marginH, marginV))
        },
    )
}

data class ResolvedKeyStyle(val colors: IntArray, val fontSize: Float, val hintSize: Float, val shadow: Float, val radius: (Size) -> Float)
