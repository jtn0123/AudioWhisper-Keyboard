/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.nlp

import android.content.Context
import android.os.SystemClock
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.suggest.GlideGuess
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.ime.editor.EditorRange
import dev.patrickgold.florisboard.ime.media.emoji.EmojiSuggestionProvider
import dev.patrickgold.florisboard.ime.nlp.latin.LatinLanguageProvider
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.subtypeManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.florisboard.lib.kotlin.guardedByLock
import org.florisboard.lib.kotlin.collectLatestIn
import java.util.concurrent.atomic.AtomicBoolean


class NlpManager(context: Context) {

    private val prefs by FlorisPreferenceStore
    private val clipboardManager by context.clipboardManager()
    private val editorInstance by context.editorInstance()
    private val keyboardManager by context.keyboardManager()
    private val subtypeManager by context.subtypeManager()

    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
    private val clipboardSuggestionProvider = ClipboardSuggestionProvider(context)
    private val emojiSuggestionProvider = EmojiSuggestionProvider(context)
    // AudioWhisper: kept directly so auto-correction and learning reach it without the providers map.
    private val latinProvider = LatinLanguageProvider(context)

    /** AudioWhisper: the private accuracy counters (see [TypingCounters]). */
    private val counters = TypingCounters(context.filesDir)
    private val providers = guardedByLock {
        mapOf(
            LatinLanguageProvider.ProviderId to ProviderInstanceWrapper(latinProvider),
        )
    }
    private val preloadLock = Mutex()

    private val environment = NlpEnvironment(context)

    /** AudioWhisper: typing events, each answered by the NLP session with one plan (see [TypingEvents]). */
    private val typing = TypingEvents(
        send = { event, context, candidate, lends -> latinProvider.typingEvent(subtypeManager.activeSubtype, event, context, candidate, lends) },
        context = { environment.fields(subtypeManager.activeSubtype, it) },
        now = SystemClock::uptimeMillis,
        saveAfter = ::saveLearnedWordsAfter,
        saveKept = { scope.launch(Dispatchers.IO) { latinProvider.saveKeptWords() } },
    )
    private var requestJob: Job? = null
    val activeCandidatesFlow = NlpState.candidates
    val activeCandidatesWordsFlow = NlpState.wordsShown
    val activeCandidates get() = activeCandidatesFlow.value

    private fun configure(subtype: Subtype = subtypeManager.activeSubtype, content: EditorContent = editorInstance.activeContent) {
        NlpState.configure(environment.fields(subtype, content))
    }

    init {
        NlpState.expanded.collectLatestIn(scope) { expanded ->
            if (expanded >= 0) {
                prefs.smartbar.sharedActionsExpandWithAnimation.set(false)
                prefs.smartbar.sharedActionsExpanded.set(expanded != 0)
            }
        }

        keyboardManager.activeState.collectLatestIn(scope) { configure() }
        editorInstance.activeInfoFlow.collectLatestIn(scope) { configure() }
        prefs.suggestion.incognitoMode.asFlow().collectLatestIn(scope) { configure() }
        prefs.suggestion.learnFromTyping.asFlow().collectLatestIn(scope) { configure() }
        scope.launch(Dispatchers.IO) { counters.load() }
        prefs.suggestion.blockPossiblyOffensive.asFlow().collectLatestIn(scope) { clearSuggestions() }
        prefs.suggestion.api30InlineSuggestionsEnabled.asFlow().collectLatestIn(scope) { configure() }
        prefs.smartbar.enabled.asFlow().collectLatestIn(scope) { configure() }
        clipboardManager.primaryClipFlow.collectLatestIn(scope) {
            assembleCandidates()
        }
        prefs.suggestion.enabled.asFlow().collectLatestIn(scope) {
            assembleCandidates()
        }
        prefs.clipboard.suggestionEnabled.asFlow().collectLatestIn(scope) {
            assembleCandidates()
        }
        prefs.emoji.suggestionEnabled.asFlow().collectLatestIn(scope) {
            assembleCandidates()
        }
        subtypeManager.activeSubtypeFlow.collectLatestIn(scope) { subtype ->
            clearSuggestions()
            preload(subtype)
        }
    }

    /**
     * Gets the punctuation rule from the currently active subtype and returns it. Falls back to a default one if the
     * subtype does not exist or defines an invalid punctuation rule.
     *
     * @return The punctuation rule or a fallback.
     */
    fun getActivePunctuationRule(): PunctuationRule {
        return getPunctuationRule(subtypeManager.activeSubtype)
    }

    /**
     * Gets the punctuation rule from the given subtype and returns it. Falls back to a default one if the subtype does
     * not exist or defines an invalid punctuation rule.
     *
     * @return The punctuation rule or a fallback.
     */
    fun getPunctuationRule(subtype: Subtype): PunctuationRule {
        return keyboardManager.resources.punctuationRules.value
            ?.get(subtype.punctuationRule) ?: PunctuationRule.Fallback
    }

