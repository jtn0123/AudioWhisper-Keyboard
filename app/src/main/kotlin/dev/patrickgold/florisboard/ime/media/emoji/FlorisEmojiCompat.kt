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

package dev.patrickgold.florisboard.ime.media.emoji

import android.content.Context
import androidx.emoji2.text.DefaultEmojiCompatConfig
import androidx.emoji2.text.EmojiCompat
import dev.patrickgold.florisboard.lib.devtools.flogError
import dev.patrickgold.florisboard.lib.devtools.flogInfo
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * Loads the app's one EmojiCompat instance and publishes it once it is ready, so the UI can draw with it as soon as it
 * loads. It uses the default config and thus relies either on a system font with emoji or Google GMS services with
 * their downloadable font provider; without either, the palette falls back to querying the system painter.
 *
 * AudioWhisper: upstream kept two instances, one replacing every emoji and one only those the system font lacks, made
 * with EmojiCompat.reset(), an API restricted to the library, and swapped the default instance between them. One
 * instance does both: the replace strategy is chosen per call to [EmojiCompat.process], see [EmojiFont]. That loads
 * the emoji metadata once instead of twice.
 */
object FlorisEmojiCompat {
    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
    private val published = MutableStateFlow<EmojiCompat?>(null)

    /** The EmojiCompat instance once it has loaded; null while it loads, or if it failed. */
    val instance: StateFlow<EmojiCompat?> = published.asStateFlow()

    /** Starts loading the emoji metadata on a background thread. */
    fun init(context: Context) {
        val config = DefaultEmojiCompatConfig.create(context) ?: return flogError { "No emoji font provider found" }
        config.setMetadataLoadStrategy(EmojiCompat.LOAD_STRATEGY_MANUAL)
        config.registerInitCallback(object : EmojiCompat.InitCallback() {
            override fun onInitialized() {
                flogInfo { "EmojiCompat successfully loaded!" }
                published.value = EmojiCompat.get()
            }

            override fun onFailed(throwable: Throwable?) {
                flogError { "EmojiCompat failed to load: $throwable" }
            }
        })
        val emojiCompat = EmojiCompat.init(config)
        scope.launch { emojiCompat.load() }
    }
}

/**
 * Draws emojis with EmojiCompat's font: every emoji if the app being typed in asked for that (its editor sets
 * [EmojiCompat.EDITOR_INFO_REPLACE_ALL_KEY]), otherwise only those the system font can't draw.
 */
class EmojiFont(private val emojiCompat: EmojiCompat, replaceAll: Boolean) {
    private val strategy = if (replaceAll) EmojiCompat.REPLACE_STRATEGY_ALL else EmojiCompat.REPLACE_STRATEGY_NON_EXISTENT

    fun process(text: String): CharSequence =
        emojiCompat.process(text, 0, text.length, Int.MAX_VALUE, strategy) ?: text

    fun isSupported(emoji: String, metadataVersion: Int): Boolean =
        emojiCompat.getEmojiMatch(emoji, metadataVersion) == EmojiCompat.EMOJI_SUPPORTED
}
