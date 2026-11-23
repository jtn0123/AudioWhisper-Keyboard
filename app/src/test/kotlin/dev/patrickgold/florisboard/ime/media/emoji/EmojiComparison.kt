package dev.patrickgold.florisboard.ime.media.emoji

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.BufferedReader
import java.io.StringReader
import kotlin.random.Random

/**
 * Puts emoji data and suggestions in Rust ([EmojiData.parse], [Emoji]'s tones,
 * [EmojiData.resolveEmojiAssetPath], [EmojiSuggestionProvider.suggestEmojis] and the table's own
 * ranking) up against the Kotlin they replaced ([ReferenceEmoji]), wherever this runs: the JVM's
 * tests and, copied in for a check, the phone. It records each difference and how often each
 * outcome was seen, so a caller can tell the runs reached them all. The reference's answers also
 * go on [tape] (docs/phase6-kotlin-references.md).
 */
class EmojiComparison(private val tape: GoldenTape) {
    val differences = mutableListOf<String>()
    val seen = mutableMapOf<String, Int>()

    private fun check(what: String, input: String, kotlin: Any?, rust: Any?) {
        tape.answer(what, kotlin)
        if (kotlin != rust && differences.size < MAX_DIFFERENCES) differences += "$what($input): kotlin $kotlin, rust $rust"
    }

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    /** A bundled file, [name]: parsed both ways, then [queries] random suggestions over it. */
    fun file(name: String, text: String, random: Random, queries: Int) {
        val (kotlin, rust) = parseBoth(name, text)
        see("bundled file")
        repeat(queries) { query(random, kotlin, rust, name) }
    }

    /**
     * [runs] files cut from [texts] (the bundled files) and damaged: lines dropped, blank, commented,
     * new or broken category lines, tabs, separators and fields added, dropped or emptied, odd
     * spaces and letters whose case maps oddly, tones, NUL and lone surrogates in values, and every
     * kind of line end. Each is parsed both ways and asked [queries] random suggestions.
     */
    fun damaged(random: Random, texts: List<String>, runs: Int, queries: Int) = repeat(runs) { run ->
        val text = EmojiInputs.damagedFile(random, texts.random(random))
        val (kotlin, rust) = parseBoth("damaged $run", text)
        repeat(queries) { query(random, kotlin, rust, "damaged $run") }
    }

    private fun parseBoth(name: String, text: String): Pair<ReferenceEmojiData, EmojiData> {
        val kotlin = ReferenceEmojiData.loadEmojiDataMap(BufferedReader(StringReader(text)))
        val rust = EmojiData.parse(text)
        for (category in EmojiCategory.entries) {
            val kotlinSets = kotlin.byCategory.getValue(category).map { set -> set.map(::describe) }
            val rustSets = rust.byCategory.getValue(category).map { set -> set.emojis.map(::describe) }
            check("parse $category", name, kotlinSets, rustSets)
            for (set in kotlinSets) see(if (set.size == 1) "parse: single" else "parse: variants")
        }
        for (skinTone in EmojiSkinTone.entries) {
            val kotlinEmojis = kotlin.bySkinTone.getValue(skinTone).map(::describe)
            check("bySkinTone $skinTone", name, kotlinEmojis, rust.bySkinTone.getValue(skinTone).map(::describe))
        }
        val emojis = kotlin.byCategory.values.flatten().flatten()
        if (emojis.any { it.name.isEmpty() }) see("parse: empty name")
        if (emojis.any { it.value.contains('\u0000') }) see("parse: nul")
        return kotlin to rust
    }

