/*
 * Copyright (C) 2022-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.nlp.latin

import android.content.Context
import android.database.sqlite.SQLiteException
import android.os.SystemClock
import android.util.Log
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.suggest.ArrangedWord
import com.jtn0123.audiowhisper.suggest.GlideGuess
import com.jtn0123.audiowhisper.suggest.GlideRanker
import com.jtn0123.audiowhisper.suggest.KeptWords
import com.jtn0123.audiowhisper.suggest.LatinDictionaryEngine
import com.jtn0123.audiowhisper.suggest.LearnedWords
import com.jtn0123.audiowhisper.suggest.LatinSuggester
import com.jtn0123.audiowhisper.suggest.NOT_A_PROBABILITY
import com.jtn0123.audiowhisper.suggest.PersonalWords
import com.jtn0123.audiowhisper.suggest.PreviousWords
import com.jtn0123.audiowhisper.suggest.PrivateTextFile
import com.jtn0123.audiowhisper.suggest.SwipeWords
import com.jtn0123.audiowhisper.suggest.TapTrail
import com.jtn0123.audiowhisper.suggest.TypingHistory
import com.jtn0123.audiowhisper.suggest.TypingState
import dev.patrickgold.florisboard.BuildConfig
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.dictionary.DictionaryManager
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryDao
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.ime.editor.InputAttributes
import dev.patrickgold.florisboard.ime.nlp.NlpEnvironment
import dev.patrickgold.florisboard.ime.nlp.NlpState
import dev.patrickgold.florisboard.ime.nlp.SpellingProvider
import dev.patrickgold.florisboard.ime.nlp.SpellingResult
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.SuggestionProvider
import dev.patrickgold.florisboard.ime.nlp.TypingStores
import dev.patrickgold.florisboard.ime.nlp.WordSuggestionCandidate
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.builtins.MapSerializer
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.Json
import org.florisboard.lib.android.readText
import org.florisboard.lib.kotlin.guardedByLock
import org.florisboard.lib.kotlin.safeSubstring
import java.io.File
import java.io.IOException
import java.util.Locale

/**
 * Word suggestions, auto-correction and next-word prediction from the AOSP LatinIME dictionary
 * engine (English only), and swipe typing's words: the dictionary's in English, upstream's test
 * list (`ime/dict/data.json`) in other languages.
 */
class LatinLanguageProvider(context: Context) : SpellingProvider, SuggestionProvider {
    companion object {
        // Default user ID used for all subtypes, unless otherwise specified.
        // See `ime/core/Subtype.kt` Line 210 and 211 for the default usage
        const val ProviderId = "org.florisboard.nlp.providers.latin"

        private const val TAG = "AWSuggest"

        /** The AOSP en_US main dictionary, stored uncompressed so native code can mmap it from the APK. */
        private const val DICTIONARY_ASSET = "dicts/main_en-US.dict"
        private const val MODEL_ASSET = "dicts/main_en-US.lm"
        private const val DICTIONARY_LOCALE = "en_US"
        private const val DICTIONARY_LANGUAGE = "en"

        /** In filesDir, outside the backup rules' include list, so it never leaves the phone. */
        private const val KEPT_WORDS_FILE = "kept-as-typed.txt"

        /** What the keyboard learned from typing; like [KEPT_WORDS_FILE], never backed up. */
        private const val LEARNED_WORDS_FILE = "learned-words.txt"

        /** Where this user's taps land in each key (see [TapTrail.aimText]); numbers only, never backed up. */
        private const val TAP_AIM_FILE = "tap-aim.txt"


    }

    private val environment = NlpEnvironment(context)
    private val prefs by FlorisPreferenceStore
    private val appContext by context.appContext()
    private val editorInstance by context.editorInstance()

    /** Upstream's glide word list, for languages the dictionary doesn't cover; [testWords] is its words. */
    private val wordData = guardedByLock { mutableMapOf<String, Int>() }
    private val wordDataSerializer = MapSerializer(String.serializer(), Int.serializer())
    @Volatile private var testWords: List<String> = emptyList()

    /** English swipe typing's words (B18): the dictionary's once [preload] has read them, plus the user's. */
    @Volatile private var swipeWords = SwipeWords.NONE
    @Volatile private var swipeWordsFromDictionary = false
    private val swipeWordsLock = Any()
    private val mutableSwipeWordsVersion = MutableStateFlow(0)

