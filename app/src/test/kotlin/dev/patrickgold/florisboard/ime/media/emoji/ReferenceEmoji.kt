package dev.patrickgold.florisboard.ime.media.emoji

import java.io.BufferedReader
import java.util.EnumMap
import java.util.stream.Collectors
import kotlin.streams.toList

/**
 * The emoji rules as FlorisBoard's Kotlin had them before they moved to Rust, copied verbatim
 * with only what it takes to stand alone: `Emoji`'s tones, `EmojiData.loadEmojiDataMap` (reading
 * a [BufferedReader] instead of an asset) and `resolveEmojiAssetPath` (given the asset names and
 * the locale's parts), and `EmojiSuggestionProvider`'s `validateInputQuery` (given the prefix and
 * minimum) and scoring, without the coroutine. [EmojiComparison] holds the Rust up against it.
 */
class ReferenceEmoji(val value: String, val name: String, val keywords: List<String>) {
    val skinTone: EmojiSkinTone

    val hairStyle: EmojiHairStyle

    init {
        val codePoints = value.codePoints().toList()
        skinTone = EmojiSkinTone.entries.firstOrNull { codePoints.contains(it.id) } ?: EmojiSkinTone.DEFAULT
        hairStyle = EmojiHairStyle.entries.firstOrNull { codePoints.contains(it.id) } ?: EmojiHairStyle.DEFAULT
    }
}

class ReferenceEmojiData(
    val byCategory: Map<EmojiCategory, List<List<ReferenceEmoji>>>,
    val bySkinTone: Map<EmojiSkinTone, List<ReferenceEmoji>>,
) {
    companion object {
        private fun newByCategory(): EnumMap<EmojiCategory, MutableList<List<ReferenceEmoji>>> {
            return EnumMap<EmojiCategory, MutableList<List<ReferenceEmoji>>>(EmojiCategory::class.java).also { map ->
                for (category in EmojiCategory.entries) {
                    map[category] = mutableListOf()
                }
            }
        }

        private fun newBySkinTone(): EnumMap<EmojiSkinTone, MutableList<ReferenceEmoji>> {
            return EnumMap<EmojiSkinTone, MutableList<ReferenceEmoji>>(EmojiSkinTone::class.java).also { map ->
                for (skinTone in EmojiSkinTone.entries) {
                    map[skinTone] = mutableListOf()
                }
            }
        }

        fun loadEmojiDataMap(reader: BufferedReader): ReferenceEmojiData {
            val byCategory = newByCategory()
            val bySkinTone = newBySkinTone()

            var ec: EmojiCategory? = null
            var emojiEditorList: MutableList<ReferenceEmoji>? = null

            fun commitEmojiEditorList() {
                emojiEditorList?.let { byCategory[ec]!!.add(it) }
                emojiEditorList = null
            }

            reader.useLines { lines ->
                for (line in lines) {
                    if (line.startsWith("#")) {
                        // Comment line
                    } else if (line.startsWith("[")) {
                        commitEmojiEditorList()
                        ec = EmojiCategory.entries.find { it.id == line.slice(1 until (line.length - 1)) }
                    } else if (line.trim().isEmpty() || ec == null) {
                        // Empty line
                        continue
                    } else {
                        if (!line.startsWith("\t")) {
                            commitEmojiEditorList()
                        }
                        // Assume it is a data line
                        val data = line.split(";")
                        if (data.size == 3) {
                            val base = emojiEditorList?.first()
                            val emoji = ReferenceEmoji(
                                value = data[0].trim(),
                                name = base?.name ?: data[1].trim(),
                                keywords = data[2].split("|").map { it.trim() },
                            )
                            if (emojiEditorList != null) {
                                emojiEditorList!!.add(emoji)
                            } else {
                                emojiEditorList = mutableListOf(emoji)
                            }
                        }
                    }
                }
                commitEmojiEditorList()
            }

            for (category in byCategory.keys) {
                for (emojiSet in byCategory[category]!!) {
                    if (emojiSet.size == 1) {
                        // No variations provided, we fallback to using the base for all skin tones
                        val base = emojiSet.first()
                        for (skinTone in EmojiSkinTone.entries) {
                            bySkinTone[skinTone]!!.add(base)
                        }
                        continue
                    }
                    for (emoji in emojiSet) {
                        bySkinTone[emoji.skinTone]!!.add(emoji)
                    }
                }
            }

            return ReferenceEmojiData(byCategory, bySkinTone)
        }

        fun resolveEmojiAssetPath(emojiAssets: List<String>, localeLanguage: String, localeCountry: String, localeVariant: String): String? {
            val makePath = { file: String -> "ime/media/emoji/$file" }
            val language = localeLanguage.lowercase()
            val country = localeCountry.takeIf { it.isNotBlank() }
            val variant = localeVariant.takeIf { it.isNotBlank() }
            if (variant != null && country != null) {
                "${language}_${country}_${variant}.txt".takeIf { emojiAssets.contains(it) }?.let {
                    return makePath(it)
                }
            }
            if (country != null) {
                "${language}_${country}.txt".takeIf { emojiAssets.contains(it) }?.let { return makePath(it) }
            }
            "${language}.txt".takeIf { emojiAssets.contains(it) }?.let {
                return makePath(it)
            }
            return null
        }
    }
}

object ReferenceEmojiSuggestions {
    private val lettersRegex = "^[A-Za-z]*$".toRegex()

    fun validateInputQuery(composingText: CharSequence, prefix: String, minLength: Int): String? {
        val queryMinLength = minLength + prefix.length
        if (prefix.isNotEmpty() && !composingText.startsWith(prefix)) {
            return null
        }
        if (composingText.length < queryMinLength) {
            return null
        }
        val emojiPartialName = composingText.substring(prefix.length)
        if (!lettersRegex.matches(emojiPartialName)) {
            return null
        }
        return emojiPartialName
    }

    /** `suggest`, from the query on: the emoji for [query] among [emojis], best first. */
    fun rank(emojis: List<ReferenceEmoji>, query: String, maxCandidateCount: Int): List<ReferenceEmoji> {
        return emojis.parallelStream()
            .map { emoji ->
                val nameWeight = emoji.name.containsWeighted(query, ignoreCase = true)
                val keywordWeight = emoji.keywords
                    .any { it.contains(query, ignoreCase = true) }
                    .let { if (it) 1.0 else 0.0 }
                emoji to (nameWeight * 0.7 + keywordWeight * 0.3)
            }
            .sorted { (_, a), (_, b) -> b.compareTo(a) }
            .limit(maxCandidateCount.toLong())
            .filter { (_, a) -> a > 0 }
            .map { (emoji, _) -> emoji }
            .collect(Collectors.toList())
    }

    /** `suggest`: the composing text's query, if any, ranked among the emoji for [skinTone]. */
    fun suggest(
        data: ReferenceEmojiData?,
        composingText: CharSequence,
        prefix: String,
        minLength: Int,
        skinTone: EmojiSkinTone,
        maxCandidateCount: Int,
    ): List<ReferenceEmoji> {
        val query = validateInputQuery(composingText, prefix, minLength) ?: return emptyList()
        val emojis = data?.bySkinTone?.get(skinTone) ?: emptyList()
        return rank(emojis, query, maxCandidateCount)
    }
}

private fun String.containsWeighted(other: String, ignoreCase: Boolean = false): Double = let { str ->
    if (str.contains(other, ignoreCase = ignoreCase)) {
        other.length.toDouble() / str.length.toDouble()
    } else {
        0.0
    }
}
