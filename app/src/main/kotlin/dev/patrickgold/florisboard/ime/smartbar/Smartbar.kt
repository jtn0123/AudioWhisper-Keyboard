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

package dev.patrickgold.florisboard.ime.smartbar

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.updateTransition
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.absoluteOffset
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.UnfoldLess
import androidx.compose.material.icons.filled.UnfoldMore
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.isUnspecified
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.dp
import com.jtn0123.audiowhisper.dictation.Dictation
import com.jtn0123.audiowhisper.reader.ReadAloud
import com.jtn0123.audiowhisper.rust.SmartbarRules
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.nlp.NlpInlineAutofill
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionButton
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionsRow
import dev.patrickgold.florisboard.ime.smartbar.quickaction.ToggleOverflowPanelAction
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.jetpref.datastore.model.observeAsState
import kotlin.math.roundToInt
import kotlinx.coroutines.launch
import org.florisboard.lib.compose.horizontalTween
import org.florisboard.lib.compose.verticalTween
import org.florisboard.lib.snygg.ui.SnyggBox
import org.florisboard.lib.snygg.ui.SnyggColumn
import org.florisboard.lib.snygg.ui.SnyggIcon
import org.florisboard.lib.snygg.ui.SnyggIconButton
import org.florisboard.lib.snygg.ui.SnyggRow
import org.florisboard.lib.snygg.ui.rememberSnyggThemeQuery

const val AnimationDuration = 200

val VerticalEnterTransition = EnterTransition.verticalTween(AnimationDuration)
val VerticalExitTransition = ExitTransition.verticalTween(AnimationDuration)

private val HorizontalEnterTransition = EnterTransition.horizontalTween(AnimationDuration)
private val HorizontalExitTransition = ExitTransition.horizontalTween(AnimationDuration)

private val NoEnterTransition = EnterTransition.horizontalTween(0)
private val NoExitTransition = ExitTransition.horizontalTween(0)

private val AnimationTween = tween<Float>(AnimationDuration)
private val NoAnimationTween = tween<Float>(0)

@Composable
fun Smartbar() {
    val prefs by FlorisPreferenceStore
    val smartbarEnabled by prefs.smartbar.enabled.observeAsState()
    val extendedActionsPlacement by prefs.smartbar.extendedActionsPlacement.observeAsState()

    AnimatedVisibility(
        visible = smartbarEnabled,
        enter = VerticalEnterTransition,
        exit = VerticalExitTransition,
    ) {
        when (extendedActionsPlacement) {
            ExtendedActionsPlacement.ABOVE_CANDIDATES -> {
                SnyggColumn(elementName = FlorisImeUi.Smartbar.elementName) {
                    SmartbarSecondaryRow()
                    SmartbarMainRow()
                }
            }

            ExtendedActionsPlacement.BELOW_CANDIDATES -> {
                SnyggColumn(elementName = FlorisImeUi.Smartbar.elementName) {
                    SmartbarMainRow()
                    SmartbarSecondaryRow()
                }
            }

            ExtendedActionsPlacement.OVERLAY_APP_UI -> {
                SnyggBox(elementName = FlorisImeUi.Smartbar.elementName,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(FlorisImeSizing.smartbarHeight),
                    allowClip = false,
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(FlorisImeSizing.smartbarHeight * 2)
                            .absoluteOffset(y = -FlorisImeSizing.smartbarHeight),
                        contentAlignment = Alignment.BottomStart,
                    ) {
                        SmartbarSecondaryRow()
                    }
                    SmartbarMainRow()
                }
            }
        }
    }
}

