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

package dev.patrickgold.florisboard.ime.clipboard

import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.outlined.Backspace
import androidx.compose.material.icons.filled.DeleteSweep
import androidx.compose.material.icons.filled.FilterList
import androidx.compose.material.icons.filled.FilterListOff
import androidx.compose.material.icons.filled.ToggleOff
import androidx.compose.material.icons.filled.ToggleOn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.media.KeyboardLikeButton
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import dev.patrickgold.florisboard.keyboardManager
import kotlinx.coroutines.launch
import org.florisboard.lib.compose.autoMirrorForRtl
import org.florisboard.lib.compose.stringRes
import org.florisboard.lib.snygg.ui.SnyggIcon
import org.florisboard.lib.snygg.ui.SnyggIconButton
import org.florisboard.lib.snygg.ui.SnyggRow
import org.florisboard.lib.snygg.ui.SnyggText

@Composable
internal fun ClipboardHeaderRow(
    historyEnabled: Boolean, deviceLocked: Boolean, popupSurfaceActive: Boolean,
    hasFilteredHistory: Boolean, hasHistory: Boolean, isFilterRowShown: Boolean,
    onClear: () -> Unit, onToggleFilter: () -> Unit,
) {
    val prefs by FlorisPreferenceStore
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    val keyboardManager by context.keyboardManager()
    SnyggRow(elementName = FlorisImeUi.ClipboardHeader.elementName,
        modifier = Modifier
            .fillMaxWidth()
            .height(FlorisImeSizing.smartbarHeight),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        val sizeModifier = Modifier
            .sizeIn(maxHeight = FlorisImeSizing.smartbarHeight)
            .aspectRatio(1f)
        SnyggIconButton(
            elementName = FlorisImeUi.ClipboardHeaderButton.elementName,
            onClick = { keyboardManager.activeState.imeUiMode = ImeUiMode.TEXT },
            modifier = sizeModifier,
        ) {
            SnyggIcon(
                imageVector = Icons.AutoMirrored.Filled.ArrowBack,
            )
        }
        SnyggText(
            elementName = FlorisImeUi.ClipboardHeaderText.elementName,
            modifier = Modifier.weight(1f),
            text = stringRes(R.string.clipboard__header_title),
        )
        SnyggIconButton(
            elementName = FlorisImeUi.ClipboardHeaderButton.elementName,
            onClick = { scope.launch { prefs.clipboard.historyEnabled.set(!historyEnabled) } },
            modifier = sizeModifier.autoMirrorForRtl(),
            enabled = !deviceLocked && !popupSurfaceActive,
        ) {
            SnyggIcon(
                imageVector = if (historyEnabled) {
                    Icons.Default.ToggleOn
                } else {
                    Icons.Default.ToggleOff
                },
            )
        }
        SnyggIconButton(
            elementName = FlorisImeUi.ClipboardHeaderButton.elementName,
            onClick = { onClear() },
            modifier = sizeModifier.autoMirrorForRtl(),
            enabled = !deviceLocked && historyEnabled && hasFilteredHistory && !popupSurfaceActive,
        ) {
            SnyggIcon(
                imageVector = Icons.Default.DeleteSweep,
            )
        }
        SnyggIconButton(
            elementName = FlorisImeUi.ClipboardHeaderButton.elementName,
            onClick = { onToggleFilter() },
            modifier = sizeModifier,
            enabled = !deviceLocked && historyEnabled && hasHistory && !popupSurfaceActive,
        ) {
            SnyggIcon(
                imageVector = if (!isFilterRowShown) {
                    Icons.Default.FilterList
                } else {
                    Icons.Default.FilterListOff
                },
            )
        }
        KeyboardLikeButton(
            modifier = sizeModifier,
            inputEventDispatcher = keyboardManager.inputEventDispatcher,
            keyData = TextKeyData.DELETE,
            elementName = FlorisImeUi.ClipboardHeaderButton.elementName,
        ) {
            SnyggIcon(imageVector = Icons.AutoMirrored.Outlined.Backspace)
        }
    }
}
