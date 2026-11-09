package com.jtn0123.audiowhisper.gif

import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json
import java.io.File
import java.io.IOException
import java.util.Locale

/**
 * The GIF panel's rules as they were in Kotlin before moving to Rust (lib/rust/crates/gif),
 * copied unchanged from commit 68e56337: GifSearch.kt's SearchText, categories, chips and
 * problem, GifSearchKeys.kt, GifResultCache.kt and GifRecents.kt. They are what
 * [GifRulesMatchTest] holds the Rust to; don't change them. [Gif], [GiphyException],
 * [GiphyFormatException], [KeyCode], [KeyType] and [SwipeAction] are the app's.
 */
object ReferenceGif {
    /** The quick searches under the search field; "Trending" is GIPHY's trending list. */
    val categories = listOf(
        "Trending", "Haha", "Yes", "No", "Thank you", "Love", "Wow", "Sad",
        "Hi", "Bye", "Congrats", "Dance", "Facepalm", "OK",
    )
    const val TRENDING = "Trending"

    /** The GIFs sent before ([GifRecents]): a chip that needs no GIPHY call. */
    const val RECENTS = "Recents"

    /**
     * [typedWord] goes first, unless it is one of the chips already, which then moves first
     * instead ("congrats" brings Congrats forward).
     */
    fun chips(typedWord: String?, hasRecents: Boolean): List<String> {
        val rest = (if (hasRecents) listOf(RECENTS) else emptyList()) + categories
        if (typedWord == null) return rest
        val same = rest.firstOrNull { it.equals(typedWord, ignoreCase = true) }
        return if (same != null) listOf(same) + (rest - same) else listOf(typedWord) + rest
    }

    fun problem(e: IOException) = when (e) {
        is GiphyFormatException -> "GIPHY sent something unreadable. Try again later."
        is GiphyException -> when (e.status) {
            401, 403 -> "GIPHY didn't accept the API key."
            429 -> "GIPHY's hourly limit is used up. Try again later."
            else -> "GIPHY isn't answering right now."
        }
        else -> "Couldn't reach GIPHY. Check your connection."
    }

    /** Editing the search field, kept free of Android so it can be unit tested. */
    object SearchText {
        const val MAX_LENGTH = 50

        fun type(query: String, text: String): String = (query + text).take(MAX_LENGTH)

        fun typeWord(query: String, word: String): String {
            val separator = if (query.isEmpty() || query.endsWith(' ')) "" else " "
            return type(query, separator + word)
        }

        fun deleteLast(query: String): String {
            if (query.isEmpty()) return query
            val end = if (query.length >= 2 && Character.isSurrogatePair(query[query.length - 2], query.last())) 2 else 1
            return query.dropLast(end)
        }

        fun deleteLast(query: String, count: Int): String {
            var left = query
            repeat(count) { left = deleteLast(left) }
            return left
        }

        /** Deletes [count] words from the end, and the spaces after them; the space before stays. */
        fun deleteWords(query: String, count: Int): String {
            var left = query
            repeat(count) { left = left.trimEnd().dropLastWhile { !it.isWhitespace() } }
            return left
        }

        /**
         * The word just typed before the cursor, to offer as a GIF search: "congrats" from "Big
         * congrats! ". Null when the text doesn't end in a word of [MIN_WORD]-[MAX_WORD] letters.
         */
        fun lastWord(text: CharSequence?): String? {
            val end = text?.toString()?.trimEnd { it.isWhitespace() || it in ".,!?;:" } ?: return null
            val word = end.takeLastWhile { it.isLetter() || it == '\'' }.trim('\'')
            return word.takeIf { it.length in MIN_WORD..MAX_WORD }
        }

        private const val MIN_WORD = 2
        private const val MAX_WORD = 30
    }

    /**
     * What the keyboard does while the user types into the GIF search box.
     *
     * The app's own text field is still connected behind the panel, so a key or
     * swipe that reached it would edit text the user can't see. Keys are
     * therefore let through only when known to be safe; anything else, including
     * keys added later, does nothing.
     */
    object GifSearchKeys {
        enum class Route {
            /** Edits the search query. */
            SEARCH,

            /** Works as usual: it changes the keyboard, not the text (shift, layouts, languages, hiding). */
            KEYBOARD,

            /** Would edit the app's text (cursor keys, undo, paste, dictation), so it does nothing. */
            IGNORE,
        }

        /** How far a delete-key swipe has reached back in the query: [count] characters, or words. */
        data class SwipeDelete(val count: Int, val words: Boolean)

        private val SEARCH_KEYS = setOf(
            KeyCode.DELETE, KeyCode.DELETE_WORD, KeyCode.SPACE, KeyCode.CJK_SPACE, KeyCode.ENTER,
        )

