package com.jtn0123.audiowhisper.render

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.requiredHeight
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.absoluteOffset
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.layout.onSizeChanged
import com.jtn0123.audiowhisper.dictation.Dictation
import androidx.compose.ui.Alignment
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.viewinterop.AndroidView
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyboard
import dev.patrickgold.florisboard.ime.nlp.NlpInlineAutofill
import dev.patrickgold.florisboard.ime.smartbar.InlineSuggestionsUi
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.themeManager
import org.florisboard.lib.snygg.ui.rememberSnyggPalette

/** Android supplies the active source and platform inputs. Rust compiles and resolves all drawing styles. */
@Composable
fun RustKeyboardLayout(evaluator: ComputingEvaluator) = with(LocalDensity.current) {
    val context = LocalContext.current
    val resources = LocalResources.current
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
    val preview by Dictation.preview.collectAsState()
    val inline by NlpInlineAutofill.suggestions.collectAsState()
    dev.patrickgold.florisboard.ime.smartbar.InlineSuggestionsStyleCache()
    LaunchedEffect(inline) { nlp.autoExpandCollapseSmartbarActions(null, inline) }
    val search = rendererSearch()
    val panels = rendererPanels()
    val watermark = rendererWatermark(evaluator)
    val baseSmartbar = rendererSmartbar(preview.isNotEmpty(), inline.isNotEmpty())
    val smartbar = if(search.active) baseSmartbar.copy(config=gifSearchBarConfig(baseSmartbar.config,FlorisImeSizing.smartbarHeight.toPx()),animating=false) else baseSmartbar
    var availableWidth by remember { mutableFloatStateOf(resources.displayMetrics.widthPixels.toFloat()) }
    val measurement = smartbar.measure(availableWidth)
    val stripHeight = measurement[0].toDp()
    val insetHeight = measurement[1].toDp()
    val rowHeight = FlorisImeSizing.keyboardRowBaseHeight.toPx()
    val marginH = prefs.keyboard.keySpacingHorizontal.get() * density
    val marginV = prefs.keyboard.keySpacingVertical.get() * density
    val heightPx = keyboardHeight.toPx()
    val stripPx = stripHeight.toPx()
    val maxSurfaceHeight = resources.displayMetrics.heightPixels - dev.patrickgold.florisboard.ime.keyboard.systemBarHeights()
    val surfaceHeight = if (panels.kind == 3) maxSurfaceHeight.toDp() else keyboardHeight + stripHeight
    Box(Modifier.fillMaxWidth().height(keyboardHeight + insetHeight).onSizeChanged { availableWidth = it.width.toFloat() }) {
        AndroidView(
            modifier = Modifier.fillMaxWidth().wrapContentHeight(Alignment.Bottom, unbounded = true).requiredHeight(surfaceHeight).align(Alignment.BottomStart),
            factory = { RustKeyboardView(it) },
            onRelease = { it.release() },
            update = { view ->
                view.update(KeyboardSnapshot(keyboard, keys, if (preview.isEmpty() && !search.active) candidates else emptyList(), theme,
                    evaluator.state.inputShiftState.toString(), heightPx, stripPx, rowHeight, marginH, marginV,
                    smartbar, preview.isNotEmpty(), panels, intArrayOf(evaluator.editorInfo.imeOptions.raw, evaluator.editorInfo.inputAttributes.raw), search, watermark, preview))
            },
        )
        // Kind 2 is the native reservation for Android's InlineContentViews. The AndroidView
        // above keeps its identity when modes or inline suggestions change.
        val inlineRegion = measurement.drop(8).chunked(6).firstOrNull { it[0] == 2f }
        if (inlineRegion != null && panels.kind == 0 && !search.active) {
            Box(Modifier.absoluteOffset(inlineRegion[2].toDp(),
                (inlineRegion[3] - measurement[0] + measurement[1]).toDp())
                .width((inlineRegion[4] - inlineRegion[2]).toDp())
                .height((inlineRegion[5] - inlineRegion[3]).toDp())) {
                InlineSuggestionsUi(inline)
            }
        }

    }
}

data class RendererTheme(val json: String, val paletteNames: List<String>, val paletteColors: List<Int>,
    val density: Float, val fontScale: Float, val fontMultiplier: Float, val mode: String = "characters", val shift: String = "unshifted")
