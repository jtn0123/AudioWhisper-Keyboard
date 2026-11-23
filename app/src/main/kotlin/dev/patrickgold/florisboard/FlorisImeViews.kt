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


import android.content.Context
import android.inputmethodservice.ExtractEditText
import android.util.TypedValue
import android.view.Gravity
import android.widget.FrameLayout
import android.widget.LinearLayout
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.AbstractComposeView
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import dev.patrickgold.florisboard.ime.core.SelectSubtypePanel
import dev.patrickgold.florisboard.ime.core.isSubtypeSelectionShowing
import dev.patrickgold.florisboard.ime.sheet.BottomSheetHostUi
import dev.patrickgold.florisboard.ime.sheet.isBottomSheetShowing
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionsEditorPanel
import dev.patrickgold.florisboard.ime.theme.FlorisImeTheme
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import dev.patrickgold.florisboard.lib.devtools.flogError
import kotlinx.coroutines.flow.update
import org.florisboard.lib.compose.ProvideLocalizedResources
import org.florisboard.lib.snygg.ui.SnyggBox
import org.florisboard.lib.snygg.ui.SnyggButton
import org.florisboard.lib.snygg.ui.SnyggRow
import org.florisboard.lib.snygg.ui.SnyggText
import org.florisboard.lib.snygg.ui.rememberSnyggThemeQuery

private const val INPUT_VIEW_TRACE_TAG = "aw-profile"

internal class ComposeInputView(context: Context) : AbstractComposeView(context) {
    private val service = context as? FlorisImeService
    init {
        isHapticFeedbackEnabled = true
        layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
    }

    @Composable
    override fun Content() {
        val service = service ?: return
        service.ImeUiWrapper()
    }

    override fun getAccessibilityClassName(): CharSequence {
        return javaClass.name
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        service?.updateSoftInputWindowLayoutParameters()
    }

    // AudioWhisper: a quick hide and show must not leave the keyboard's surfaces believing the
    // window is gone (InputViewVisibility).
    private val windowVisibility = com.jtn0123.audiowhisper.render.InputViewVisibility(::dispatchWindowVisibilityChanged)

    override fun onWindowVisibilityChanged(visibility: Int) {
        super.onWindowVisibilityChanged(visibility)
        windowVisibility.told(visibility)
    }

    /** The input method window is being shown. */
    fun windowShown() {
        val retold = windowVisibility.shown(getWindowVisibility())
        if (android.util.Log.isLoggable(INPUT_VIEW_TRACE_TAG, android.util.Log.DEBUG)) {
            android.util.Log.d(INPUT_VIEW_TRACE_TAG, "input view shown retold=${if (retold) 1 else 0}")
        }
    }
}

internal class FlorisBottomSheetHostUiView(context: Context) : AbstractComposeView(context) {
    private val service = context as? FlorisImeService
    init {
        isHapticFeedbackEnabled = true
        layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT)
    }

    @Composable
    override fun Content() {
        val service = service ?: return
        val context = LocalContext.current
        val keyboardManager by context.keyboardManager()
        val state by keyboardManager.activeState.collectAsState()

        ProvideLocalizedResources(
            service.localizedContext.collectAsState().value,
            appName = R.string.floris_app_name,
            forceLayoutDirection = LayoutDirection.Ltr,
        ) {
            FlorisImeTheme {
                BottomSheetHostUi(
                    isShowing = state.isBottomSheetShowing() || state.isSubtypeSelectionShowing(),
                    onHide = {
                        if (state.isBottomSheetShowing()) {
                            keyboardManager.activeState.isActionsEditorVisible = false
                        }
                        if (state.isSubtypeSelectionShowing()) {
                            keyboardManager.activeState.isSubtypeSelectionVisible = false
                        }
                    },
                ) {
                    if (state.isBottomSheetShowing()) {
                        QuickActionsEditorPanel()
                    }
                    if (state.isSubtypeSelectionShowing()) {
                        SelectSubtypePanel()
                    }
                }
            }
        }
    }

    override fun getAccessibilityClassName(): CharSequence {
        return javaClass.name
    }
}

