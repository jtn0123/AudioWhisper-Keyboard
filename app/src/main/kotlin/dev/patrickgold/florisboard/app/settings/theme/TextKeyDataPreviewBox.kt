/*
 * Copyright (C) 2022-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.app.settings.theme

import androidx.compose.runtime.getValue

import android.icu.lang.UCharacter
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.requiredSize
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.DefaultComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.Key
import dev.patrickgold.florisboard.ime.keyboard.Keyboard
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.computeImageVector
import dev.patrickgold.florisboard.ime.keyboard.computeLabel
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData

import dev.patrickgold.jetpref.datastore.model.observeAsState

@Composable
internal fun TextKeyDataPreviewBox(
    textKeyData: TextKeyData?,
    modifier: Modifier = Modifier,
) {
    val data = textKeyData ?: TextKeyData.UNSPECIFIED

    val context = LocalContext.current
    val evaluator = remember(context) {
        object : ComputingEvaluator by DefaultComputingEvaluator {
            override val keyboard = object : Keyboard() {
                override val mode = KeyboardMode.NUMERIC_ADVANCED
                override fun getKeyForPos(pointerX: Float, pointerY: Float) = error("not implemented")
                override fun keys() = error("not implemented")
                override fun layout(
                    keyboardWidth: Float, keyboardHeight: Float, desiredKey: Key,
                    extendTouchBoundariesDownwards: Boolean,
                ) = error("not implemented")
            }

            override fun context() = context
        }
    }

    val label = remember(data) { evaluator.computeLabel(data) }
    val icon = remember(data) { evaluator.computeImageVector(data) }
    val displayName = remember(data) {
        if (data.code > 0) {
            UCharacter.getName(data.code) ?: UCharacter.getExtendedName(data.code)
        } else {
            data.label
        }
    }

    val prefs by dev.patrickgold.florisboard.app.FlorisPreferenceStore
    val prototype by prefs.keyboard.rustRendererPrototype.observeAsState()
    Row(modifier = modifier) {
        Box(
            modifier = Modifier
                .padding(end = 16.dp)
                .background(
                    color = if (prototype) Color.Transparent else MaterialTheme.colorScheme.onSurface.copy(alpha = 0.12f),
                    shape = MaterialTheme.shapes.medium,
                )
                .height(36.dp)
                .widthIn(min = 36.dp)
                .align(Alignment.CenterVertically),
            contentAlignment = Alignment.Center,
        ) {
            if (prototype) {
                com.jtn0123.audiowhisper.render.RustKeyPreview(data.code, data.type.toString(), label ?: "", icon)
            } else if (icon != null) {
                Icon(
                    modifier = Modifier.requiredSize(24.dp),
                    imageVector = icon,
                    contentDescription = null,
                )
            } else if (label != null) {
                Text(
                    text = label,
                    fontSize = 16.sp,
                    maxLines = 1,
                    softWrap = false,
                )
            }
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(text = displayName)
            Text(text = data.type.toString())
        }
    }
}

