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

import android.util.Size
import android.view.inputmethod.InlineSuggestionsRequest
import android.view.inputmethod.InlineSuggestionsResponse
import android.widget.inline.InlinePresentationSpec
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import dev.patrickgold.florisboard.ime.nlp.NlpInlineAutofill
import dev.patrickgold.florisboard.lib.devtools.LogTopic
import dev.patrickgold.florisboard.lib.devtools.flogInfo
import dev.patrickgold.florisboard.lib.devtools.flogWarning

internal fun FlorisImeService.createInlineSuggestionsRequest(): InlineSuggestionsRequest? {
    if (!prefs.smartbar.enabled.get() || !prefs.suggestion.api30InlineSuggestionsEnabled.get()) {
        flogInfo(LogTopic.IMS_EVENTS) {
            "Ignoring inline suggestions request because Smartbar and/or inline suggestions are disabled."
        }
        return null
    }

    flogInfo(LogTopic.IMS_EVENTS) { "Creating inline suggestions request" }
    val stylesBundle = themeManager.createInlineSuggestionUiStyleBundle(this)
    if (stylesBundle == null) {
        flogWarning(LogTopic.IMS_EVENTS) { "Failed to retrieve inline suggestions style bundle" }
        return null
    }
    val spec = InlinePresentationSpec.Builder(
        Size(0, 0),
        Size(Int.MAX_VALUE, Int.MAX_VALUE),
    ).run {
        setStyle(stylesBundle)
        build()
    }

    return InlineSuggestionsRequest.Builder(listOf(spec)).run {
        setMaxSuggestionCount(InlineSuggestionsRequest.SUGGESTION_COUNT_UNLIMITED)
        build()
    }
}

internal fun FlorisImeService.receiveInlineSuggestions(response: InlineSuggestionsResponse): Boolean {
    val inlineSuggestions = response.inlineSuggestions
    flogInfo(LogTopic.IMS_EVENTS) {
        "Received inline suggestions response with ${inlineSuggestions.size} suggestion(s) provided."
    }
    return NlpInlineAutofill.showInlineSuggestions(this, inlineSuggestions)
}