    /** Goes up each time [getListOfWords] would return a new list, so glide typing re-indexes. */
    val swipeWordsVersion: StateFlow<Int> = mutableSwipeWordsVersion.asStateFlow()

    private val dictionaryLoadLock = Mutex()
    @Volatile private var engine: LatinDictionaryEngine? = null
    @Volatile private var suggester: LatinSuggester? = null

    /** The personal dictionaries, re-read by [refreshPersonalWords] each time the keyboard shows. */
    @Volatile private var personalWords = PersonalWords.NONE

    /** Words whose auto-correction the user undid; they are not corrected again. Loaded by [preload]. */
    private val keptAsTyped by lazy { KeptWords(File(appContext.filesDir, KEPT_WORDS_FILE)) }
    @Volatile private var keptWordsLoaded = false

    /**
     * What the user has typed, which the NLP session's typing events teach (see
     * lib/rust/crates/suggest/src/word_learner.rs). Loaded by [preload], saved by [saveLearnedWords].
     */
    private val learnedWords by lazy { LearnedWords(File(appContext.filesDir, LEARNED_WORDS_FILE)) }
    private val typingStores by lazy { TypingStores(Learner(learnedWords.store), learnedWords.store, tapTrail.positions, keptAsTyped.store) }
    @Volatile private var learnedWordsLoaded = false

    /** AudioWhisper: where the letters being typed were tapped, for the decoder (B13). */
    val tapTrail = TapTrail()

    /** Where [tapTrail] keeps what it learned of this user's aim. Loaded by [preload], saved by [saveTapAim]. */
    private val tapAimFile by lazy { PrivateTextFile(File(appContext.filesDir, TAP_AIM_FILE)) }
    @Volatile private var tapAimLoaded = false

    override val providerId = ProviderId

    override suspend fun create() {
        // Here we initialize our provider, set up all things which are not language dependent.
    }

    override suspend fun preload(subtype: Subtype) = withContext(Dispatchers.IO) {
        val english = usesDictionary(subtype)
        if (english) loadDictionary() else loadTestWords()
        if (!keptWordsLoaded) {
            keptAsTyped.load()
            keptWordsLoaded = true
        }
        if (!learnedWordsLoaded) {
            learnedWords.load()
            learnedWordsLoaded = true
        }
        if (!tapAimLoaded) {
            tapTrail.loadAim(tapAimFile.read())
            tapAimLoaded = true
        }
        if (english) loadSwipeWords()
    }

    private fun usesDictionary(subtype: Subtype) = WireFlag.decode(NlpState.call(NlpOp.LANGUAGE, subtype.primaryLocale.language)[0])

    /** Upstream's glide word list, for a language the dictionary doesn't cover. */
    private suspend fun loadTestWords() {
        val loaded = wordData.withLock { wordData ->
            if (wordData.isNotEmpty()) return@withLock false
            // Here we use readText() because the test dictionary is a json dictionary
            val rawData = appContext.assets.readText("ime/dict/data.json")
            wordData.putAll(Json.decodeFromString(wordDataSerializer, rawData))
            testWords = wordData.keys.toList()
            true
        }
        if (loaded) mutableSwipeWordsVersion.update { it + 1 }
    }

    /** Reads swipe typing's words from the dictionary, once: about 160,000 against upstream's 50,000. */
    private suspend fun loadSwipeWords() {
        dictionaryLoadLock.withLock {
            val engine = engine
            if (swipeWordsFromDictionary || engine == null) return@withLock
            val started = SystemClock.elapsedRealtime()
            val fromDictionary = SwipeWords.fromDictionary(engine)
            synchronized(swipeWordsLock) {
                swipeWords = withUserSwipeWords(fromDictionary)
                swipeWordsFromDictionary = true
            }
            mutableSwipeWordsVersion.update { it + 1 }
            Log.d(TAG, "${fromDictionary.words.size} swipe words in ${SystemClock.elapsedRealtime() - started} ms")
        }
    }

    /**
     * Makes the user's current personal and learned words swipeable, and forgotten ones not, each
     * learned word raised as much as its uses now raise it.
     */
    private fun refreshSwipeWords() {
        val changed = synchronized(swipeWordsLock) {
            val updated = withUserSwipeWords(swipeWords)
            (updated !== swipeWords).also { swipeWords = updated }
        }
        if (changed) mutableSwipeWordsVersion.update { it + 1 }
    }