@Composable
private fun SmartbarMainRow(modifier: Modifier = Modifier) {
    val prefs by FlorisPreferenceStore
    val context = LocalContext.current
    val keyboardManager by context.keyboardManager()
    val nlpManager by context.nlpManager()
    val scope = rememberCoroutineScope()

    val inlineSuggestions by NlpInlineAutofill.suggestions.collectAsState()
    // AudioWhisper: the words being dictated show where the suggestions go (see CandidatesRow),
    // ahead of autofill chips and the expanded actions.
    val dictationPreview by Dictation.preview.collectAsState()
    val shouldShowInlineSuggestionsUi = inlineSuggestions.isNotEmpty() && dictationPreview.isEmpty()
    // AudioWhisper: while words show, the row is Gboard's, mirrored for a left hand: the mic, the
    // words and the arrow. Rust decides, and sizes the arrow's slot, as for its own renderer.
    val config = smartbarConfig(dictationPreview.isNotEmpty(), inlineSuggestions.isNotEmpty())
    val roomy = SmartbarRules.roomy(config)
    val toggleSlot = SmartbarRules.toggleWidth(config)

    val smartbarLayout by prefs.smartbar.layout.observeAsState()
    val flipToggles by prefs.smartbar.flipToggles.observeAsState()
    val sharedActionsExpanded by prefs.smartbar.sharedActionsExpanded.observeAsState()
    val extendedActionsExpanded by prefs.smartbar.extendedActionsExpanded.observeAsState()

    val shouldAnimate by prefs.smartbar.sharedActionsExpandWithAnimation.observeAsState()

    @Composable
    fun SharedActionsToggle() {
        SnyggIconButton(
            elementName = FlorisImeUi.SmartbarSharedActionsToggle.elementName,
            onClick = {
                if (/* was */ sharedActionsExpanded) {
                    keyboardManager.activeState.isActionsOverflowVisible = false
                }
                scope.launch {
                    prefs.smartbar.sharedActionsExpanded.set(!sharedActionsExpanded)
                }
            },
            modifier = Modifier.sizeIn(maxHeight = FlorisImeSizing.smartbarHeight).aspectRatio(1f)
        ) {
            val transition = updateTransition(sharedActionsExpanded, label = "sharedActionsExpandedToggleBtn")
            val rotation by transition.animateFloat(
                transitionSpec = {
                    if (shouldAnimate) AnimationTween else NoAnimationTween
                },
                label = "rotation",
            ) {
                if (it) 180f else 0f
            }
            val arrowIcon = if (flipToggles) {
                Icons.AutoMirrored.Default.KeyboardArrowLeft
            } else {
                Icons.AutoMirrored.Default.KeyboardArrowRight
            }
            val incognitoIcon = ImageVector.vectorResource(id = R.drawable.ic_incognito)
            val incognitoDisplayMode = prefs.keyboard.incognitoDisplayMode.observeAsState()
            val isIncognitoMode = keyboardManager.activeState.isIncognitoMode
            val icon = if (isIncognitoMode) {
                when (incognitoDisplayMode.value) {
                    IncognitoDisplayMode.REPLACE_SHARED_ACTIONS_TOGGLE -> incognitoIcon
                    IncognitoDisplayMode.DISPLAY_BEHIND_KEYBOARD -> arrowIcon
                }
            } else {
                arrowIcon
            }
            SnyggIcon(
                modifier = Modifier.rotate(if (incognitoDisplayMode.value == IncognitoDisplayMode.DISPLAY_BEHIND_KEYBOARD) rotation else 0f),
                imageVector = icon,
            )
        }
    }

    @Composable
    fun RowScope.CenterContent() {
        val expanded = sharedActionsExpanded && smartbarLayout == SmartbarLayout.SUGGESTIONS_ACTIONS_SHARED &&
            dictationPreview.isEmpty()
        Box(
            modifier = Modifier
                .weight(1f)
                .height(FlorisImeSizing.smartbarHeight),
        ) {
            val enterTransition = if (shouldAnimate) HorizontalEnterTransition else NoEnterTransition
            val exitTransition = if (shouldAnimate) HorizontalExitTransition else NoExitTransition
            androidx.compose.animation.AnimatedVisibility(
                visible = !expanded,
                enter = enterTransition,
                exit = exitTransition,
            ) {
                if (shouldShowInlineSuggestionsUi) {
                    InlineSuggestionsUi(inlineSuggestions)
                } else {
                    CandidatesRow()
                }
            }
            androidx.compose.animation.AnimatedVisibility(
                visible = expanded,
                enter = enterTransition,
                exit = exitTransition,
            ) {
                QuickActionsRow(
                    FlorisImeUi.SmartbarSharedActionsRow.elementName,
                    modifier = modifier
                        .fillMaxWidth()
                        .height(FlorisImeSizing.smartbarHeight),
                )
            }
        }
    }

    @Composable
    fun ExtendedActionsToggle() {
        SnyggIconButton(
            elementName = FlorisImeUi.SmartbarExtendedActionsToggle.elementName,
            onClick = {
                if (/* was */ extendedActionsExpanded) {
                    keyboardManager.activeState.isActionsOverflowVisible = false
                }
                scope.launch {
                    prefs.smartbar.extendedActionsExpanded.set(!extendedActionsExpanded)
                }
            },
            modifier = Modifier.sizeIn(maxHeight = FlorisImeSizing.smartbarHeight).aspectRatio(1f)
        ) {
            val transition = updateTransition(extendedActionsExpanded, label = "smartbarSecondaryRowToggleBtn")
            val alpha by transition.animateFloat(label = "alpha") { if (it) 1f else 0f }
            val rotation by transition.animateFloat(label = "rotation") { if (it) 180f else 0f }
            // Expanded icon
            SnyggIcon(
                elementName = FlorisImeUi.SmartbarExtendedActionsToggle.elementName,
                modifier = Modifier
                    .alpha(alpha)
                    .rotate(rotation),
                imageVector = Icons.Default.UnfoldLess,
            )
            // Not expanded icon
            SnyggIcon(
                elementName = FlorisImeUi.SmartbarExtendedActionsToggle.elementName,
                modifier = Modifier
                    .alpha(1f - alpha)
                    .rotate(rotation - 180f),
                imageVector = Icons.Default.UnfoldMore,
            )
        }
    }

    /** [gboard]: while words show the mic has no padding, and no slot at all when none is set. */
    @Composable
    fun StickyAction(gboard: Boolean = false) {
        val actionArrangement by prefs.smartbar.actionArrangement.observeAsState()
        val evaluator by keyboardManager.activeSmartbarEvaluator.collectAsState()

        val action = when {
            actionArrangement.stickyAction != null -> {
                actionArrangement.stickyAction
            }

            smartbarLayout == SmartbarLayout.SUGGESTIONS_ACTIONS_SHARED && sharedActionsExpanded -> {
                ToggleOverflowPanelAction
            }

            else -> null
        }

        if (action != null) {
            QuickActionButton(
                modifier = if (gboard) Modifier else Modifier.padding(horizontal = 4.dp),
                action = action,
                evaluator = evaluator,
            )
        } else if (!gboard) {
            Spacer(
                modifier = Modifier
                    .padding(horizontal = 4.dp)
                    .aspectRatio(1f),
            )
        }
    }

    SideEffect {
        if (!shouldAnimate) {
            scope.launch {
                prefs.smartbar.sharedActionsExpandWithAnimation.set(true)
            }
        }
    }

    SnyggRow(
        modifier = modifier
            .fillMaxWidth()
            .height(FlorisImeSizing.smartbarHeight),
    ) {
        when (smartbarLayout) {
            SmartbarLayout.SUGGESTIONS_ONLY -> {
                if (shouldShowInlineSuggestionsUi) {
                    InlineSuggestionsUi(inlineSuggestions)
                } else {
                    CandidatesRow()
                }
            }

            SmartbarLayout.ACTIONS_ONLY -> {
                if (shouldShowInlineSuggestionsUi) {
                    InlineSuggestionsUi(inlineSuggestions)
                } else {
                    QuickActionsRow(FlorisImeUi.SmartbarSharedActionsRow.elementName)
                }
            }

            SmartbarLayout.SUGGESTIONS_ACTIONS_SHARED -> when {
                roomy && flipToggles -> {
                    StickyAction(gboard = true)
                    CenterContent()
                    ToggleSlot(toggleSlot) { SharedActionsToggle() }
                }
                roomy -> {
                    ToggleSlot(toggleSlot) { SharedActionsToggle() }
                    CenterContent()
                    StickyAction(gboard = true)
                }
                !flipToggles -> {
                    SharedActionsToggle()
                    CenterContent()
                    PinnedReadAloudAction()
                    StickyAction()
                }
                else -> {
                    StickyAction()
                    PinnedReadAloudAction()
                    CenterContent()
                    SharedActionsToggle()
                }
            }

            SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED -> when {
                roomy && flipToggles -> {
                    StickyAction(gboard = true)
                    CenterContent()
                    ToggleSlot(toggleSlot) { ExtendedActionsToggle() }
                }
                roomy -> {
                    ToggleSlot(toggleSlot) { ExtendedActionsToggle() }
                    CenterContent()
                    StickyAction(gboard = true)
                }
                !flipToggles -> {
                    ExtendedActionsToggle()
                    CenterContent()
                    PinnedReadAloudAction()
                    StickyAction()
                }
                else -> {
                    StickyAction()
                    PinnedReadAloudAction()
                    CenterContent()
                    ExtendedActionsToggle()
                }
            }
        }
    }
}