    private suspend fun getSpellingProvider(subtype: Subtype): SpellingProvider {
        return providers.withLock { it[subtype.nlpProviders.spelling] }?.provider as? SpellingProvider
            ?: FallbackNlpProvider
    }

    private suspend fun getSuggestionProvider(subtype: Subtype): SuggestionProvider {
        return providers.withLock { it[subtype.nlpProviders.suggestion] }?.provider as? SuggestionProvider
            ?: FallbackNlpProvider
    }

    fun preload(subtype: Subtype) {
        scope.launch {
            emojiSuggestionProvider.preload(subtype)
            // AudioWhisper: the map's lock is held only to read the map. Loading the dictionary under
            // it made every provider lookup wait seconds for the load, including the ones the keyboard
            // makes on the main thread: touching a key while it started froze it (an ANR). Loads
            // still run one at a time, under their own lock.
            val wrappers = providers.withLock { providers ->
                buildList { subtype.nlpProviders.forEach { _, providerId -> providers[providerId]?.let(::add) } }
            }
            preloadLock.withLock {
                wrappers.forEach { provider ->
                    provider.createIfNecessary()
                    provider.preload(subtype)
                }
            }
        }
    }

    /**
     * Spell wrapper helper which calls the spelling provider and returns the result. Coroutine management must be done
     * by the source spell checker service.
     */
    suspend fun spell(
        subtype: Subtype,
        word: String,
        precedingWords: List<String>,
        followingWords: List<String>,
        maxSuggestionCount: Int,
    ): SpellingResult {
        return getSpellingProvider(subtype).spell(
            subtype = subtype,
            word = word,
            precedingWords = precedingWords,
            followingWords = followingWords,
            maxSuggestionCount = maxSuggestionCount,
            allowPossiblyOffensive = !prefs.suggestion.blockPossiblyOffensive.get(),
            isPrivateSession = keyboardManager.activeState.isIncognitoMode,
        )
    }

    suspend fun determineLocalComposing(
        textBeforeSelection: CharSequence, breakIterators: BreakIteratorGroup, localLastCommitPosition: Int
    ): EditorRange {
        return getSuggestionProvider(subtypeManager.activeSubtype).determineLocalComposing(
            subtypeManager.activeSubtype, textBeforeSelection, breakIterators, localLastCommitPosition
        )
    }

    fun providerForcesSuggestionOn(subtype: Subtype): Boolean =
        WireFlag.decode(NlpState.call(NlpOp.FORCE, subtype.nlpProviders.suggestion)[0])

    // The editor's legacy composing switch is independent of candidate field eligibility.
    fun isSuggestionOn(): Boolean = WireFlag.decode(environment.policy(subtypeManager.activeSubtype)[NlpPolicy.ENABLED])

    @Synchronized fun suggest(subtype: Subtype, content: EditorContent) {
        requestJob?.cancel()
        val plan = NlpState.begin(SystemClock.uptimeMillis(), environment.fields(subtype, content))
        if (plan[NlpBegin.DELAY].toLong() < 0) return
        requestJob = scope.launch {
            delay(plan[NlpBegin.DELAY].toLong())
            if (!WireFlag.decode(NlpState.call(NlpOp.READY, plan[NlpBegin.GENERATION], "${SystemClock.uptimeMillis()}")[0])) return@launch
            val values = try {
                val emoji = if (WireFlag.decode(plan[NlpBegin.EMOJI])) emojiSuggestionProvider.suggest(subtype, content,
                    prefs.emoji.suggestionCandidateMaxCount.get(), !prefs.suggestion.blockPossiblyOffensive.get(),
                    keyboardManager.activeState.isIncognitoMode) else emptyList()
                val words = if (WireFlag.decode(plan[NlpBegin.WORDS])) getSuggestionProvider(subtype).suggest(subtype, content, 8,
                    !prefs.suggestion.blockPossiblyOffensive.get(), keyboardManager.activeState.isIncognitoMode) else emptyList()
                emoji + words
            } catch (cancelled: kotlinx.coroutines.CancellationException) {
                throw cancelled
            } catch (_: Exception) {
                emptyList()
            }
            NlpState.complete(plan[NlpBegin.GENERATION], values, environment.fields(subtypeManager.activeSubtype))
            assembleCandidates()
        }
    }

    fun beginExternalRequest(): String {
        configure()
        return NlpState.call(NlpOp.EXTERNAL_BEGIN)[0]
    }

    fun startGlidePreviewClock(now: Long) { NlpState.call(NlpOp.PREVIEW_START, now.toString()) }

