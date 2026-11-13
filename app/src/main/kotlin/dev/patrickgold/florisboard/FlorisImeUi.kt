/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInteropFilter
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.clipboard.ClipboardInputLayout
import dev.patrickgold.florisboard.ime.input.LocalInputFeedbackController
import dev.patrickgold.florisboard.ime.keyboard.ProvideKeyboardRowBaseHeight
import dev.patrickgold.florisboard.ime.media.MediaInputLayout
import dev.patrickgold.florisboard.ime.onehanded.OneHandedMode
import dev.patrickgold.florisboard.ime.onehanded.OneHandedPanel
import dev.patrickgold.florisboard.ime.text.TextInputLayout
import dev.patrickgold.florisboard.ime.theme.FlorisImeTheme
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import dev.patrickgold.florisboard.lib.compose.SystemUiIme
import dev.patrickgold.florisboard.lib.observeAsTransformingState
import dev.patrickgold.jetpref.datastore.model.observeAsState
import org.florisboard.lib.android.isOrientationLandscape
import org.florisboard.lib.android.isOrientationPortrait
import org.florisboard.lib.compose.ProvideLocalizedResources
import org.florisboard.lib.snygg.ui.SnyggBox
import org.florisboard.lib.snygg.ui.SnyggSurfaceView

@Composable
internal fun FlorisImeService.ImeUiWrapper() {
    ProvideLocalizedResources(
        resourcesContext,
        appName = R.string.floris_app_name,
    ) {
        ProvideKeyboardRowBaseHeight {
            CompositionLocalProvider(LocalInputFeedbackController provides inputFeedbackController) {
                FlorisImeTheme {
                    // Do not apply system bar padding here yet, we want to draw it ourselves
                    Column(modifier = Modifier.fillMaxWidth()) {
                        // The transparent area lets the keyboard stay anchored at the bottom.
                        if (!(isFullscreenUiMode && isExtractUiShown)) {
                            Spacer(Modifier.fillMaxWidth().weight(1f))
                        }
                        ImeUi()
                        SystemUiIme()
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun FlorisImeService.ImeUi() {
    val state by keyboardManager.activeState.collectAsState()
    val evaluator by keyboardManager.activeEvaluator.collectAsState()
    val rustPrototype by prefs.keyboard.rustRendererPrototype.observeAsState()
    val typingGif by com.jtn0123.audiowhisper.gif.GifSearch.typing.collectAsState()
    val showingGif by com.jtn0123.audiowhisper.gif.GifSearch.showingGifs.collectAsState()
    val attributes = mapOf(
        FlorisImeUi.Attr.Mode to state.keyboardMode.toString(),
        FlorisImeUi.Attr.ShiftState to state.inputShiftState.toString(),
    )
    val layoutDirection = LocalLayoutDirection.current
    LaunchedEffect(layoutDirection) {
        keyboardManager.activeState.layoutDirection = layoutDirection
    }
    CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
        SnyggBox(
            elementName = FlorisImeUi.Window.elementName,
            attributes = attributes,
            modifier = Modifier
                .fillMaxWidth()
                .wrapContentHeight()
                .onGloballyPositioned { coords -> inputViewSize = coords.size },
            clickAndSemanticsModifier = Modifier
                // Do not remove below line or touch input may get stuck
                .pointerInteropFilter { false },
            allowClip = false,
        ) {
            // The background image is drawn in a SurfaceView, so it can sit under the inline-autofill chips.
            SnyggSurfaceView(
                elementName = FlorisImeUi.Window.elementName,
                attributes = attributes,
                modifier = Modifier.matchParentSize(),
            )
            val configuration = LocalConfiguration.current
            val bottomOffset by if (configuration.isOrientationPortrait()) {
                prefs.keyboard.bottomOffsetPortrait
            } else {
                prefs.keyboard.bottomOffsetLandscape
            }.observeAsTransformingState { it.dp }
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .wrapContentHeight()
                    // Apply system bars padding here (we already drew our keyboard background)
                    .safeDrawingPadding()
                    .padding(bottom = bottomOffset),
            ) {
                val oneHandedMode by prefs.keyboard.oneHandedMode.observeAsState()
                val oneHandedModeEnabled by prefs.keyboard.oneHandedModeEnabled.observeAsState()
                val oneHandedModeScaleFactor by prefs.keyboard.oneHandedModeScaleFactor.observeAsState()
                val keyboardWeight = when {
                    !oneHandedModeEnabled || configuration.isOrientationLandscape() -> 1f
                    else -> oneHandedModeScaleFactor / 100f
                }
                if (oneHandedModeEnabled && oneHandedMode == OneHandedMode.END && configuration.isOrientationPortrait()) {
                    OneHandedPanel(
                        panelSide = OneHandedMode.START,
                        weight = 1f - keyboardWeight,
                    )
                }
                CompositionLocalProvider(LocalLayoutDirection provides layoutDirection) {
                    Box(
                        modifier = Modifier
                            .weight(keyboardWeight)
                            .wrapContentHeight(),
                    ) {
                        if (rustPrototype && !typingGif && !showingGif && state.imeUiMode != ImeUiMode.CLIPBOARD) {
                            com.jtn0123.audiowhisper.render.RustKeyboardLayout(evaluator)
                        } else when (state.imeUiMode) {
                            ImeUiMode.TEXT -> TextInputLayout()
                            ImeUiMode.MEDIA -> MediaInputLayout()
                            ImeUiMode.CLIPBOARD -> ClipboardInputLayout()
                        }
                    }
                }
                if (oneHandedModeEnabled && oneHandedMode == OneHandedMode.START && configuration.isOrientationPortrait()) {
                    OneHandedPanel(
                        panelSide = OneHandedMode.END,
                        weight = 1f - keyboardWeight,
                    )
                }
            }
        }
    }
}
