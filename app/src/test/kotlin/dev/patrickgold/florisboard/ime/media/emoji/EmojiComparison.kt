package dev.patrickgold.florisboard.ime.media.emoji

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.EmojiRules
import kotlin.random.Random

/**
 * Asks emoji data and suggestions in Rust ([EmojiData.parse], [Emoji]'s tones,
 * [EmojiData.resolveEmojiAssetPath], [EmojiSuggestionProvider.suggestEmojis] and the table's own
 * ranking) what the Kotlin they replaced (`ReferenceEmoji`) was asked, and records each answer on
 * [tape], whose golden file holds that reference's answers (docs/phase6-kotlin-references.md). It
 * also counts how often each outcome was seen, so a caller can tell the runs reached them all.
 */
class EmojiComparison(private val tape: GoldenTape) {
    val seen = mutableMapOf<String, Int>()

    private fun check(what: String, rust: Any?) = tape.answer(what, rust)

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    /** A bundled file's [text]: parsed, then [queries] random suggestions over it. */
    fun file(text: String, random: Random, queries: Int) {
        val data = parse(text)
        see("bundled file")
        repeat(queries) { query(random, data) }
    }

    /**
     * [runs] files cut from [texts] (the bundled files) and damaged: lines dropped, blank, commented,
     * new or broken category lines, tabs, separators and fields added, dropped or emptied, odd
     * spaces and letters whose case maps oddly, tones, NUL and lone surrogates in values, and every
     * kind of line end. Each is parsed and asked [queries] random suggestions.
     */
    fun damaged(random: Random, texts: List<String>, runs: Int, queries: Int) = repeat(runs) {
        val data = parse(EmojiInputs.damagedFile(random, texts.random(random)))
        repeat(queries) { query(random, data) }
    }

    private fun parse(text: String): EmojiData {
        val rust = EmojiData.parse(text)
        for (category in EmojiCategory.entries) {
            val sets = rust.byCategory.getValue(category).map { set -> set.emojis.map(::describe) }
            check("parse $category", sets)
            for (set in sets) see(if (set.size == 1) "parse: single" else "parse: variants")
        }
        for (skinTone in EmojiSkinTone.entries) {
            check("bySkinTone $skinTone", rust.bySkinTone.getValue(skinTone).map(::describe))
        }
        val emojis = rust.byCategory.values.flatten().flatMap { it.emojis }
        if (emojis.any { it.name.isEmpty() }) see("parse: empty name")
        if (emojis.any { it.value.contains('\u0000') }) see("parse: nul")
        return rust
    }

    /** One random suggestion through the provider's path, and one random ranking of a raw query. */
    private fun query(random: Random, rust: EmojiData) {
        val skinTone = EmojiSkinTone.entries.random(random)
        val emojis = rust.bySkinTone.getValue(skinTone)
        val query = EmojiInputs.query(random, emojis)
        val prefix = EmojiSuggestionType.entries.random(random).prefix
        val composing = EmojiInputs.composing(random, query, prefix)
        val minLength = MIN_LENGTHS.random(random)
        val max = MAXES.random(random)
        val found = outcome {
            EmojiSuggestionProvider.suggestEmojis(rust, composing, prefix, minLength, skinTone, max).map(::describe)
        }
        check("suggest", found)
        val validated = EmojiRules.validateQuery(composing, prefix, minLength)
        see(if (validated == null) "suggest: no query" else "suggest: ${kind(found, max)}")

        // The ranking alone, for queries no composing text gets past validation with.
        val table = rust.table ?: return
        val ranked = outcome { table.suggest(query, skinTone.ordinal, max).map { describe(emojis[it]) } }
        check("rank", ranked)
        see("rank: ${kind(ranked, max)}")
        if (query.isEmpty() && ranked is List<*> && max > 0) {
            val all = table.suggest(query, skinTone.ordinal, Int.MAX_VALUE).size
            if (ranked.size < minOf(max, all)) see("rank: a NaN took a place")
        }
    }

    private fun kind(found: Any?, max: Int): String = when {
        found !is List<*> -> "threw"
        found.isEmpty() -> "none"
        found.size == max -> "max"
        else -> "some"
    }

    /**
     * Validation alone: random composing texts, prefixes and minimum lengths. Then suggestions for
     * them with no emoji data (a locale with no file), which still throw for a negative count.
     */
    fun queries(random: Random, runs: Int) = repeat(runs) {
        val prefix = EmojiSuggestionType.entries.random(random).prefix
        val composing = EmojiInputs.composing(random, EmojiInputs.query(random, emptyList()), prefix)
        val minLength = MIN_LENGTHS.random(random)
        val validated = EmojiRules.validateQuery(composing, prefix, minLength)
        check("validateQuery", validated)
        see(if (validated == null) "query: none" else "query: valid")

        val skinTone = EmojiSkinTone.entries.random(random)
        val max = MAXES.random(random)
        val data = if (random.nextBoolean()) null else EmojiData.empty()
        val found = outcome {
            EmojiSuggestionProvider.suggestEmojis(data, composing, prefix, minLength, skinTone, max).map(::describe)
        }
        check("suggest without data", found)
        if (validated != null) see("no data: ${kind(found, max)}")
    }

    /** An emoji's skin tone and hair style, for random strings of tones, emoji, NUL and surrogate halves. */
    fun tones(random: Random, runs: Int) = repeat(runs) {
        val rust = Emoji(EmojiInputs.toneValue(random), "", emptyList())
        check("tones", rust.skinTone to rust.hairStyle)
        see("skin ${rust.skinTone}")
        see("hair ${rust.hairStyle}")
    }

    /** Which file a locale reads, among random lists of files that sometimes hold its candidates. */
    fun assets(random: Random, runs: Int) = repeat(runs) {
        val (language, country, variant) = EmojiInputs.locale(random)
        val assets = EmojiInputs.assets(random, language, country, variant)
        val rust = EmojiData.resolveEmojiAssetPath(assets, language, country, variant)
        check("resolveEmojiAssetPath", rust)
        see(
            when (rust?.count { it == '_' }) {
                null -> "asset: none"
                0 -> "asset: language"
                1 -> "asset: country"
                else -> "asset: variant"
            },
        )
    }

    private fun outcome(block: () -> List<String>): Any = try {
        block()
    } catch (e: IllegalArgumentException) {
        "threw IllegalArgumentException"
    }

    private companion object {
        val MIN_LENGTHS = listOf(Int.MIN_VALUE, -1, 0, 0, 0, 1, 1, 1, 2, 2, 3, 3, 3, 4, 6, 40, Int.MAX_VALUE - 1, Int.MAX_VALUE)
        val MAXES = listOf(-1, 0, 1, 2, 3, 5, 8, 8, 8, 16, 100, 100_000)

        fun describe(emoji: Emoji) =
            "${emoji.value}|${emoji.name}|${emoji.keywords}|${emoji.skinTone}|${emoji.hairStyle}"
    }
}
