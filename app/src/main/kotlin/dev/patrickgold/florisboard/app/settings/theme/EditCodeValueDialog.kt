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
import androidx.compose.runtime.setValue

import android.widget.Toast
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColor
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.selection.LocalTextSelectionColors
import androidx.compose.foundation.text.selection.TextSelectionColors
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.HelpOutline
import androidx.compose.material.icons.filled.Pageview
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.input.InputKeyEventReceiver
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.NATIVE_NULLPTR
import dev.patrickgold.florisboard.lib.compose.FlorisHyperlinkText
import dev.patrickgold.florisboard.lib.util.InputMethodUtils
import dev.patrickgold.jetpref.material.ui.JetPrefAlertDialog
import dev.patrickgold.jetpref.material.ui.JetPrefTextField
import dev.patrickgold.jetpref.material.ui.JetPrefTextFieldDefaults
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.launch
import org.florisboard.lib.android.showShortToast
import org.florisboard.lib.android.stringRes
import org.florisboard.lib.compose.FlorisIconButton
import org.florisboard.lib.compose.stringRes
import org.florisboard.lib.kotlin.curlyFormat

private val TransparentTextSelectionColors = TextSelectionColors(
    handleColor = Color.Transparent,
    backgroundColor = Color.Transparent,
)
@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun EditCodeValueDialog(
    codeValue: String,
    checkExisting: (String) -> Boolean,
    onAdd: (String) -> Unit,
    onDelete: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    val context = LocalContext.current
    val keyboardManager by context.keyboardManager()
    val scope = rememberCoroutineScope()

    var inputCodeString by rememberSaveable(codeValue) {
        val str = if (codeValue == KeyCode.UNSPECIFIED.toString()) "" else codeValue
        mutableStateOf(str)
    }
    val textKeyData = remember(inputCodeString) {
        inputCodeString.toIntOrNull()?.let { code ->
            TextKeyData.getCodeInfoAsTextKeyData(code)
        }
    }
    var showKeyCodesHelp by rememberSaveable(codeValue) { mutableStateOf(false) }
    var showError by rememberSaveable(codeValue) { mutableStateOf(false) }
    var errorId by rememberSaveable(codeValue) { mutableIntStateOf(NATIVE_NULLPTR.toInt()) }

    val focusRequester = remember { FocusRequester() }
    val isFlorisBoardEnabled by InputMethodUtils.observeIsFlorisboardEnabled()
    val isFlorisBoardSelected by InputMethodUtils.observeIsFlorisboardSelected()

    var isRecordingKey by remember { mutableStateOf(false) }
    var lastRecordingToast by remember { mutableStateOf<Toast?>(null) }
    val recordingKeyColor = if (isRecordingKey) {
        rememberInfiniteTransition().animateColor(
            initialValue = LocalContentColor.current,
            targetValue = MaterialTheme.colorScheme.error,
            animationSpec = infiniteRepeatable(
                tween(750),
                repeatMode = RepeatMode.Reverse,
            ),
        ).value
    } else {
        LocalContentColor.current
    }

    fun requestStartRecording() {
        if (isRecordingKey) {
            isRecordingKey = false
            return
        }
        if (!isFlorisBoardEnabled || !isFlorisBoardSelected) {
            scope.launch {
                lastRecordingToast?.cancel()
                lastRecordingToast = context.showShortToast(
                    R.string.settings__theme_editor__code_recording_requires_default_ime_floris,
                    "app_name" to context.stringRes(R.string.floris_app_name),
                )
                InputMethodUtils.showImePicker(context)
            }
            return
        }
        showError = false
        isRecordingKey = true
    }

    if (isRecordingKey) {
        DisposableEffect(Unit) {
            val receiver = object : InputKeyEventReceiver {
                override fun onInputKeyDown(data: KeyData) = Unit
                override fun onInputKeyUp(data: KeyData) {
                    inputCodeString = data.code.toString()
                    isRecordingKey = false
                }

                override fun onInputKeyRepeat(data: KeyData) = Unit
                override fun onInputKeyCancel(data: KeyData) = Unit
            }
            val defaultReceiver = keyboardManager.inputEventDispatcher.keyEventReceiver
            keyboardManager.inputEventDispatcher.keyEventReceiver = receiver
            lastRecordingToast?.cancel()
            scope.launch {
                lastRecordingToast = context.showShortToast(R.string.settings__theme_editor__code_recording_started)
            }
            focusRequester.requestFocus()
            onDispose {
                keyboardManager.inputEventDispatcher.keyEventReceiver = defaultReceiver
                lastRecordingToast?.cancel()
                // Undispatched: if the dialog itself is leaving, `scope` is cancelled right after this
                // onDispose, and a dispatched launch would never run.
                scope.launch(start = CoroutineStart.UNDISPATCHED) {
                    lastRecordingToast = context.showShortToast(R.string.settings__theme_editor__code_recording_stopped)
                }
            }
        }
    }

    JetPrefAlertDialog(
        title = stringRes(
            if (codeValue == KeyCode.UNSPECIFIED.toString()) {
                R.string.settings__theme_editor__add_code
            } else {
                R.string.settings__theme_editor__edit_code
            }
        ),
        confirmLabel = stringRes(
            if (codeValue == KeyCode.UNSPECIFIED.toString()) {
                R.string.action__add
            } else {
                R.string.action__apply
            }
        ),
        onConfirm = {
            val code = inputCodeString.trim().toIntOrNull(radix = 10)
            when {
                code == null || (code !in KeyCode.Spec.CHARACTERS && code !in KeyCode.Spec.INTERNAL) -> {
                    errorId = R.string.settings__theme_editor__code_invalid
                    showError = true
                }

                code.toString() == codeValue -> {
                    onDismiss()
                }

                checkExisting(code.toString()) -> {
                    errorId = R.string.settings__theme_editor__code_already_exists
                    showError = true
                }

                else -> {
                    if (codeValue != KeyCode.UNSPECIFIED.toString()) {
                        onDelete(codeValue)
                    }
                    onAdd(code.toString())
                    onDismiss()
                }
            }
        },
        dismissLabel = stringRes(R.string.action__cancel),
        onDismiss = onDismiss,
        neutralLabel = if (codeValue != KeyCode.UNSPECIFIED.toString()) {
            stringRes(R.string.action__delete)
        } else {
            null
        },
        neutralColors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.error),
        onNeutral = {
            onDelete(codeValue)
            onDismiss()
        },
        trailingIconTitle = {
            FlorisIconButton(
                onClick = { showKeyCodesHelp = !showKeyCodesHelp },
                modifier = Modifier.offset(x = 12.dp),
                icon = Icons.AutoMirrored.Filled.HelpOutline,
            )
        },
    ) {
        Column {
            AnimatedVisibility(visible = showKeyCodesHelp) {
                Column(modifier = Modifier.padding(bottom = 16.dp)) {
                    Text(text = stringRes(R.string.settings__theme_editor__code_recording_help_text))
                    Text(text = stringRes(R.string.settings__theme_editor__code_help_text))
                    FlorisHyperlinkText(
                        text = "Characters (unicode-table.com)",
                        url = stringRes(R.string.florisboard__character_key_codes_url),
                    )
                    FlorisHyperlinkText(
                        text = "Internal (github.com)",
                        url = stringRes(R.string.florisboard__internal_key_codes_url),
                    )
                }
            }
            TextKeyDataPreviewBox(
                modifier = Modifier.padding(bottom = 8.dp),
                textKeyData = textKeyData,
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                val textSelectionColors = if (isRecordingKey) {
                    TransparentTextSelectionColors
                } else {
                    LocalTextSelectionColors.current
                }
                CompositionLocalProvider(LocalTextSelectionColors provides textSelectionColors) {
                    JetPrefTextField(
                        modifier = Modifier
                            .focusRequester(focusRequester)
                            .weight(1f),
                        value = inputCodeString,
                        onValueChange = { v ->
                            inputCodeString = v
                            showError = false
                        },
                        placeholderText = when {
                            isRecordingKey -> {
                                stringRes(R.string.settings__theme_editor__code_recording_placeholder)
                            }
                            inputCodeString.isEmpty() -> {
                                stringRes(R.string.settings__theme_editor__code_placeholder)
                            }
                            else -> {
                                null
                            }
                        },
                        isError = showError,
                        singleLine = true,
                        appearance = JetPrefTextFieldDefaults.filled(
                            colors = if (isRecordingKey) {
                                TextFieldDefaults.colors(
                                    focusedTextColor = Color.Transparent,
                                    cursorColor = Color.Transparent,
                                )
                            } else {
                                TextFieldDefaults.colors()
                            }
                        ),
                        trailingIcon = {
                            FlorisIconButton(
                                onClick = { requestStartRecording() },
                                icon = Icons.Default.Pageview,
                                iconColor = recordingKeyColor,
                            )
                        }
                    )
                }
            }
            AnimatedVisibility(visible = showError) {
                Text(
                    modifier = Modifier.padding(top = 4.dp),
                    text = stringRes(errorId).curlyFormat(
                        "c_min" to KeyCode.Spec.CHARACTERS_MIN,
                        "c_max" to KeyCode.Spec.CHARACTERS_MAX,
                        "i_min" to KeyCode.Spec.INTERNAL_MIN,
                        "i_max" to KeyCode.Spec.INTERNAL_MAX,
                    ),
                    color = MaterialTheme.colorScheme.error,
                )
            }
        }
    }
}