internal class ComposeExtractedLandscapeInputView @JvmOverloads constructor(
    context: Context,
    eet: ExtractEditText? = null,
) : FrameLayout(context) {
    private val service = context as? FlorisImeService
    val composeView: ComposeView
    val extractEditText: ExtractEditText

    init {
        isHapticFeedbackEnabled = true
        layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT)

        extractEditText = (eet ?: ExtractEditText(context)).also {
            it.id = android.R.id.inputExtractEditText
            it.layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT)
            it.background = null
            it.gravity = Gravity.TOP
            it.isVerticalScrollBarEnabled = true
        }
        addView(extractEditText)

        composeView = ComposeView(context).also { it.setContent { Content() } }
        addView(composeView)
    }

    @Composable
    fun Content() {
        val service = service ?: return
        ProvideLocalizedResources(
            service.localizedContext.collectAsState().value,
            appName = R.string.floris_app_name,
            forceLayoutDirection = LayoutDirection.Ltr,
        ) {
            FlorisImeTheme {
                val activeEditorInfo by service.editorInstance.activeInfoFlow.collectAsState()
                SnyggBox(elementName = FlorisImeUi.ExtractedLandscapeInputLayout.elementName) {
                    SnyggRow(
                        modifier = Modifier.fillMaxSize(),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        SnyggBox(
                            elementName = FlorisImeUi.ExtractedLandscapeInputLayout.elementName,
                            modifier = Modifier
                                .fillMaxHeight()
                                .weight(1f),
                        ) {
                            val fieldStyle = rememberSnyggThemeQuery(FlorisImeUi.ExtractedLandscapeInputField.elementName)
                            val foreground = fieldStyle.foreground()
                            AndroidView(
                                factory = { extractEditText },
                                update = { view ->
                                    view.background = null
                                    view.backgroundTintList = null
                                    view.foregroundTintList = null
                                    view.setTextColor(foreground.toArgb())
                                    view.setHintTextColor(foreground.copy(foreground.alpha * 0.6f).toArgb())
                                    view.setTextSize(
                                        TypedValue.COMPLEX_UNIT_SP,
                                        fieldStyle.fontSize(default = 16.sp).value,
                                    )
                                },
                            )
                        }
                        SnyggButton(
                            elementName = FlorisImeUi.ExtractedLandscapeInputAction.elementName,
                            onClick = {
                                if (activeEditorInfo.extractedActionId != 0) {
                                    service.currentInputConnection?.performEditorAction(activeEditorInfo.extractedActionId)
                                } else {
                                    service.editorInstance.performEnterAction(activeEditorInfo.imeOptions.action)
                                }
                            },
                            modifier = Modifier.padding(horizontal = 8.dp),
                        ) {
                            SnyggText(
                                text = activeEditorInfo.extractedActionLabel
                                    ?: service.getTextForImeAction(activeEditorInfo.imeOptions.action.toInt())
                                    ?: "ACTION",
                            )
                        }
                    }
                }
            }
        }
    }

    override fun getAccessibilityClassName(): CharSequence {
        return javaClass.name
    }

    override fun onAttachedToWindow() {
        removeView(extractEditText)
        super.onAttachedToWindow()
        try {
            (parent as LinearLayout).let { extractEditLayout ->
                extractEditLayout.layoutParams = LayoutParams(
                    LinearLayout.LayoutParams.MATCH_PARENT,
                    LinearLayout.LayoutParams.MATCH_PARENT,
                ).also { it.setMargins(0, 0, 0, 0) }
                extractEditLayout.setPadding(0, 0, 0, 0)
            }
        } catch (e: Throwable) {
            flogError { e.message.toString() }
        }
    }
}