    fun refreshGlidePreview(now: Long, enabled: Boolean, delay: Int): Boolean =
        WireFlag.decode(NlpState.call(NlpOp.PREVIEW, now.toString(), WireFlag.encode(enabled), delay.toString())[0])

    /** Of [count] glide suggestions, the ones to show and the one to commit (-1 for none), as an [NlpGlidePlan] row says. */
    fun glideResultPlan(count: Int, max: Int, commit: Boolean): Pair<IntRange, Int> = with(NlpGlidePlan) {
        NlpState.call(NlpOp.GLIDE_PLAN, "$count", "$max", WireFlag.encode(commit)).map(String::toInt).let { it[FIRST] until it[END] to it[COMMIT] } }

    fun canApplyExternalRequest(generation: String): Boolean {
        configure()
        return WireFlag.decode(NlpState.call(NlpOp.EXTERNAL_VALID, generation)[0])
    }

    @Synchronized fun suggestDirectly(suggestions: List<SuggestionCandidate>) {
        configure()
        requestJob?.cancel()
        NlpState.directly(suggestions)
    }

    @Synchronized fun clearSuggestions() {
        requestJob?.cancel()
        configure()
        NlpState.clear()
    }

    /**
     * AudioWhisper: whether [candidate], tapped as the editor stands, may go in. With [commit] it
     * goes in and is learned, and with [corrects] it is the auto-correction a backspace straight
     * after undoes (see [TypingEvents.picked]).
     */
    fun pick(candidate: SuggestionCandidate, commit: Boolean, corrects: Boolean = false): Boolean =
        typing.picked(candidate, editorInstance.activeContent, commit, corrects)

    /**
     * The candidate to commit before [triggerCodePoint] (null for space) is inserted, if any.
     *
     * AudioWhisper: for the Latin provider the answer is computed for the word at the cursor right
     * now rather than read from [activeCandidates], which can be a keystroke behind; and digits,
     * apostrophes, hyphens and `@` continue a word instead of ending it (see [com.jtn0123.audiowhisper.rust.SuggestRules.continuesWord]).
     */
    fun getAutoCommitCandidate(triggerCodePoint: Int? = null): SuggestionCandidate? {
        val requested = environment.fields(subtypeManager.activeSubtype)
        NlpState.configure(requested)
        if (!WireFlag.decode(NlpState.call(NlpOp.AUTO, "${triggerCodePoint ?: -1}", WireFlag.encode(editorInstance.phantomSpace.isActive))[0])) return null
        val candidate = latinProvider.autoCommitCandidateFor(subtypeManager.activeSubtype, editorInstance.activeContent,
            !prefs.suggestion.blockPossiblyOffensive.get()) ?: return null
        return candidate.takeIf { NlpState.autoCandidate(it, requested, environment.fields(subtypeManager.activeSubtype)) }
    }

    /**
     * AudioWhisper: the word being composed in [content] was finished by [triggerCodePoint] (null
     * for a space or Enter) and stays as typed.
     */
    fun learnTypedWord(content: EditorContent, triggerCodePoint: Int? = null) =
        typing.finished(content, triggerCodePoint, aim = prefs.suggestion.learnTapAim.get())

    fun learnSwipedWord(word: String, content: EditorContent) = typing.swiped(word, content)

    /**
     * AudioWhisper: a backspace; a swiped word waiting to be learned is being taken out, and so
     * is a typed word waiting to teach where this user taps.
     */
    fun dropSwipedWord() = editorInstance.activeContent.selection.let { typing.backspace(it.start, it.end) }

    /** AudioWhisper: a key that types went down (a space or Enter), for the counters. */
    fun noteKey() = typing.key()

    /** AudioWhisper: a tapped character, [text], went in. */
    fun noteTyped(text: String) = typing.typed(text)

    /** AudioWhisper: the tap trail's last character was deleted. */
    fun noteTapDeleted() = typing.tapDeleted()

    /** AudioWhisper: what was just done can no longer undo an auto-correction. */
    fun disarmCorrectionUndo() = typing.disarm()

    /** AudioWhisper: how to put back what was typed, if the last key auto-corrected it. */
    internal fun undoAutoCorrection(): TypingEvents.Revert? = typing.undoCorrection(editorInstance.activeContent)

    /** AudioWhisper: the backspace straight after an auto-correction undid it. */
    fun noteCorrectionUndone() = typing.correctionUndone()

    /** AudioWhisper: the undo key; nothing just typed may teach where this user taps. */
    fun noteUndo() = typing.undo()

    /** AudioWhisper: the screen's orientation, which where this user taps is learned per. */
    fun noteOrientation(landscape: Boolean) = tapTrail.landscape(landscape)

    /** AudioWhisper: what was typed and where, for the Rust renderer's hit test. */
    val tapPositions get() = latinProvider.tapTrail.positions