/**
 * AudioWhisper: the arrow's slot while words show, [width] pixels from Rust
 * (`smartbar::Config::toggle_width`). The toggle keeps its square button, centred; only its
 * transparent margins overhang the slot, as the Rust renderer draws it.
 */
@Composable
private fun ToggleSlot(width: Float, toggle: @Composable () -> Unit) {
    Box(
        modifier = Modifier
            .height(FlorisImeSizing.smartbarHeight)
            .layout { measurable, constraints ->
                val slot = width.roundToInt()
                val button = measurable.measure(constraints.copy(minWidth = 0, maxWidth = Constraints.Infinity))
                layout(slot, button.height) { button.place((slot - button.width) / 2, 0) }
            },
        contentAlignment = Alignment.Center,
    ) {
        toggle()
    }
}

private val PinnedReadAloud = QuickAction.InsertKey(TextKeyData.READ_ALOUD)

/**
 * The speaker beside the mic, which reads the screen aloud: a tap from the top of the screen, a hold
 * from the top of the page. It can be turned off in the Smartbar settings.
 */
@Composable
private fun PinnedReadAloudAction() {
    val prefs by FlorisPreferenceStore
    val shown by prefs.smartbar.showReadAloud.observeAsState()
    if (!shown) return
    val keyboardManager by LocalContext.current.keyboardManager()
    val evaluator by keyboardManager.activeSmartbarEvaluator.collectAsState()
    QuickActionButton(
        modifier = Modifier.padding(horizontal = 4.dp),
        action = PinnedReadAloud,
        evaluator = evaluator,
    )
}