    private fun withUserSwipeWords(words: SwipeWords): SwipeWords {
        val learned = if (prefs.suggestion.learnFromTyping.get()) learnedWords.knownSpellings() else emptyList()
        val boosts = SwipeWords.boosts(learned, learnedWords.swipeBoosts(learned))
        return words.withUserWords(SwipeWords.userWords(personalWords.entries, learned), boosts)
    }

    /** Opens the bundled dictionary on first use; later calls return at once. */
    private suspend fun loadDictionary() {
        dictionaryLoadLock.withLock {
            if (engine == null) openDictionary()
        }
    }

    private fun openDictionary() {
        val started = SystemClock.elapsedRealtime()
        val opened = openBundledDictionary() ?: return
        engine = opened
        suggester = LatinSuggester(opened, strength = { prefs.correction.autoCorrect.get() },
            personalWords = { personalWords }, history = ::learnedHistory)
        Log.d(TAG, "Dictionary loaded in ${SystemClock.elapsedRealtime() - started} ms")
    }

    private fun openBundledDictionary(): LatinDictionaryEngine? = try {
        appContext.assets.openFd(DICTIONARY_ASSET).use { fd ->
            LatinDictionaryEngine.open(appContext.applicationInfo.sourceDir, fd.startOffset, fd.length, DICTIONARY_LOCALE)
        }.also { if (it == null) Log.w(TAG, "The bundled dictionary is not a valid AOSP dictionary") }
            ?.also(::attachBundledModel)
    } catch (e: IOException) {
        Log.w(TAG, "Could not open the bundled dictionary", e)
        null
    } catch (e: LinkageError) {
        Log.w(TAG, "Could not load the native dictionary library", e)
        null
    }

    /** The n-gram model; without it the dictionary alone predicts and weighs context. */
    private fun attachBundledModel(engine: LatinDictionaryEngine) {
        try {
            appContext.assets.openFd(MODEL_ASSET).use { fd ->
                if (!engine.attachModel(appContext.applicationInfo.sourceDir, fd.startOffset, fd.length)) {
                    Log.w(TAG, "The bundled language model is not a valid model")
                }
            }
        } catch (e: IOException) {
            Log.w(TAG, "Could not open the bundled language model", e)
        }
    }

    private fun suggesterFor(subtype: Subtype): LatinSuggester? =
        suggester.takeIf { usesDictionary(subtype) }

    override suspend fun spell(
        subtype: Subtype,
        word: String,
        precedingWords: List<String>,
        followingWords: List<String>,
        maxSuggestionCount: Int,
        allowPossiblyOffensive: Boolean,
        isPrivateSession: Boolean,
    ): SpellingResult {
        if (!WireFlag.decode(NlpState.call(NlpOp.SPELL, subtype.primaryLocale.language, word)[0])) return SpellingResult.validWord()
        withContext(Dispatchers.IO) { loadDictionary() }
        val suggester = suggester ?: return SpellingResult.validWord()
        val check = suggester.spell(word, maxSuggestionCount, blockOffensiveWords = !allowPossiblyOffensive)
        return when {
            check.isValid -> SpellingResult.validWord()
            else -> SpellingResult.typo(check.suggestions.toTypedArray(), check.isHighConfidence)
        }
    }

    override suspend fun suggest(
        subtype: Subtype,
        content: EditorContent,
        maxCandidateCount: Int,
        allowPossiblyOffensive: Boolean,
        isPrivateSession: Boolean,
    ): List<SuggestionCandidate> {
        val suggester = suggesterFor(subtype) ?: return emptyList()
        val state = typingStateOf(subtype, content) ?: return emptyList()
        val started = System.nanoTime()
        val raw = suggester.suggest(state, maxCandidateCount, !allowPossiblyOffensive)
        val flags = NlpState.call(NlpOp.CANDIDATE_PLAN, *environment.fields(subtype, content), WireFlag.encode(isKeptAsTyped(state.typed)),
            *raw.map { WireFlag.encode(it.isAutoCorrection) }.toTypedArray())
        val candidates = raw.mapIndexed { index, word -> word.toCandidate(WireFlag.decode(flags[index])) }
        val autoCorrection = candidates.firstOrNull { it.isEligibleForAutoCommit }
        // Runs on every keystroke, so the line isn't even built outside debug builds.
        if (BuildConfig.DEBUG) {
            val elapsedMs = (System.nanoTime() - started) / 1_000_000.0
            Log.d(TAG, "suggest: %d candidates, autocorrect=%b, %.2f ms".format(
                Locale.ROOT, candidates.size, autoCorrection != null, elapsedMs,
            ))
        }
        return candidates
    }

