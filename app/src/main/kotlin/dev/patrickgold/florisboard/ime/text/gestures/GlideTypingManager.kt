/*
 * Copyright (C) 2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.text.gestures

import android.content.Context
import android.os.SystemClock
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.nlp.WordSuggestionCandidate
import dev.patrickgold.florisboard.ime.keyboard.Key
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.subtypeManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Handles the [GlideTypingClassifier]. Basically responsible for linking [GlideTypingGesture.Detector]
 * with [GlideTypingClassifier].
 */
class GlideTypingManager(context: Context) : GlideTypingGesture.Listener {
    companion object {
        private const val MAX_SUGGESTION_COUNT = 8
    }

    private val prefs by FlorisPreferenceStore
    private val editorInstance by context.editorInstance()
    private val keyboardManager by context.keyboardManager()
    private val nlpManager by context.nlpManager()
    private val subtypeManager by context.subtypeManager()

    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
    private var glideTypingClassifier = StatisticalGlideTypingClassifier(context)


    init {
        val previewStarted = SystemClock.uptimeMillis()
        // AudioWhisper: the words change once the dictionary has loaded, and with the user's (B18).
        scope.launch(Dispatchers.Main) {
            nlpManager.startGlidePreviewClock(previewStarted)
            nlpManager.swipeWordsVersion.collect {
                glideTypingClassifier.setWordData(subtypeManager.activeSubtype)
            }
        }
    }

    override fun onGlideComplete(data: GlideTypingGesture.Detector.PointerData) {
        updateSuggestionsAsync(MAX_SUGGESTION_COUNT, true)
        // AudioWhisper: cleared now, not once the words arrive: it is scored from a copy, and a quick
        // next swipe added its first points to this gesture and then lost them to the late clear.
        glideTypingClassifier.clear()
    }

    override fun onGlideCancelled() {
        glideTypingClassifier.clear()
    }

    override fun onGlideAddPoint(point: GlideTypingGesture.Detector.Position) {
        val normalized = GlideTypingGesture.Detector.Position(point.x, point.y, point.time)

        this.glideTypingClassifier.addGesturePoint(normalized)

        if (nlpManager.refreshGlidePreview(SystemClock.uptimeMillis(), prefs.glide.showPreview.get(),
                prefs.glide.previewRefreshDelay.get())) {
            updateSuggestionsAsync(1, false)
        }
    }

    /**
     * Change the layout of the internal gesture classifier to [keys], built for [subtype]. Paired
     * with the active subtype instead, the last subtype's keys drawn once more after a switch read
     * as a new layout for the new one, and the classifier dropped its cached index and built it
     * again. The Compose keyboard passes no subtype and gets the active one, as before.
     */
    fun setLayout(keys: List<Key>, subtype: Subtype = subtypeManager.activeSubtype) {
        if (keys.isNotEmpty()) {
            glideTypingClassifier.setLayout(keys, subtype)
        }
    }

    /**
     * Asks gesture classifier for suggestions and then passes that on to the smartbar.
     * Also commits the most confident suggestion if [commit] is set. All happens on an async executor.
     * NB: only fetches [MAX_SUGGESTION_COUNT] suggestions.
     */
    private fun updateSuggestionsAsync(maxSuggestionsToShow: Int, commit: Boolean) {
        if (!glideTypingClassifier.ready) return

        // AudioWhisper: the gesture so far, and the words before the cursor that re-rank the guesses
        // (B14), both read here on the main thread.
        val swipe = glideTypingClassifier.snapshot()
        val textBefore = editorInstance.activeContent.textBeforeSelection
        val request = nlpManager.beginExternalRequest()
        scope.launch(Dispatchers.Default) {
            val guesses = glideTypingClassifier.getScoredSuggestions(swipe, MAX_SUGGESTION_COUNT)
            val suggestions = nlpManager.rankGlideGuesses(guesses, textBefore)

            withContext(Dispatchers.Main) {
                // A preview finishing after a later request would show a word for less of the swipe.
                if (!nlpManager.canApplyExternalRequest(request)) return@withContext
                val (shown, committed) = nlpManager.glideResultPlan(suggestions.size, maxSuggestionsToShow, commit)
                val suggestionList = shown.map { index ->
                    WordSuggestionCandidate(keyboardManager.fixCase(suggestions[index]), confidence = 1.0)
                }

                nlpManager.suggestDirectly(suggestionList)
                if (committed >= 0) {
                    keyboardManager.commitGesture(suggestions[committed])
                }
            }
        }
    }
}
