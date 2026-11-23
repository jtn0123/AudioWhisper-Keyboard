package com.jtn0123.audiowhisper.gif

import android.os.SystemClock
import android.util.Log
import com.jtn0123.audiowhisper.rust.GifRules
import dev.patrickgold.florisboard.BuildConfig
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob

/**
 * The GIF panel's session ([GifSearchSession]). One keyboard per process, so the panel, the
 * search bar and the key handler share this. Main thread only.
 */
object GifSearch : GifSearchSession(
    scope = CoroutineScope(SupervisorJob() + Dispatchers.Main),
    io = Dispatchers.IO,
    clock = SystemClock::elapsedRealtime,
    log = { Log.w("AWGif", it) },
    forgetPreviews = { GifImages.forgetPreviews() },
    source = BuildConfig.GIPHY_API_KEY.takeIf { it.isNotBlank() }?.let { Giphy(it) },
) {
    sealed interface Results {
        data object Loading : Results
        data class Loaded(val gifs: List<Gif>) : Results
        data class Failed(val message: String) : Results
    }

    /** The quick searches under the search field; [TRENDING] is GIPHY's trending list. */
    val categories: List<String> = GifRules.categories
    val TRENDING: String = GifRules.trending

    /** The GIFs sent before ([GifRecents]): a chip that needs no GIPHY call. */
    val RECENTS: String = GifRules.recents
}
