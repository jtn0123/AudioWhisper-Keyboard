package com.jtn0123.audiowhisper.gif

import android.os.SystemClock
import android.util.Log
import dev.patrickgold.florisboard.BuildConfig
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException

/**
 * What the GIF panel shows, and the search field the keyboard types into while it is open.
 * One keyboard per process, so the panel, the search bar and the key handler share this.
 * Main thread only.
 */
object GifSearch {
    private const val TAG = "AWGif"

    sealed interface Results {
        data object Loading : Results
        data class Loaded(val gifs: List<Gif>) : Results
        data class Failed(val message: String) : Results
    }

    /** The quick searches under the search field; "Trending" is GIPHY's trending list. */
    val categories = listOf(
        "Trending", "Haha", "Yes", "No", "Thank you", "Love", "Wow", "Sad",
        "Hi", "Bye", "Congrats", "Dance", "Facepalm", "OK",
    )
    const val TRENDING = "Trending"

    /** The GIFs sent before ([GifRecents]): a chip that needs no GIPHY call. */
    const val RECENTS = "Recents"

    private val giphy = BuildConfig.GIPHY_API_KEY.takeIf { it.isNotBlank() }?.let { Giphy(it) }

    /**
     * A GIPHY key was built in. Without one there is no GIF tab: nothing on the phone could
     * supply the key, so the tab could only ever say it was missing. See `giphy.apiKey` in
     * app/build.gradle.kts.
     */
    val available: Boolean get() = giphy != null
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var loading: Job? = null
    private val cache = GifResultCache(SystemClock::elapsedRealtime)
    private var recents = GifRecents(file = null)

    /** The query when the current delete-key swipe began, so swiping back restores it. */
    private var beforeSwipe = ""

    private val mutableShowingGifs = MutableStateFlow(false)
    private val mutableTyping = MutableStateFlow(false)
    private val mutableQuery = MutableStateFlow("")
    private val mutableShown = MutableStateFlow(TRENDING)
    private val mutableResults = MutableStateFlow<Results>(Results.Loading)
    private val mutableChips = MutableStateFlow(chips(typedWord = null, hasRecents = false))
    private var typedWord: String? = null

    /** The media panel shows GIFs rather than emoji. */
    val showingGifs: StateFlow<Boolean> = mutableShowingGifs.asStateFlow()

    /** The search field is open over the keyboard and typing goes into it. */
    val typing: StateFlow<Boolean> = mutableTyping.asStateFlow()

    /** What is in the search field. */
    val query: StateFlow<String> = mutableQuery.asStateFlow()

    /** The search (or [TRENDING]) whose GIFs are shown. */
    val shown: StateFlow<String> = mutableShown.asStateFlow()

    val results: StateFlow<Results> = mutableResults.asStateFlow()

    /** The quick searches to offer: the word just typed, Recents once a GIF was sent, then [categories]. */
    val chips: StateFlow<List<String>> = mutableChips.asStateFlow()

    /** The keyboard started: read the GIFs sent before from [filesDir]. Main thread. */
    fun attach(filesDir: File) {
        val file = File(filesDir, RECENTS_FILE)
        if (recents.isStoredIn(file)) return
        recents = GifRecents(file).also { loaded ->
            scope.launch {
                withContext(Dispatchers.IO) { loaded.load() }
                updateChips()
            }
        }
    }

    /**
     * The first time it opens, the panel shows Recents if there are any (no GIPHY call), else
     * trending.
     */
    fun showGifs(show: Boolean) {
        mutableShowingGifs.value = show && available
        if (!show || results.value is Results.Loaded) return
        val first = shown.value == TRENDING && recents.all().isNotEmpty()
        load(if (first) RECENTS else shown.value)
    }

    /** The panel opened after [textBefore]: offer its last word as a search ("congrats"). */
    fun offerTypedWord(textBefore: CharSequence?) {
        typedWord = SearchText.lastWord(textBefore)
        updateChips()
    }

    /** [gif] was sent. In incognito it isn't remembered. */
    fun sent(gif: Gif, incognito: Boolean) {
        if (incognito) return
        recents.add(gif)
        updateChips()
        val toSave = recents
        scope.launch(Dispatchers.IO) { toSave.save() }
    }