    /** AudioWhisper: whether the hit tests use where this user is known to tap (the setting). */
    val aimsTaps get() = prefs.suggestion.learnTapAim.get()

    /**
     * AudioWhisper: [hit], or its neighbour if the letters likely to come next or, with the
     * setting on, this user's learned aim say a tap at ([x], [y]) meant it (see
     * [com.jtn0123.audiowhisper.suggest.TapTrail.retarget]).
     */
    fun aimedKey(hit: Int, layout: IntArray, bounds: FloatArray, x: Float, y: Float): Int =
        tapTrail.retarget(hit, layout, bounds, x, y, aimsTaps)

    /** AudioWhisper: input moved to another field. */
    fun resetLearning() = typing.field()

    /** AudioWhisper: where the letters being typed were tapped (see [LatinLanguageProvider.tapTrail]). */
    val tapTrail get() = latinProvider.tapTrail

    /** AudioWhisper: erases everything learned from typing, on the phone too. */
    fun forgetLearnedWords() {
        scope.launch(Dispatchers.IO) {
            latinProvider.forgetLearnedWords()
            counters.reset()
        }
    }

    /** Saves the learned words, the tap aim and the counters after [delayMs], as a typing event's plan asked (-1: none). */
    private fun saveLearnedWordsAfter(delayMs: Long) {
        if (delayMs < 0) return
        scope.launch(Dispatchers.IO) {
            delay(delayMs)
            NlpState.call(NlpOp.SAVE, SystemClock.uptimeMillis().toString())
            latinProvider.saveLearnedWords()
            latinProvider.saveTapAim()
            counters.save()
        }
    }

    /**
     * AudioWhisper: the swipe [guesses] in order for the cursor after [textBefore], which a swiped
     * word follows with a space (see [LatinLanguageProvider.rankGlideGuesses]). Any thread.
     */
    fun rankGlideGuesses(guesses: List<GlideGuess>, textBefore: String): List<String> =
        latinProvider.rankGlideGuesses(subtypeManager.activeSubtype, guesses, beforeNextWord(textBefore))

    /** [textBefore] as the next word will see it: a word right before the cursor gets its space first. */
    private fun beforeNextWord(textBefore: String): String =
        NlpState.call(NlpOp.NEXT, textBefore)[0]

    /** AudioWhisper: see [LatinLanguageProvider.isName]. */
    fun isName(word: String) = latinProvider.isName(word)

    /** AudioWhisper: goes up when glide typing's word list changes (see [getListOfWords]). */
    val swipeWordsVersion: StateFlow<Int> get() = latinProvider.swipeWordsVersion

    /** AudioWhisper: picks up words added to the personal dictionaries since the keyboard last showed. */
    fun refreshPersonalWords() {
        scope.launch(Dispatchers.IO) { latinProvider.refreshPersonalWords() }
    }

    fun removeSuggestion(subtype: Subtype, candidate: SuggestionCandidate): Boolean {
        if (!pick(candidate, commit = false)) return false
        return runBlocking { candidate.sourceProvider?.removeSuggestion(subtype, candidate) == true }.also { result ->
            if (result) {
                scope.launch {
                    // Need to re-trigger the suggestions algorithm
                    if (candidate is ClipboardSuggestionCandidate) {
                        assembleCandidates()
                    } else {
                        suggest(subtypeManager.activeSubtype, editorInstance.activeContent)
                    }
                }
            }
        }
    }

    fun getListOfWords(subtype: Subtype): List<String> {
        return runBlocking { getSuggestionProvider(subtype).getListOfWords(subtype) }
    }

    fun getFrequencyForWord(subtype: Subtype, word: String): Double {
        return runBlocking { getSuggestionProvider(subtype).getFrequencyForWord(subtype, word) }
    }

    private fun assembleCandidates() {
        val requested = environment.fields(subtypeManager.activeSubtype)
        val generation = NlpState.beginClipboard(requested)
        runBlocking {
            val values = clipboardSuggestionProvider.suggest(Subtype.DEFAULT, editorInstance.activeContent, 8,
                !prefs.suggestion.blockPossiblyOffensive.get(), keyboardManager.activeState.isIncognitoMode)
            NlpState.clipboard(generation, values, requested, environment.fields(subtypeManager.activeSubtype))
        }
    }

    private class ProviderInstanceWrapper(val provider: NlpProvider) {
        private var isInstanceAlive = AtomicBoolean(false)

        suspend fun createIfNecessary() {
            if (!isInstanceAlive.getAndSet(true)) provider.create()
        }

        suspend fun preload(subtype: Subtype) {
            provider.preload(subtype)
        }

        suspend fun destroyIfNecessary() {
            if (isInstanceAlive.getAndSet(true)) provider.destroy()
        }
    }

}