    /**
     * AudioWhisper: the auto-correction for the word at the cursor in [content] *now*. It runs
     * on the main thread at every space or punctuation key, so it reuses the decode the candidate
     * strip just did when that was for exactly this word, context, taps and setting, and decodes
     * here only when the strip is a keystroke behind ([LatinSuggester.autoCorrection]).
     */
    fun autoCommitCandidateFor(subtype: Subtype, content: EditorContent, allowPossiblyOffensive: Boolean): SuggestionCandidate? {
        val suggester = suggesterFor(subtype) ?: return null
        if (environment.policy(subtype, content)[3] != "1" || editorInstance.phantomSpace.isActive) return null
        val state = typingStateOf(subtype, content)?.takeIf { it.typed.isNotEmpty() && !isKeptAsTyped(it.typed) } ?: return null
        return suggester.autoCorrection(state, blockOffensiveWords = !allowPossiblyOffensive)
            ?.toCandidate(autoCorrectAllowed = true)
    }

    /** AudioWhisper: writes [keptAsTyped] to filesDir. Blocking; call off the main thread. */
    fun saveKeptWords() {
        if (!keptAsTyped.save()) Log.w(TAG, "Could not save the words kept as typed")
    }

    /**
     * AudioWhisper: [word], capitalised, is usually written that way ("John", but not "Maybe"),
     * so dictation keeps its capital mid-sentence. A word the dictionary doesn't know is
     * assumed to be a name; so is everything when no dictionary is loaded.
     */
    fun isName(word: String): Boolean = engine?.session?.isName(personalWords.store, word)
        ?: com.jtn0123.audiowhisper.rust.NlpNames.withoutDictionary(personalWords.store, word)

    /**
     * AudioWhisper: re-reads the Floris and system personal dictionaries (those the settings
     * enable), so a word added since counts from the next keystroke. Blocking; call off the main thread.
     */
    fun refreshPersonalWords() {
        val manager = DictionaryManager.default()
        manager.loadUserDictionariesIfNecessary()
        val entries = listOfNotNull(manager.florisUserDictionaryDao(), manager.systemUserDictionaryDao())
            .flatMap(::readPersonalDictionary)
            .filter { PersonalWords.appliesTo(it.locale, DICTIONARY_LANGUAGE) }
            .map { PersonalWords.Entry(it.word, it.freq, it.shortcut) }
        personalWords = PersonalWords(entries)
        refreshSwipeWords()
    }

    private fun readPersonalDictionary(dao: UserDictionaryDao) = try {
        dao.queryAll()
    } catch (e: SecurityException) {
        Log.w(TAG, "Not allowed to read a personal dictionary", e)
        emptyList()
    } catch (e: SQLiteException) {
        Log.w(TAG, "Could not read a personal dictionary", e)
        emptyList()
    }

    private fun isKeptAsTyped(typed: String) = typed in keptAsTyped

    /** AudioWhisper: what suggestions may draw on from the user's typing, or null if learning is off. */
    private fun learnedHistory(): LearnedWords? = learnedWords.takeIf { prefs.suggestion.learnFromTyping.get() }

    /** AudioWhisper: [learnedHistory] for the swipe ranker, which takes [TypingHistory.NONE] for none. */
    private fun typingHistory(): TypingHistory = learnedHistory() ?: TypingHistory.NONE

    /**
     * AudioWhisper: the plan for typing event [event] with [context] after it (see [NlpState.event]),
     * over the learned and kept words and the tap trail. With [lends], words may be learned: the
     * dictionary is lent while [subtype] suggests from it, as the session asks.
     */
    internal fun typingEvent(subtype: Subtype, event: Array<String>, context: Array<String>, candidate: SuggestionCandidate?,
                             lends: Boolean): Array<String> {
        val dictionary = if (lends && suggesterFor(subtype) != null) engine?.session else null
        return NlpState.event(typingStores, dictionary, event, context, candidate)
    }