        private val KEYBOARD_KEYS = setOf(
            KeyCode.SHIFT, KeyCode.CAPS_LOCK,
            KeyCode.VIEW_CHARACTERS, KeyCode.VIEW_SYMBOLS, KeyCode.VIEW_SYMBOLS2, KeyCode.VIEW_NUMERIC,
            KeyCode.VIEW_NUMERIC_ADVANCED, KeyCode.VIEW_PHONE, KeyCode.VIEW_PHONE2,
            KeyCode.IME_UI_MODE_TEXT, KeyCode.IME_UI_MODE_MEDIA, KeyCode.IME_UI_MODE_CLIPBOARD,
            KeyCode.IME_SHOW_UI, KeyCode.IME_HIDE_UI,
            KeyCode.IME_PREV_SUBTYPE, KeyCode.IME_NEXT_SUBTYPE, KeyCode.LANGUAGE_SWITCH,
            KeyCode.IME_SUBTYPE_PICKER, KeyCode.SHOW_SUBTYPE_PICKER,
            KeyCode.SYSTEM_INPUT_METHOD_PICKER, KeyCode.SYSTEM_PREV_INPUT_METHOD, KeyCode.SYSTEM_NEXT_INPUT_METHOD,
            KeyCode.SETTINGS, KeyCode.TOGGLE_SMARTBAR_VISIBILITY, KeyCode.TOGGLE_INCOGNITO_MODE, KeyCode.TOGGLE_AUTOCORRECT,
            KeyCode.TOGGLE_COMPACT_LAYOUT, KeyCode.COMPACT_LAYOUT_TO_LEFT, KeyCode.COMPACT_LAYOUT_TO_RIGHT,
            KeyCode.CHAR_WIDTH_SWITCHER, KeyCode.CHAR_WIDTH_FULL, KeyCode.CHAR_WIDTH_HALF,
            KeyCode.KANA_SWITCHER, KeyCode.KANA_HIRA, KeyCode.KANA_KATA, KeyCode.KANA_HALF_KATA,
        )

        fun route(code: Int, type: KeyType): Route = when {
            code in SEARCH_KEYS -> Route.SEARCH
            code in KEYBOARD_KEYS -> Route.KEYBOARD
            // Control characters such as tab and escape aren't search text.
            code in 1 until KeyCode.SPACE -> Route.IGNORE
            type == KeyType.CHARACTER || type == KeyType.NUMERIC -> Route.SEARCH
            else -> Route.IGNORE
        }

        /**
         * A delete-key swipe, [unitsX] key widths from where it started (negative is left), as
         * the same amount the app's field would have selected; null if [action] isn't a precise
         * delete. A precise select deletes nothing: there's nothing to select in the search box.
         */
        fun swipeDelete(action: SwipeAction, unitsX: Int): SwipeDelete? = when (action) {
            SwipeAction.DELETE_CHARACTERS_PRECISELY -> SwipeDelete((-unitsX - 1).coerceAtLeast(0), words = false)
            SwipeAction.DELETE_WORDS_PRECISELY -> SwipeDelete((-unitsX / 2 - 1).coerceAtLeast(0), words = true)
            SwipeAction.SELECT_CHARACTERS_PRECISELY, SwipeAction.SELECT_WORDS_PRECISELY -> SwipeDelete(0, words = false)
            else -> null
        }
    }

    /**
     * The GIFs each recent search returned, so tapping back to a category, or ✕ back to trending,
     * doesn't spend another of GIPHY's 100 calls an hour. Entries last [ttlMs]: trending changes,
     * and a search may have new GIFs. In memory only, and at most [capacity] searches.
     */
    class GifResultCache(
        private val clock: () -> Long,
        private val ttlMs: Long = TTL_MS,
        private val capacity: Int = CAPACITY,
    ) {
        private class Entry(val gifs: List<Gif>, val at: Long)

        /** Least recently used first, so the eldest goes when full. */
        private val entries = LinkedHashMap<String, Entry>(capacity, 0.75f, true)

        /** What [search] returned within [ttlMs], or null. "Cats" and "cats " are the same search. */
        fun get(search: String): List<Gif>? {
            val key = key(search)
            val entry = entries[key] ?: return null
            if (clock() - entry.at >= ttlMs) {
                entries.remove(key)
                return null
            }
            return entry.gifs
        }

        val size: Int get() = entries.size

        fun put(search: String, gifs: List<Gif>) {
            entries[key(search)] = Entry(gifs, clock())
            while (entries.size > capacity) entries.remove(entries.keys.first())
        }

        private fun key(search: String) = search.trim().lowercase(Locale.ROOT)

        companion object {
            const val TTL_MS = 10 * 60 * 1000L
            const val CAPACITY = 20
        }
    }

    /**
     * The GIFs the user sent, newest first, so they can be sent again without a GIPHY call.
     * Stored in [file] (files/gif-recents.json, never backed up), never logged, and not added to
     * in incognito. Loaded on a background thread and used on the main one, hence the locking.
     */
    class GifRecents(private val file: File?, private val capacity: Int = CAPACITY) {
        private val gifs = ArrayList<Gif>()

        @Synchronized
        fun all(): List<Gif> = gifs.toList()

        fun isStoredIn(other: File) = file == other

        /** [gif] was sent: it goes first, once. */
        @Synchronized
        fun add(gif: Gif) {
            gifs.removeAll { it.id == gif.id }
            gifs.add(0, gif)
            while (gifs.size > capacity) gifs.removeAt(gifs.lastIndex)
        }

        /** Reads [file], keeping anything sent before it finished. A missing or damaged file is no GIFs. */
        fun load() {
            val saved = try {
                file?.takeIf { it.exists() }?.readText()?.let { json.decodeFromString<List<Gif>>(it) }
            } catch (_: IOException) {
                null
            } catch (_: SerializationException) {
                null
            } ?: return
            synchronized(this) {
                val sentMeanwhile = gifs.toList()
                gifs.clear()
                gifs.addAll(saved.take(capacity))
                sentMeanwhile.asReversed().forEach(::add)
            }
        }

        /** Writes [file] whole, through a temporary file so a crash mid-write can't damage it. */
        fun save(): Boolean {
            val target = file ?: return false
            val text = json.encodeToString(all())
            return try {
                val part = File(target.parentFile, "${target.name}.part")
                part.writeText(text)
                part.renameTo(target)
            } catch (_: IOException) {
                false
            }
        }

        companion object {
            const val CAPACITY = 30
            private val json = Json { ignoreUnknownKeys = true }
        }
    }
}