@Composable
private fun SmartbarSecondaryRow(modifier: Modifier = Modifier) {
    val prefs by FlorisPreferenceStore
    val smartbarLayout by prefs.smartbar.layout.observeAsState()
    val secondaryRowStyle = rememberSnyggThemeQuery(FlorisImeUi.SmartbarExtendedActionsRow.elementName)
    val windowStyle = rememberSnyggThemeQuery(FlorisImeUi.Window.elementName)
    val extendedActionsExpanded by prefs.smartbar.extendedActionsExpanded.observeAsState()
    val extendedActionsPlacement by prefs.smartbar.extendedActionsPlacement.observeAsState()
    val background = secondaryRowStyle.background().let { color ->
        if (extendedActionsPlacement == ExtendedActionsPlacement.OVERLAY_APP_UI) {
            if (color.isUnspecified || color.alpha == 0f) {
                windowStyle.background(default = Color.Black)
            } else {
                color
            }
        } else {
            color
        }
    }

    AnimatedVisibility(
        visible = smartbarLayout == SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED && extendedActionsExpanded,
        enter = VerticalEnterTransition,
        exit = VerticalExitTransition,
    ) {
        QuickActionsRow(
            FlorisImeUi.SmartbarExtendedActionsRow.elementName,
            modifier = modifier
                .fillMaxWidth()
                .height(FlorisImeSizing.smartbarHeight)
                .background(background),
        )
    }
}

/**
 * The packed facts Rust places the Smartbar from (see `smartbar::Config`), as Compose reads them. The
 * Rust renderer reads the same facts without Compose (`smartbarFacts`), so both ask Rust the same question.
 */
@Composable
private fun smartbarConfig(preview: Boolean, inline: Boolean): FloatArray = with(LocalDensity.current) {
    val prefs by FlorisPreferenceStore
    val nlp by LocalContext.current.nlpManager()
    val dictation by Dictation.status.collectAsState()
    val reading by ReadAloud.status.collectAsState()
    val words by nlp.activeCandidatesWordsFlow.collectAsState()
    val enabled by prefs.smartbar.enabled.observeAsState()
    val layout by prefs.smartbar.layout.observeAsState()
    val flip by prefs.smartbar.flipToggles.observeAsState()
    val shared by prefs.smartbar.sharedActionsExpanded.observeAsState()
    val extended by prefs.smartbar.extendedActionsExpanded.observeAsState()
    val placement by prefs.smartbar.extendedActionsPlacement.observeAsState()
    val speaker by prefs.smartbar.showReadAloud.observeAsState()
    val arrangement by prefs.smartbar.actionArrangement.observeAsState()
    val display by prefs.suggestion.displayMode.observeAsState()
    fun bit(value: Boolean) = if (value) 1f else 0f
    floatArrayOf(bit(enabled), layout.ordinal.toFloat(), bit(flip), bit(shared), bit(extended),
        placement.ordinal.toFloat(), bit(speaker), bit(arrangement.stickyAction != null),
        arrangement.dynamicActions.size.toFloat(), display.ordinal.toFloat(), density,
        FlorisImeSizing.smartbarHeight.toPx(), bit(inline), bit(preview), bit(words),
        bit(dictation != Dictation.Status.IDLE), bit(reading == ReadAloud.Status.READING))
}
