package com.jtn0123.audiowhisper.gif

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

    private val giphy = BuildConfig.GIPHY_API_KEY.takeIf { it.isNotBlank() }?.let { Giphy(it) }

    /**
     * A GIPHY key was built in. Without one there is no GIF tab: nothing on the phone could
     * supply the key, so the tab could only ever say it was missing. See `giphy.apiKey` in
     * app/build.gradle.kts.
     */
    val available: Boolean get() = giphy != null
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var loading: Job? = null

    /** The query when the current delete-key swipe began, so swiping back restores it. */
    private var beforeSwipe = ""

    private val mutableShowingGifs = MutableStateFlow(false)
    private val mutableTyping = MutableStateFlow(false)
    private val mutableQuery = MutableStateFlow("")
    private val mutableShown = MutableStateFlow(TRENDING)
    private val mutableResults = MutableStateFlow<Results>(Results.Loading)

    /** The media panel shows GIFs rather than emoji. */
    val showingGifs: StateFlow<Boolean> = mutableShowingGifs.asStateFlow()

    /** The search field is open over the keyboard and typing goes into it. */
    val typing: StateFlow<Boolean> = mutableTyping.asStateFlow()

    /** What is in the search field. */
    val query: StateFlow<String> = mutableQuery.asStateFlow()

    /** The search (or [TRENDING]) whose GIFs are shown. */
    val shown: StateFlow<String> = mutableShown.asStateFlow()

    val results: StateFlow<Results> = mutableResults.asStateFlow()

    fun showGifs(show: Boolean) {
        mutableShowingGifs.value = show && available
        if (show && results.value !is Results.Loaded) load(shown.value)
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

    fun load(search: String) {
        mutableShown.value = search
        val giphy = giphy ?: return
        loading?.cancel()
        mutableResults.value = Results.Loading
        loading = scope.launch {
            mutableResults.value = try {
                Results.Loaded(if (search == TRENDING) giphy.trending() else giphy.search(search))
            } catch (e: IOException) {
                Log.w(TAG, "Could not load GIFs", e)
                Results.Failed(problem(e))
            }
        }
    }

    /** The keyboard went away: next time it opens on the keyboard, not a half-typed search. */
    fun reset() {
        stopTyping()
        mutableShowingGifs.value = false
        GifImages.forgetPreviews()
    }

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
}