    private fun updateChips() {
        mutableChips.value = chips(typedWord, hasRecents = recents.all().isNotEmpty())
    }

    /**
     * [typedWord] goes first, unless it is one of the chips already, which then moves first
     * instead ("congrats" brings Congrats forward).
     */
    internal fun chips(typedWord: String?, hasRecents: Boolean): List<String> {
        val rest = (if (hasRecents) listOf(RECENTS) else emptyList()) + categories
        if (typedWord == null) return rest
        val same = rest.firstOrNull { it.equals(typedWord, ignoreCase = true) }
        return if (same != null) listOf(same) + (rest - same) else listOf(typedWord) + rest
    }

    fun startTyping() {
        mutableQuery.value = if (shown.value == TRENDING) "" else shown.value
        mutableTyping.value = true
    }

    fun stopTyping() {
        mutableTyping.value = false
    }

    fun type(text: String) {
        mutableQuery.value = SearchText.type(query.value, text)
    }

    /** A glide-typed word, spaced from the one before. */
    fun typeWord(word: String) {
        mutableQuery.value = SearchText.typeWord(query.value, word)
    }

    fun deleteLast() {
        mutableQuery.value = SearchText.deleteLast(query.value)
    }

    fun deleteWord() {
        mutableQuery.value = SearchText.deleteWords(query.value, 1)
    }

    /**
     * A delete-key swipe has reached [swipe] back from where it began ([starts]: it just
     * began). The query shrinks as it goes and grows back if the swipe returns.
     */
    fun deleteSwiped(swipe: GifSearchKeys.SwipeDelete, starts: Boolean) {
        if (starts) beforeSwipe = query.value
        mutableQuery.value = if (swipe.words) {
            SearchText.deleteWords(beforeSwipe, swipe.count)
        } else {
            SearchText.deleteLast(beforeSwipe, swipe.count)
        }
    }

    /** Searches for what was typed (trending if nothing was) and closes the field. */
    fun submit() {
        stopTyping()
        load(query.value.trim().ifEmpty { TRENDING })
    }

    /**
     * Shows [search]'s GIFs. Each GIPHY call counts against 100 an hour, so the chip already
     * showing (or loading) isn't fetched again, and a search made in the last 10 minutes comes
     * from [cache]. A failed one is always tried again.
     */
    fun load(search: String) {
        val giphy = giphy ?: return
        if (search == RECENTS) return showRecents()
        val alreadyShowing = search == shown.value && (results.value is Results.Loaded || loading?.isActive == true)
        if (alreadyShowing) return
        mutableShown.value = search
        loading?.cancel()
        cache.get(search)?.let { gifs ->
            mutableResults.value = Results.Loaded(gifs)
            return
        }
        mutableResults.value = Results.Loading
        loading = scope.launch {
            mutableResults.value = try {
                val gifs = if (search == TRENDING) giphy.trending() else giphy.search(search)
                cache.put(search, gifs)
                Results.Loaded(gifs)
            } catch (e: IOException) {
                Log.w(TAG, "Could not load GIFs", e)
                Results.Failed(problem(e))
            }
        }
    }

    private fun showRecents() {
        loading?.cancel()
        mutableShown.value = RECENTS
        mutableResults.value = Results.Loaded(recents.all())
    }

    /** The keyboard went away: next time it opens on the keyboard, not a half-typed search. */
    fun reset() {
        stopTyping()
        mutableShowingGifs.value = false
        GifImages.forgetPreviews()
    }

    private const val RECENTS_FILE = "gif-recents.json"

    internal fun problem(e: IOException) = when (e) {
        is GiphyFormatException -> "GIPHY sent something unreadable. Try again later."
        is GiphyException -> when (e.status) {
            401, 403 -> "GIPHY didn't accept the API key."
            429 -> "GIPHY's hourly limit is used up. Try again later."
            else -> "GIPHY isn't answering right now."
        }
        else -> "Couldn't reach GIPHY. Check your connection."
    }
}

/** Editing the search field, kept free of Android so it can be unit tested. */
internal object SearchText {
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