    /** AudioWhisper: writes [learnedWords] to filesDir. Blocking; call off the main thread. */
    fun saveLearnedWords() {
        if (!learnedWords.save()) Log.w(TAG, "Could not save the learned words")
    }

    /** AudioWhisper: writes where this user's taps land to filesDir. Blocking; call off the main thread. */
    fun saveTapAim() {
        if (!tapAimFile.write(tapTrail.aimText())) Log.w(TAG, "Could not save the tap aim")
    }

    /** AudioWhisper: forgets everything learned from typing, on disk too. Blocking. */
    fun forgetLearnedWords() {
        learnedWords.clear()
        saveLearnedWords()
        keptAsTyped.clear()
        saveKeptWords()
        tapTrail.forgetAim()
        tapAimFile.delete()
        refreshSwipeWords()
    }

    /**
     * AudioWhisper: the swipe [guesses] for a word right after [textBefore], best first once the
     * words before it are counted (see [GlideRanker]). Other languages keep the classifier's order.
     */
    fun rankGlideGuesses(subtype: Subtype, guesses: List<GlideGuess>, textBefore: String): List<String> {
        if (suggesterFor(subtype) == null) return guesses.map { it.word }
        return GlideRanker(typingHistory(), engine).rank(guesses, PreviousWords.from(textBefore))
    }

    /**
     * The word left of the cursor and its context, or null when there is nothing to suggest for:
     * a selection, a field that must not get suggestions, or an emoji search like ":smile".
     */
    private fun typingStateOf(subtype: Subtype, content: EditorContent): TypingState? {
        val before = if (content.composingText.isEmpty()) content.textBeforeSelection else content.text.safeSubstring(0, content.localComposing.start)
        val plan = NlpState.call(NlpOp.TYPING_POLICY, *environment.fields(subtype, content, suggesterFor(subtype) != null),
            *with(NlpTyping) { row(BEFORE to before, TYPED to content.composingText, AFTER to content.textAfterSelection) })
        if (plan.isEmpty()) return null
        val taps = tapTrail.offsetsFor(plan[NlpTyping.TYPED], aimed = prefs.suggestion.learnTapAim.get())
        return TypingState.of(plan[NlpTyping.BEFORE], plan[NlpTyping.TYPED], plan[NlpTyping.AFTER]).copy(taps = taps)
    }

    private fun ArrangedWord.toCandidate(autoCorrectAllowed: Boolean) = WordSuggestionCandidate(
        text = text,
        isEligibleForAutoCommit = autoCorrectAllowed,
        // The dictionary is read-only, so there is nothing for a long-press to remove.
        isEligibleForUserRemoval = false,
        // We set ourselves as the source provider so we can get notify events for our candidate
        sourceProvider = this@LatinLanguageProvider,
    )

    override suspend fun notifySuggestionAccepted(subtype: Subtype, candidate: SuggestionCandidate) {
        // Nothing to learn: the dictionary is read-only. (Candidate text is never logged.)
    }

    override suspend fun notifySuggestionReverted(subtype: Subtype, candidate: SuggestionCandidate) {
        // Nothing to unlearn, see notifySuggestionAccepted.
    }

    override suspend fun removeSuggestion(subtype: Subtype, candidate: SuggestionCandidate): Boolean {
        return false
    }

    /** The same list until [swipeWordsVersion] goes up. */
    override suspend fun getListOfWords(subtype: Subtype): List<String> {
        return if (usesDictionary(subtype)) swipeWords.words else testWords
    }

    override suspend fun getFrequencyForWord(subtype: Subtype, word: String): Double {
        if (usesDictionary(subtype)) return swipeWords.tunedFrequency(word)
        return wordData.withLock { it.getOrDefault(word, 0) / 255.0 }
    }

    override suspend fun destroy() {
        // Might never be called: the app process is usually killed instead.
        dictionaryLoadLock.withLock {
            suggester = null
            synchronized(swipeWordsLock) {
                swipeWords = SwipeWords.NONE
                swipeWordsFromDictionary = false
            }
            mutableSwipeWordsVersion.update { it + 1 }
            engine?.close()
            engine = null
        }
    }
}
