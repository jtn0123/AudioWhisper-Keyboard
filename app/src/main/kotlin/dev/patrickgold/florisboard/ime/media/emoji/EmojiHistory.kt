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

import dev.patrickgold.florisboard.app.FlorisPreferenceModel
import dev.patrickgold.florisboard.lib.devtools.flogError
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
data class EmojiHistory(
    val pinned: List<@Serializable(with = Emoji.ValueOnlySerializer::class) Emoji>,
    val recent: List<@Serializable(with = Emoji.ValueOnlySerializer::class) Emoji>,
) {
    fun edit(): Editor {
        return Editor(pinned.toMutableList(), recent.toMutableList())
    }

    data class Editor(
        val pinned: MutableList<Emoji>,
        val recent: MutableList<Emoji>,
    ) {
        fun build(): EmojiHistory {
            return EmojiHistory(pinned.toList(), recent.toList())
        }
    }

    enum class UpdateStrategy(val isAutomatic: Boolean, val isPrepend: Boolean) {
        AUTO_SORT_PREPEND(isAutomatic = true, isPrepend = true),
        AUTO_SORT_APPEND(isAutomatic = true, isPrepend = false),
        MANUAL_SORT_PREPEND(isAutomatic = false, isPrepend = true),
        MANUAL_SORT_APPEND(isAutomatic = false, isPrepend = false);
    }

    object Serializer : PreferenceSerializer<EmojiHistory> {
        override fun serialize(value: EmojiHistory): String {
            return Json.encodeToString(value)
        }

        override fun deserialize(value: String): EmojiHistory {
            try {
                return Json.decodeFromString(value)
            } catch (e: Exception) {
                flogError { "Failed to deserialize EmojiHistory: ${e.javaClass.simpleName}" }
                return Empty
            }
        }
    }

    companion object {
        val Empty = EmojiHistory(emptyList(), emptyList())

        @Suppress("ConstPropertyName")
        const val MaxSizeUnlimited: Int = 0
    }
}

object EmojiHistoryHelper {
    private var emojiGuard = Mutex(locked = false)

    private suspend fun apply(prefs: FlorisPreferenceModel, op: Int, emoji: Emoji? = null, offset: Int = 0) = emojiGuard.withLock {
        val h = prefs.emoji.historyData.get()
        val pool = (h.pinned + h.recent + listOfNotNull(emoji)).distinct()
        val result = com.jtn0123.audiowhisper.rust.PanelRules.history(
            h.pinned.map { pool.indexOf(it) }.toIntArray(), h.recent.map { pool.indexOf(it) }.toIntArray(),
            intArrayOf(if (prefs.emoji.historyEnabled.get()) 1 else 0,
                prefs.emoji.historyPinnedUpdateStrategy.get().ordinal, prefs.emoji.historyRecentUpdateStrategy.get().ordinal,
                prefs.emoji.historyPinnedMaxSize.get(), prefs.emoji.historyRecentMaxSize.get(), op, pool.indexOf(emoji), offset),
        )
        prefs.emoji.historyData.set(EmojiHistory(result[0].map { pool[it] }, result[1].map { pool[it] }))
    }

    suspend fun markEmojiUsed(prefs: FlorisPreferenceModel, emoji: Emoji) = apply(prefs, 0, emoji)
    suspend fun pinEmoji(prefs: FlorisPreferenceModel, emoji: Emoji) = apply(prefs, 1, emoji)
    suspend fun unpinEmoji(prefs: FlorisPreferenceModel, emoji: Emoji) = apply(prefs, 2, emoji)
    suspend fun moveEmoji(prefs: FlorisPreferenceModel, emoji: Emoji, offset: Int) = apply(prefs, 3, emoji, offset)
    suspend fun removeEmoji(prefs: FlorisPreferenceModel, emoji: Emoji) = apply(prefs, 4, emoji)
    suspend fun deleteHistory(prefs: FlorisPreferenceModel) = apply(prefs, 5)
    suspend fun deletePinned(prefs: FlorisPreferenceModel) = apply(prefs, 6)
}
