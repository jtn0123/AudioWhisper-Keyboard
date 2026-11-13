package com.jtn0123.audiowhisper.render

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.viewinterop.AndroidView
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyboard
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.themeManager
import org.florisboard.lib.snygg.ui.rememberSnyggPalette

/** Android supplies the active source and platform inputs. Rust compiles and resolves all drawing styles. */
@Composable
fun RustKeyboardLayout(evaluator: ComputingEvaluator) = with(LocalDensity.current) {
    val context = LocalContext.current
    val prefs by FlorisPreferenceStore
    val nlp by context.nlpManager()
    val themeManager by context.themeManager()
    val activeTheme by themeManager.activeThemeInfo.collectAsState()
    val candidates by nlp.activeCandidatesFlow.collectAsState()
    val keyboard = evaluator.keyboard as TextKeyboard
    val keys = keyboard.keys().asSequence().toList()
    val palette = rememberSnyggPalette()
    val multiplier = prefs.keyboard.fontSizeMultiplier()
    val theme = remember(activeTheme, palette, density, fontScale, multiplier) {
        RendererTheme(activeTheme.stylesheetJson, palette.names, palette.colors, density, fontScale, multiplier)
    }
    val keyboardHeight = FlorisImeSizing.keyboardUiHeight()
    val stripHeight = FlorisImeSizing.smartbarHeight
    val rowHeight = FlorisImeSizing.keyboardRowBaseHeight.toPx()
    val marginH = prefs.keyboard.keySpacingHorizontal.get() * density
    val marginV = prefs.keyboard.keySpacingVertical.get() * density
    val heightPx = keyboardHeight.toPx()
    val stripPx = stripHeight.toPx()
    AndroidView(
        modifier = Modifier.fillMaxWidth().height(keyboardHeight + stripHeight),
        factory = { RustKeyboardView(it) },
        onRelease = { it.release() },
        update = { view ->
            view.update(KeyboardSnapshot(keyboard, keys, candidates.take(3), theme,
                evaluator.state.inputShiftState.toString(), heightPx, stripPx, rowHeight, marginH, marginV))
        },
    )
}

data class RendererTheme(val json: String, val paletteNames: List<String>, val paletteColors: List<Int>,
    val density: Float, val fontScale: Float, val fontMultiplier: Float)
