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
import android.util.LruCache
import androidx.lifecycle.MutableLiveData
import com.jtn0123.audiowhisper.suggest.GlideGuess
import com.jtn0123.audiowhisper.suggest.WordCharacters
import com.jtn0123.audiowhisper.suggest.WordLearner
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.ime.clipboard.provider.ItemType
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.ime.editor.EditorRange
import dev.patrickgold.florisboard.ime.media.emoji.EmojiSuggestionProvider
import dev.patrickgold.florisboard.ime.nlp.han.HanShapeBasedLanguageProvider
import dev.patrickgold.florisboard.ime.nlp.latin.LatinLanguageProvider
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.util.NetworkUtils
import dev.patrickgold.florisboard.subtypeManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.florisboard.lib.kotlin.guardedByLock
import org.florisboard.lib.kotlin.collectLatestIn
import org.florisboard.lib.kotlin.safeSubstring
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger
import kotlin.properties.Delegates

private const val BLANK_STR_PATTERN = "^\\s*$"

/** AudioWhisper: how long after the first unsaved learned word the learned words are saved. */
private const val LEARNED_SAVE_DELAY_MS = 5_000L

class NlpManager(context: Context) {
    private val blankStrRegex = Regex(BLANK_STR_PATTERN)

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
    private val providers = guardedByLock {
        mapOf(
            LatinLanguageProvider.ProviderId to ProviderInstanceWrapper(latinProvider),
            HanShapeBasedLanguageProvider.ProviderId to ProviderInstanceWrapper(HanShapeBasedLanguageProvider(context)),
        )
    }
    // lock unnecessary because values constant
    private val providersForceSuggestionOn = mutableMapOf<String, Boolean>()
    private val preloadLock = Mutex()

    private val internalSuggestionsGuard = Mutex()
    private var internalSuggestions by Delegates.observable(SystemClock.uptimeMillis() to listOf<SuggestionCandidate>()) { _, _, _ ->
        scope.launch { assembleCandidates() }
    }

    private val _activeCandidatesFlow = MutableStateFlow(listOf<SuggestionCandidate>())
    val activeCandidatesFlow = _activeCandidatesFlow.asStateFlow()
    inline var activeCandidates
        get() = activeCandidatesFlow.value
        private set(v) {
            _activeCandidatesFlow.value = v
        }

    val debugOverlaySuggestionsInfos = LruCache<Long, Pair<String, SpellingResult>>(10)
    var debugOverlayVersion = MutableLiveData(0)
    private val debugOverlayVersionSource = AtomicInteger(0)

