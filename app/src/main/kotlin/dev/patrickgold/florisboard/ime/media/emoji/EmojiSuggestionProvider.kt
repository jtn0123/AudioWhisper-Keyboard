/*
 * Copyright (C) 2024-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.media.emoji

import android.content.Context
import com.jtn0123.audiowhisper.rust.EmojiRules
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.ime.nlp.EmojiSuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.SuggestionProvider
import dev.patrickgold.florisboard.lib.FlorisLocale
import io.github.reactivecircus.cache4k.Cache

/**
 * Provides emoji suggestions within a text input context.
 *
 * This class handles the following tasks:
 * - Initializes and maintains a list of supported emojis.
 * - Generates and returns emoji suggestions based on user input and preferences.
 *
 * @param context The application context.
 */
class EmojiSuggestionProvider(private val context: Context) : SuggestionProvider {
    override val providerId = "org.florisboard.nlp.providers.emoji"

    private val prefs by FlorisPreferenceStore

    private val cachedEmojiData = Cache.Builder<FlorisLocale, EmojiData>().build()

    override suspend fun create() {
    }

    override suspend fun preload(subtype: Subtype) {
        subtype.locales().forEach { locale ->
            cachedEmojiData.get(locale) {
                EmojiData.get(context, locale)
            }
        }
    }

    override suspend fun suggest(
        subtype: Subtype,
        content: EditorContent,
        maxCandidateCount: Int,
        allowPossiblyOffensive: Boolean,
        isPrivateSession: Boolean
    ): List<SuggestionCandidate> {
        val preferredSkinTone = prefs.emoji.preferredSkinTone.get()
        val showName = prefs.emoji.suggestionCandidateShowName.get()
        val prefix = prefs.emoji.suggestionType.get().prefix
        val minLength = prefs.emoji.suggestionQueryMinLength.get()
        val data = cachedEmojiData.get(subtype.primaryLocale)
        // One call into Rust, a fraction of a millisecond, so no longer moved to Dispatchers.Default.
        return suggestEmojis(data, content.composingText, prefix, minLength, preferredSkinTone, maxCandidateCount).map { emoji ->
            EmojiSuggestionCandidate(
                emoji = emoji,
                showName = showName,
                sourceProvider = this@EmojiSuggestionProvider,
            )
        }
    }

    override suspend fun notifySuggestionAccepted(subtype: Subtype, candidate: SuggestionCandidate) {
        val updateHistory = prefs.emoji.suggestionUpdateHistory.get()
        if (!updateHistory || candidate !is EmojiSuggestionCandidate) {
            return
        }
        EmojiHistoryHelper.markEmojiUsed(prefs, candidate.emoji)
    }

    override suspend fun notifySuggestionReverted(subtype: Subtype, candidate: SuggestionCandidate) {
        // No-op
    }

    override suspend fun removeSuggestion(subtype: Subtype, candidate: SuggestionCandidate) = false

    override suspend fun getListOfWords(subtype: Subtype) = emptyList<String>()

    override suspend fun getFrequencyForWord(subtype: Subtype, word: String) = 0.0

    override suspend fun destroy() {
        cachedEmojiData.invalidateAll()
    }

    companion object {
        /**
         * The emoji [data] suggests for the [composingText], best first, at most [maxCandidateCount]: none unless
         * the text is the prefix and then a query of at least [minLength] ASCII letters (`validateInputQuery`), else
         * those shown for [skinTone] whose name or keywords hold the query, scored and sorted in Rust (see
         * EmojiRules.validateQuery and EmojiTable.suggest).
         */
        internal fun suggestEmojis(
            data: EmojiData?,
            composingText: CharSequence,
            prefix: String,
            minLength: Int,
            skinTone: EmojiSkinTone,
            maxCandidateCount: Int,
        ): List<Emoji> {
            val query = EmojiRules.validateQuery(composingText.toString(), prefix, minLength) ?: return emptyList()
            val table = data?.table
            if (data == null || table == null) {
                // No emoji to search; a negative count still throws, as Java's Stream.limit did.
                require(maxCandidateCount >= 0) { maxCandidateCount.toString() }
                return emptyList()
            }
            val emojis = data.bySkinTone.getValue(skinTone)
            return table.suggest(query, skinTone.ordinal, maxCandidateCount).map { emojis[it] }
        }
    }
}
