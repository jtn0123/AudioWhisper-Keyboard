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
import com.jtn0123.audiowhisper.rust.EmojiTable
import dev.patrickgold.florisboard.lib.FlorisLocale
import org.florisboard.lib.android.readText
import io.github.reactivecircus.cache4k.Cache
import java.util.*

private typealias EmojiDataByCategoryImpl = EnumMap<EmojiCategory, MutableList<EmojiSet>>
private typealias EmojiDataBySkinToneImpl = EnumMap<EmojiSkinTone, MutableList<Emoji>>
typealias EmojiDataByCategory = Map<EmojiCategory, List<EmojiSet>>
typealias EmojiDataBySkinTone = Map<EmojiSkinTone, List<Emoji>>

data class EmojiData(
    val byCategory: EmojiDataByCategory,
    val bySkinTone: EmojiDataBySkinTone,
) {
    /** The file parsed in Rust, which emoji suggestions search; null for data not read from a file. */
    internal var table: EmojiTable? = null
        private set

    private constructor(byCategory: EmojiDataByCategory, bySkinTone: EmojiDataBySkinTone, table: EmojiTable) :
        this(byCategory, bySkinTone) {
        this.table = table
    }

    companion object {
        private val cache = Cache.Builder<String, EmojiData>().build()
        val Fallback = empty()

        private fun newByCategory(): EmojiDataByCategoryImpl {
            return EmojiDataByCategoryImpl(EmojiCategory::class.java).also { map ->
                for (category in EmojiCategory.entries) {
                    map[category] = mutableListOf()
                }
            }
        }

        private fun newBySkinTone(): EmojiDataBySkinToneImpl {
            return EmojiDataBySkinToneImpl(EmojiSkinTone::class.java).also { map ->
                for (skinTone in EmojiSkinTone.entries) {
                    map[skinTone] = mutableListOf()
                }
            }
        }

        fun empty(): EmojiData {
            return EmojiData(newByCategory(), newBySkinTone())
        }

        suspend fun get(context: Context, path: String): EmojiData {
            return cache.get(path) {
                loadEmojiDataMap(context, path)
            }
        }

        suspend fun get(context: Context, locale: FlorisLocale): EmojiData {
            val path = resolveEmojiAssetPath(context, locale) ?: return empty()
            return get(context, path)
        }

        private fun loadEmojiDataMap(context: Context, path: String): EmojiData {
            return parse(context.assets.readText(path))
        }

        /**
         * Parses an emoji file's [text] in Rust (lib/rust/crates/emoji, which documents the
         * format), and makes an [Emoji] of each emoji in it.
         */
        internal fun parse(text: String): EmojiData {
            val table = EmojiTable.parse(text, EmojiCategory.entries.map { it.id })
            val emojis = table.entries.map {
                Emoji(it.value, it.name, it.keywords, EmojiSkinTone.entries[it.skinTone], EmojiHairStyle.entries[it.hairStyle])
            }
            val byCategory = newByCategory()
            for (set in table.sets) {
                byCategory[EmojiCategory.entries[set.category]]!!.add(EmojiSet(emojis.subList(set.start, set.start + set.size)))
            }
            val bySkinTone = newBySkinTone()
            for (skinTone in EmojiSkinTone.entries) {
                table.bySkinTone[skinTone.ordinal].mapTo(bySkinTone[skinTone]!!) { emojis[it] }
            }
            return EmojiData(byCategory, bySkinTone, table)
        }

        /**
         * Resolves the path to the emoji asset file based on the active keyboard subtype and active locale.
         *
         * This method prioritizes usage of cached emoji layout maps when available. If a cached map is found then return,
         * the parseRawEmojiSpecsFile will correctly return the cached data
         *
         * It then attempts to locate a matching emoji asset file within the "ime/media/emoji/" directory in the
         * application's assets based on locale priority:
         * - Primary locale of the active subtype
         * - Secondary locales of the active subtype
         * - Root path ("ime/media/emoji/root.txt") as a fallback
         *
         * For each locale, file matching follows this preference order:
         * - {language}_{country}_{variant}.txt
         * - {language}_{country}.txt
         * - {language}.txt
         *
         * @param context The context used to access application assets.
         * @param locale The locale to resolve for.
         *
         * @return The path to the emoji asset file, or the root path ("ime/media/emoji/root.txt") if no match is found.
         */
        private fun resolveEmojiAssetPath(context: Context, locale: FlorisLocale): String? {
            val emojiAssets = context.assets.list("ime/media/emoji/")!!.toList()
            return resolveEmojiAssetPath(emojiAssets, locale.language, locale.country, locale.variant)
        }

        /** [resolveEmojiAssetPath] among the files [emojiAssets], for a locale's parts, chosen in Rust. */
        internal fun resolveEmojiAssetPath(emojiAssets: List<String>, language: String, country: String, variant: String): String? {
            val found = EmojiRules.resolveAsset(emojiAssets, language, country, variant)
            return if (found == EmojiRules.NO_ASSET) null else "ime/media/emoji/${emojiAssets[found]}"
        }
    }
}