    init {
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

    fun providerForcesSuggestionOn(subtype: Subtype): Boolean {
        // Using a cache because I have no idea how fast the runBlocking is
        return providersForceSuggestionOn.getOrPut(subtype.nlpProviders.suggestion) {
            runBlocking {
                getSuggestionProvider(subtype).forcesSuggestionOn
            }
        }
    }

    fun isSuggestionOn(): Boolean =
        prefs.suggestion.enabled.get()
            || prefs.emoji.suggestionEnabled.get()
            || providerForcesSuggestionOn(subtypeManager.activeSubtype)

    fun suggest(subtype: Subtype, content: EditorContent) {
        val reqTime = SystemClock.uptimeMillis()
        scope.launch {
            val emojiSuggestions = when {
                prefs.emoji.suggestionEnabled.get() -> {
                    emojiSuggestionProvider.suggest(
                        subtype = subtype,
                        content = content,
                        maxCandidateCount = prefs.emoji.suggestionCandidateMaxCount.get(),
                        allowPossiblyOffensive = !prefs.suggestion.blockPossiblyOffensive.get(),
                        isPrivateSession = keyboardManager.activeState.isIncognitoMode,
                    )
                }
                else -> emptyList()
            }
            val suggestions = when {
                emojiSuggestions.isNotEmpty() && prefs.emoji.suggestionType.get().prefix.isNotEmpty() -> {
                    emptyList()
                }
                else -> {
                    getSuggestionProvider(subtype).suggest(
                        subtype = subtype,
                        content = content,
                        maxCandidateCount = 8,
                        allowPossiblyOffensive = !prefs.suggestion.blockPossiblyOffensive.get(),
                        isPrivateSession = keyboardManager.activeState.isIncognitoMode,
                    )
                }
            }
            internalSuggestionsGuard.withLock {
                if (internalSuggestions.first < reqTime) {
                    internalSuggestions = reqTime to buildList {
                        addAll(emojiSuggestions)
                        addAll(suggestions)
                    }
                }
            }
        }
    }

    fun suggestDirectly(suggestions: List<SuggestionCandidate>) {
        val reqTime = SystemClock.uptimeMillis()
        runBlocking {
            internalSuggestions = reqTime to suggestions
        }
    }

    fun clearSuggestions() {
        val reqTime = SystemClock.uptimeMillis()
        runBlocking {
            internalSuggestions = reqTime to emptyList()
        }
    }

    /**
     * The candidate to commit before [triggerCodePoint] (null for space) is inserted, if any.
     *
     * AudioWhisper: for the Latin provider the answer is computed for the word at the cursor right
     * now rather than read from [activeCandidates], which can be a keystroke behind; and digits,
     * apostrophes, hyphens and `@` continue a word instead of ending it (see [WordCharacters]).
     */
    fun getAutoCommitCandidate(triggerCodePoint: Int? = null): SuggestionCandidate? {
        val subtype = subtypeManager.activeSubtype
        if (subtype.nlpProviders.suggestion != LatinLanguageProvider.ProviderId) {
            return activeCandidates.firstOrNull { it.isEligibleForAutoCommit }
        }
        if (!prefs.suggestion.enabled.get()) return null
        if (triggerCodePoint != null && WordCharacters.continuesWord(triggerCodePoint)) return null
        // Clipboard or emoji candidates have replaced the word suggestions: nothing visible to commit.
        val shown = activeCandidates
        if (shown.isNotEmpty() && shown.none { it.sourceProvider === latinProvider }) return null
        return latinProvider.autoCommitCandidateFor(
            subtype = subtype,
            content = editorInstance.activeContent,
            allowPossiblyOffensive = !prefs.suggestion.blockPossiblyOffensive.get(),
        )
    }

    /**
     * AudioWhisper: the user undid the auto-correction of [word]; leave it alone from now on,
     * remembered across restarts unless the field or the user asked for incognito.
     */
    fun keepAsTyped(word: String) {
        val remember = !keyboardManager.activeState.isIncognitoMode
        if (latinProvider.keepAsTyped(word, remember)) {
            scope.launch(Dispatchers.IO) { latinProvider.saveKeptWords() }
        }
    }

    /** AudioWhisper: a save of the learned words is scheduled (see [saveLearnedWordsSoon]). */
    private val learnedSavePending = AtomicBoolean(false)

    /**
     * AudioWhisper: the word being composed in [content] was finished by [triggerCodePoint] (null
     * for a space or Enter) and stays as typed. See [WordLearner.finished].
     */
    fun learnTypedWord(content: EditorContent, triggerCodePoint: Int? = null) {
        if (triggerCodePoint != null && WordCharacters.continuesWord(triggerCodePoint)) return
        val word = content.composingText
        if (word.isEmpty() || !content.localComposing.isValid) return
        val textBefore = content.text.safeSubstring(0, content.localComposing.start)
        learn { it.finished(word, textBefore, content.composing.start) }
    }

    /** AudioWhisper: the candidate [text] is about to replace the word composed in [content], or follow it. */
    fun learnCandidate(text: String, content: EditorContent) {
        latinProvider.dropSwipedWord()
        val replacesWord = content.composingText.isNotEmpty() && content.localComposing.isValid
        if (replacesWord) {
            val textBefore = content.text.safeSubstring(0, content.localComposing.start)
            learn { it.finished(text, textBefore, content.composing.start) }
        } else {
            learn { it.finished(text, beforeNextWord(content.textBeforeSelection), content.selection.start) }
        }
    }

    /** AudioWhisper: the swiped [word] is about to go in at the cursor in [content]. */
    fun learnSwipedWord(word: String, content: EditorContent) {
        learn { it.swiped(word, beforeNextWord(content.textBeforeSelection)) }
    }

    /** AudioWhisper: a backspace; a swiped word waiting to be learned is being taken out. */
    fun dropSwipedWord() = latinProvider.dropSwipedWord()

    /** AudioWhisper: the auto-correction that started at [start] was undone; what it taught goes too. */
    fun unlearnAutoCorrection(start: Int) {
        learn { it.undo(start) }
    }

    /** AudioWhisper: input moved to another field. */
    fun resetLearning() = latinProvider.resetLearning()

    /** AudioWhisper: where the letters being typed were tapped (see [LatinLanguageProvider.tapTrail]). */
    val tapTrail get() = latinProvider.tapTrail

    /** AudioWhisper: erases everything learned from typing, on the phone too. */
    fun forgetLearnedWords() {
        scope.launch(Dispatchers.IO) { latinProvider.forgetLearnedWords() }
    }

    /** Applies [change] to the learner unless incognito or the field rules learning out. */
    private inline fun learn(change: (WordLearner) -> Boolean) {
        if (keyboardManager.activeState.isIncognitoMode) return
        val learner = latinProvider.learnerFor(subtypeManager.activeSubtype) ?: return
        if (change(learner)) saveLearnedWordsSoon()
    }

    /** One write for a burst of typing, a few seconds after its first word. */
    private fun saveLearnedWordsSoon() {
        if (!learnedSavePending.compareAndSet(false, true)) return
        scope.launch(Dispatchers.IO) {
            delay(LEARNED_SAVE_DELAY_MS)
            learnedSavePending.set(false)
            latinProvider.saveLearnedWords()
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
        if (textBefore.isNotEmpty() && !textBefore.last().isWhitespace()) "$textBefore " else textBefore

    /** AudioWhisper: see [LatinLanguageProvider.isName]. */
    fun isName(word: String) = latinProvider.isName(word)

    /** AudioWhisper: goes up when glide typing's word list changes (see [getListOfWords]). */
    val swipeWordsVersion: StateFlow<Int> get() = latinProvider.swipeWordsVersion

    /** AudioWhisper: picks up words added to the personal dictionaries since the keyboard last showed. */
    fun refreshPersonalWords() {
        scope.launch(Dispatchers.IO) { latinProvider.refreshPersonalWords() }
    }

    fun removeSuggestion(subtype: Subtype, candidate: SuggestionCandidate): Boolean {
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
        runBlocking {
            val candidates = when {
                isSuggestionOn() -> {
                    clipboardSuggestionProvider.suggest(
                        subtype = Subtype.DEFAULT,
                        content = editorInstance.activeContent,
                        maxCandidateCount = 8,
                        allowPossiblyOffensive = !prefs.suggestion.blockPossiblyOffensive.get(),
                        isPrivateSession = keyboardManager.activeState.isIncognitoMode,
                    ).ifEmpty {
                        buildList {
                            internalSuggestionsGuard.withLock {
                                addAll(internalSuggestions.second)
                            }
                        }
                    }
                }
                else -> emptyList()
            }
            activeCandidates = candidates
            autoExpandCollapseSmartbarActions(candidates, NlpInlineAutofill.suggestions.value)
        }
    }

    fun autoExpandCollapseSmartbarActions(list1: List<*>?, list2: List<*>?) {
        if (!prefs.smartbar.enabled.get()) {// || !prefs.smartbar.sharedActionsAutoExpandCollapse.get()) {
            return
        }
        // TODO: this is a mess and needs to be cleaned up in v0.5 with the NLP development
        /*if (keyboardManager.inputEventDispatcher.isRepeatableCodeLastDown()
            && !keyboardManager.inputEventDispatcher.isPressed(KeyCode.DELETE)
            && !keyboardManager.inputEventDispatcher.isPressed(KeyCode.FORWARD_DELETE)
            || keyboardManager.activeState.isActionsOverflowVisible
        ) {
            return // We do not auto switch if a repeatable action key was last pressed or if the actions overflow
                   // menu is visible to prevent annoying UI changes
        }*/
        val isSelection = editorInstance.activeContent.selection.isSelectionMode
        val isExpanded = list1.isNullOrEmpty() && list2.isNullOrEmpty() || isSelection
        scope.launch {
            prefs.smartbar.sharedActionsExpandWithAnimation.set(false)
            prefs.smartbar.sharedActionsExpanded.set(isExpanded)
        }
    }

    fun addToDebugOverlay(word: String, info: SpellingResult) {
        val version = debugOverlayVersionSource.incrementAndGet()
        debugOverlaySuggestionsInfos.put(System.currentTimeMillis(), word to info)
        debugOverlayVersion.postValue(version)
    }

    fun clearDebugOverlay() {
        val version = debugOverlayVersionSource.incrementAndGet()
        debugOverlaySuggestionsInfos.evictAll()
        debugOverlayVersion.postValue(version)
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

    inner class ClipboardSuggestionProvider internal constructor(private val context: Context) : SuggestionProvider {
        private var lastClipboardItemId: Long = -1

        override val providerId = "org.florisboard.nlp.providers.clipboard"

        override suspend fun create() {
            // Do nothing
        }

        override suspend fun preload(subtype: Subtype) {
            // Do nothing
        }

        override suspend fun suggest(
            subtype: Subtype,
            content: EditorContent,
            maxCandidateCount: Int,
            allowPossiblyOffensive: Boolean,
            isPrivateSession: Boolean,
        ): List<SuggestionCandidate> {
            // Check if enabled
            if (!prefs.clipboard.suggestionEnabled.get()) return emptyList()

            val currentItem = validateClipboardItem(clipboardManager.primaryClip, lastClipboardItemId, content.text)
                ?: return emptyList()

            return buildList {
                val now = System.currentTimeMillis()
                if ((now - currentItem.creationTimestampMs) < prefs.clipboard.suggestionTimeout.get() * 1000) {
                    add(ClipboardSuggestionCandidate(currentItem, sourceProvider = this@ClipboardSuggestionProvider, context = context))
                    if (currentItem.isSensitive) {
                        return@buildList
                    }
                    if (currentItem.type == ItemType.TEXT) {
                        val text = currentItem.stringRepresentation()
                        val matches = buildList {
                            addAll(NetworkUtils.getEmailAddresses(text))
                            addAll(NetworkUtils.getUrls(text))
                            addAll(NetworkUtils.getPhoneNumbers(text))
                        }
                        matches.forEachIndexed { i, match ->
                            val isUniqueMatch = matches.subList(0, i).all { prevMatch ->
                                prevMatch.value != match.value && prevMatch.range.intersect(match.range).isEmpty()
                            }
                            if (match.value != text && isUniqueMatch) {
                                add(ClipboardSuggestionCandidate(
                                    clipboardItem = currentItem.copy(
                                        // TODO: adjust regex of phone number so we don't need to manually strip the
                                        //  parentheses from the match results
                                        text = if (match.value.startsWith("(") && match.value.endsWith(")")) {
                                            match.value.substring(1, match.value.length - 1)
                                        } else {
                                            match.value
                                        }
                                    ),
                                    sourceProvider = this@ClipboardSuggestionProvider,
                                    context = context,
                                ))
                            }
                        }
                    }
                }
            }
        }

        override suspend fun notifySuggestionAccepted(subtype: Subtype, candidate: SuggestionCandidate) {
            if (candidate is ClipboardSuggestionCandidate) {
                lastClipboardItemId = candidate.clipboardItem.id
            }
        }

        override suspend fun notifySuggestionReverted(subtype: Subtype, candidate: SuggestionCandidate) {
            // Do nothing
        }

        override suspend fun removeSuggestion(subtype: Subtype, candidate: SuggestionCandidate): Boolean {
            if (candidate is ClipboardSuggestionCandidate) {
                lastClipboardItemId = candidate.clipboardItem.id
                return true
            }
            return false
        }

        override suspend fun getListOfWords(subtype: Subtype): List<String> {
            return emptyList()
        }

        override suspend fun getFrequencyForWord(subtype: Subtype, word: String): Double {
            return 0.0
        }

        override suspend fun destroy() {
            // Do nothing
        }

        private fun validateClipboardItem(currentItem: ClipboardItem?, lastItemId: Long, contentText: String) =
            currentItem?.takeIf {
                // Check if already used
                it.id != lastItemId
                    // Check if content is empty
                    && contentText.isBlank()
                    // Check if clipboard content has any valid characters
                    && !currentItem.text.isNullOrBlank()
                    && !blankStrRegex.matches(currentItem.text)
            }
    }
}
