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

package dev.patrickgold.florisboard.ime.smartbar.quickaction

import androidx.compose.foundation.LocalIndication
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.indication
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.PressInteraction
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.disabled
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import com.jtn0123.audiowhisper.dictation.Dictation
import com.jtn0123.audiowhisper.dictation.LitMic
import com.jtn0123.audiowhisper.reader.LitSpeaker
import com.jtn0123.audiowhisper.reader.ReadAloud
import dev.patrickgold.compose.tooltip.PlainTooltip
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.computeImageVector
import dev.patrickgold.florisboard.ime.keyboard.computeLabel
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import org.florisboard.lib.snygg.SnyggSelector
import org.florisboard.lib.snygg.ui.SnyggBox
import org.florisboard.lib.snygg.ui.SnyggIcon
import org.florisboard.lib.snygg.ui.SnyggText

enum class QuickActionBarType {
    INTERACTIVE_BUTTON,
    INTERACTIVE_TILE,
    EDITOR_TILE;
}

@Composable
fun QuickActionButton(
    action: QuickAction,
    evaluator: ComputingEvaluator,
    modifier: Modifier = Modifier,
    type: QuickActionBarType = QuickActionBarType.INTERACTIVE_BUTTON,
) {
    val context = LocalContext.current
    // Get the inputFeedbackController through the FlorisImeService companion-object.
    val inputFeedbackController = FlorisImeService.inputFeedbackController()
    val interactionSource = remember { MutableInteractionSource() }
    val isPressed by interactionSource.collectIsPressedAsState()
    val isEnabled = type == QuickActionBarType.EDITOR_TILE || evaluator.evaluateEnabled(action.keyData())
    val elementName = when (type) {
        QuickActionBarType.INTERACTIVE_BUTTON -> FlorisImeUi.SmartbarActionKey
        QuickActionBarType.INTERACTIVE_TILE -> FlorisImeUi.SmartbarActionTile
        QuickActionBarType.EDITOR_TILE -> FlorisImeUi.SmartbarActionsEditorTile
    }.elementName
    val attributes = mapOf(FlorisImeUi.Attr.Code to action.keyData().code)
    val dictation by Dictation.status.collectAsState()
    val litMic = type == QuickActionBarType.INTERACTIVE_BUTTON &&
        action.keyData().code == KeyCode.VOICE_INPUT &&
        dictation != Dictation.Status.IDLE
    val reading by ReadAloud.status.collectAsState()
    val isSpeaker = action.keyData().code == KeyCode.READ_ALOUD
    val litSpeaker = type == QuickActionBarType.INTERACTIVE_BUTTON && isSpeaker && reading == ReadAloud.Status.READING
    val selector = when {
        isPressed -> SnyggSelector.PRESSED
        !isEnabled -> SnyggSelector.DISABLED
        else -> null
    }

    // AudioWhisper: the press handling below sees raw touches only, so TalkBack is told what the
    // button is and given a click (and, for the speaker, the hold's "from the top") to perform.
    val spokenName = when {
        litMic -> stringResource(R.string.dictation__stop)
        litSpeaker -> stringResource(R.string.read_aloud__stop)
        else -> action.computeDisplayName()
    }
    val readFromTop = stringResource(R.string.quick_action__read_aloud__from_top)
    val pressable = isEnabled && type != QuickActionBarType.EDITOR_TILE

    // Need to manually cancel an action if this composable suddenly leaves the composition to prevent the key from
    // being stuck in the pressed state
    DisposableEffect(action, isEnabled) {
        onDispose {
            if (action is QuickAction.InsertKey) {
                action.onPointerCancel(context)
            }
        }
    }

    // Holding the speaker reads from the top of the page; a tooltip would take the hold and cancel the press.
    PlainTooltip(action.computeTooltip(evaluator), enabled = type == QuickActionBarType.INTERACTIVE_BUTTON && !isSpeaker) {
        SnyggBox(
            elementName = elementName,
            attributes = attributes,
            selector = selector,
            modifier = modifier,
            clickAndSemanticsModifier = Modifier
                .semantics(mergeDescendants = true) {
                    role = Role.Button
                    // Tiles show their name as text, which TalkBack already reads.
                    if (type == QuickActionBarType.INTERACTIVE_BUTTON) contentDescription = spokenName
                    if (!isEnabled) disabled()
                    if (pressable) {
                        onClick {
                            action.onPointerDown(context)
                            action.onPointerUp(context)
                            true
                        }
                    }
                    if (pressable && isSpeaker && !litSpeaker) {
                        customActions = listOf(
                            CustomAccessibilityAction(readFromTop) {
                                ReadAloud.read(context, ReadAloud.From.Top)
                                true
                            },
                        )
                    }
                }
                .aspectRatio(1f)
                .indication(interactionSource, LocalIndication.current)
                .pointerInput(action, isEnabled) {
                    awaitEachGesture {
                        val down = awaitFirstDown()
                        down.consume()
                        if (pressable) {
                            val press = PressInteraction.Press(down.position)
                            inputFeedbackController?.keyPress(TextKeyData.UNSPECIFIED)
                            interactionSource.tryEmit(press)
                            action.onPointerDown(context)
                            val up = waitForUpOrCancellation()
                            if (up != null) {
                                up.consume()
                                interactionSource.tryEmit(PressInteraction.Release(press))
                                action.onPointerUp(context)
                            } else {
                                interactionSource.tryEmit(PressInteraction.Cancel(press))
                                action.onPointerCancel(context)
                            }
                        }
                    }
                },
            contentAlignment = Alignment.Center,
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                // Render foreground
                when (action) {
                    is QuickAction.InsertKey -> {
                        val (imageVector, label) = remember(action, evaluator) {
                            evaluator.computeImageVector(action.data) to evaluator.computeLabel(action.data)
                        }
                        if (imageVector != null && litMic) {
                            LitMic(imageVector)
                        } else if (litSpeaker) {
                            LitSpeaker()
                        } else if (imageVector != null) {
                            SnyggBox(
                                elementName = "$elementName-icon",
                                attributes = attributes,
                                selector = selector,
                            ) {
                                SnyggIcon(imageVector = imageVector)
                            }
                        } else if (label != null) {
                            SnyggText(
                                elementName = "$elementName-text",
                                attributes = attributes,
                                selector = selector,
                                text = label,
                            )
                        }
                    }

                    is QuickAction.InsertText -> {
                        SnyggText(
                            elementName = "$elementName-text",
                            attributes = attributes,
                            selector = selector,
                            text = action.data.firstOrNull().toString().ifBlank { "?" },
                        )
                    }
                }

                // Render additional info if this is a tile
                if (type != QuickActionBarType.INTERACTIVE_BUTTON) {
                    SnyggText(
                        elementName = "$elementName-text",
                        attributes = attributes,
                        selector = selector,
                        text = action.computeDisplayName(),
                    )
                }
            }
        }
    }
}
