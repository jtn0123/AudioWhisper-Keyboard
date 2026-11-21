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

package dev.patrickgold.florisboard.ime.nlp

import android.content.Context
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.ime.clipboard.provider.ItemType
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.lib.util.NetworkUtils

internal class ClipboardSuggestionProvider(private val context: Context) : SuggestionProvider {
    private val prefs by FlorisPreferenceStore
    private val clipboardManager by context.clipboardManager()
    private var lastClipboardItemId: Long = -1

    override val providerId = "org.florisboard.nlp.providers.clipboard"

    override suspend fun create() {
        // Do nothing
    }

    override suspend fun preload(subtype: Subtype) {
        // Do nothing
    }

    override suspend fun suggest(
        subtype: Subtype,
        content: EditorContent,
        maxCandidateCount: Int,
        allowPossiblyOffensive: Boolean,
        isPrivateSession: Boolean,
    ): List<SuggestionCandidate> {
        val item = clipboardManager.primaryClip ?: return emptyList()
        // Android's regex engine supplies spans; native policy owns eligibility, overlap and expiry.
        val text = item.stringRepresentation()
        val matches = if (item.type == ItemType.TEXT) buildList {
            addAll(NetworkUtils.getEmailAddresses(text))
            addAll(NetworkUtils.getUrls(text))
            addAll(NetworkUtils.getPhoneNumbers(text))
        } else emptyList()
        val plan = with(NlpClipboardPlan) { row(ENABLED to WireFlag.encode(prefs.clipboard.suggestionEnabled.get()), ITEM_ID to "${item.id}",
            LAST_ID to "$lastClipboardItemId", CONTENT_TEXT to content.text, ITEM_TEXT to (item.text ?: ""), SENSITIVE to WireFlag.encode(item.isSensitive),
            AGE to "${System.currentTimeMillis() - item.creationTimestampMs}", TIMEOUT to "${prefs.clipboard.suggestionTimeout.get() * 1000}") }
        val spans = matches.flatMap { with(NlpClipboardSpan) { row(VALUE to it.value, START to "${it.range.first}", END to "${it.range.last}").asList() } }
        return NlpState.call(NlpOp.CLIPBOARD_PLAN, *plan, *spans.toTypedArray()).map { index ->
            val selected = index.toInt()
            val candidate = if (selected < 0) item else item.copy(text = matches[selected].value.removeSurrounding("(", ")"))
            ClipboardSuggestionCandidate(candidate, this, context)
        }
    }

    override suspend fun notifySuggestionAccepted(subtype: Subtype, candidate: SuggestionCandidate) {
        if (candidate is ClipboardSuggestionCandidate) {
            lastClipboardItemId = candidate.clipboardItem.id
        }
    }

    override suspend fun notifySuggestionReverted(subtype: Subtype, candidate: SuggestionCandidate) {
        // Do nothing
    }

    override suspend fun removeSuggestion(subtype: Subtype, candidate: SuggestionCandidate): Boolean {
        if (candidate is ClipboardSuggestionCandidate) {
            lastClipboardItemId = candidate.clipboardItem.id
            return true
        }
        return false
    }

    override suspend fun getListOfWords(subtype: Subtype): List<String> {
        return emptyList()
    }

    override suspend fun getFrequencyForWord(subtype: Subtype, word: String): Double {
        return 0.0
    }

    override suspend fun destroy() {
        // Do nothing
    }

}