    /** One random suggestion through the provider's path, and one random ranking of a raw query. */
    private fun query(random: Random, kotlin: ReferenceEmojiData, rust: EmojiData, name: String) {
        val skinTone = EmojiSkinTone.entries.random(random)
        val emojis = kotlin.bySkinTone.getValue(skinTone)
        val query = EmojiInputs.query(random, emojis)
        val prefix = EmojiSuggestionType.entries.random(random).prefix
        val composing = EmojiInputs.composing(random, query, prefix)
        val minLength = MIN_LENGTHS.random(random)
        val max = MAXES.random(random)
        val input = "$name, ${skinTone.ordinal}, ${composing.length} units, prefix ${prefix.length}, min $minLength, max $max"
        val kotlinFound = outcome {
            ReferenceEmojiSuggestions.suggest(kotlin, composing, prefix, minLength, skinTone, max).map(::describe)
        }
        val rustFound = outcome {
            EmojiSuggestionProvider.suggestEmojis(rust, composing, prefix, minLength, skinTone, max).map(::describe)
        }
        check("suggest", input, kotlinFound, rustFound)
        val validated = ReferenceEmojiSuggestions.validateInputQuery(composing, prefix, minLength)
        see(if (validated == null) "suggest: no query" else "suggest: ${kind(kotlinFound, max)}")

        // The ranking alone, for queries no composing text gets past validation with.
        val table = rust.table ?: return
        val kotlinRanked = outcome { ReferenceEmojiSuggestions.rank(emojis, query, max).map(::describe) }
        val rustRanked = outcome {
            table.suggest(query, skinTone.ordinal, max).map { describe(rust.bySkinTone.getValue(skinTone)[it]) }
        }
        check("rank", "$input, query of ${query.length} units", kotlinRanked, rustRanked)
        see("rank: ${kind(kotlinRanked, max)}")
        if (query.isEmpty() && kotlinRanked is List<*> && max > 0) {
            val all = ReferenceEmojiSuggestions.rank(emojis, query, Int.MAX_VALUE).size
            if (kotlinRanked.size < minOf(max, all)) see("rank: a NaN took a place")
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
        val input = "${composing.length} units, prefix ${prefix.length}, min $minLength"
        val kotlin = ReferenceEmojiSuggestions.validateInputQuery(composing, prefix, minLength)
        val rust = com.jtn0123.audiowhisper.rust.EmojiRules.validateQuery(composing, prefix, minLength)
        check("validateQuery", input, kotlin, rust)
        see(if (kotlin == null) "query: none" else "query: valid")

        val skinTone = EmojiSkinTone.entries.random(random)
        val max = MAXES.random(random)
        val data = if (random.nextBoolean()) null else EmojiData.empty()
        val kotlinFound = outcome {
            ReferenceEmojiSuggestions.suggest(null, composing, prefix, minLength, skinTone, max).map(::describe)
        }
        val rustFound = outcome {
            EmojiSuggestionProvider.suggestEmojis(data, composing, prefix, minLength, skinTone, max).map(::describe)
        }
        check("suggest without data", "$input, max $max", kotlinFound, rustFound)
        if (kotlin != null) see("no data: ${kind(kotlinFound, max)}")
    }

    /** An emoji's skin tone and hair style, for random strings of tones, emoji, NUL and surrogate halves. */
    fun tones(random: Random, runs: Int) = repeat(runs) {
        val value = EmojiInputs.toneValue(random)
        val kotlin = ReferenceEmoji(value, "", emptyList())
        val rust = Emoji(value, "", emptyList())
        check("tones", value.map { it.code }.toString(), kotlin.skinTone to kotlin.hairStyle, rust.skinTone to rust.hairStyle)
        see("skin ${kotlin.skinTone}")
        see("hair ${kotlin.hairStyle}")
    }

    /** Which file a locale reads, among random lists of files that sometimes hold its candidates. */
    fun assets(random: Random, runs: Int) = repeat(runs) {
        val (language, country, variant) = EmojiInputs.locale(random)
        val assets = EmojiInputs.assets(random, language, country, variant)
        val kotlin = ReferenceEmojiData.resolveEmojiAssetPath(assets, language, country, variant)
        val rust = EmojiData.resolveEmojiAssetPath(assets, language, country, variant)
        check("resolveEmojiAssetPath", "${assets.size} files, ${listOf(language, country, variant).map { it.length }}", kotlin, rust)
        see(
            when (kotlin?.count { it == '_' }) {
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
        const val MAX_DIFFERENCES = 20
        val MIN_LENGTHS = listOf(Int.MIN_VALUE, -1, 0, 0, 0, 1, 1, 1, 2, 2, 3, 3, 3, 4, 6, 40, Int.MAX_VALUE - 1, Int.MAX_VALUE)
        val MAXES = listOf(-1, 0, 1, 2, 3, 5, 8, 8, 8, 16, 100, 100_000)

        fun describe(emoji: ReferenceEmoji) =
            "${emoji.value}|${emoji.name}|${emoji.keywords}|${emoji.skinTone}|${emoji.hairStyle}"

        fun describe(emoji: Emoji) =
            "${emoji.value}|${emoji.name}|${emoji.keywords}|${emoji.skinTone}|${emoji.hairStyle}"
    }
}
