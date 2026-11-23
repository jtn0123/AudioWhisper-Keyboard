package com.jtn0123.audiowhisper.render

import android.view.Choreographer
import com.jtn0123.audiowhisper.dictation.Dictation
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.reader.ReadAloud
import com.jtn0123.audiowhisper.rust.ImeChromeRules
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.media.emoji.FlorisEmojiCompat
import dev.patrickgold.florisboard.ime.nlp.NlpInlineAutofill
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.displayNameRes
import dev.patrickgold.florisboard.ime.smartbar.quickaction.tooltipRes
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.ime.popup.GlobalStateNumPopupsShowing
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.subtypeManager
import dev.patrickgold.florisboard.themeManager
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import org.florisboard.lib.android.isOrientationLandscape
import org.florisboard.lib.kotlin.CurlyArg
import org.florisboard.lib.kotlin.curlyFormat

/**
 * What the plain-View root's adapters read where Compose's read its locals: the service's
 * managers and settings, strings in the keyboard's language, the display's scale and the row
 * height Rust planned. It reads; Rust decides.
 */
internal class RootReads(val service: FlorisImeService) {
    val prefs by FlorisPreferenceStore
    val keyboard by service.keyboardManager()
    val nlp by service.nlpManager()
    val editor by service.editorInstance()
    val clipboard by service.clipboardManager()
    val subtypes by service.subtypeManager()
    val themes by service.themeManager()
    /** Where the adapters' setting writes run: the root's while it is attached. */
    var scope: CoroutineScope = MainScope()
    /** A keyboard row's height in pixels, as Rust last planned it. */
    var rowHeight = 0f
    val density get() = service.resources.displayMetrics.density
    val landscape get() = service.resources.configuration.isOrientationLandscape()
    private var palette: Pair<List<Any?>, ThemePalette>? = null
    private var dates: Pair<Any, DateTimeFormatter>? = null

    /** String [id] in the keyboard's language, as Compose's `stringRes` gave it. */
    fun string(id: Int, vararg args: CurlyArg): String = service.resourcesContext.resources.getString(id)
        .curlyFormat("app_name" to service.getString(R.string.floris_app_name), *args)

    fun name(action: QuickAction): String = when (action) {
        is QuickAction.InsertKey -> string(displayNameRes(action.data.code))
        is QuickAction.InsertText -> action.data
    }

    fun tooltip(action: QuickAction): String = when (action) {
        is QuickAction.InsertKey -> string(tooltipRes(action.data.code))
        is QuickAction.InsertText -> "Insert text '${action.data}'"
    }

    /** The clipboard's dates, in the keyboard's language and this zone. */
    fun dates(): DateTimeFormatter {
        val context = service.resourcesContext
        dates?.takeIf { it.first === context }?.let { return it.second }
        val locales = context.resources.configuration.locales
        return DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM)
            .withLocale(if (locales.isEmpty) Locale.getDefault() else locales.get(0))
            .withZone(ZoneId.systemDefault()).also { dates = context to it }
    }

    /** `fontSizeMultiplier()`: the settings' multiplier for this orientation and one-handed mode. */
    fun fontMultiplier(): Float = with(prefs.keyboard) {
        ImeChromeRules.fontMultiplier(intArrayOf(
            (if (landscape) fontSizeMultiplierLandscape else fontSizeMultiplierPortrait).get(),
            oneHandedModeScaleFactor.get(), if (oneHandedModeEnabled.get()) 1 else 0, if (landscape) 0 else 1))
    }

    /** What the renderer draws in; [stateful] for the chrome, which reads the keyboard's mode and shift. */
    fun theme(stateful: Boolean): RendererTheme {
        val info = themes.activeThemeInfo.value
        val system = service.resources.getColor(android.R.color.system_accent1_500, service.theme)
        val accent = themes.preferredAccent()
        val key = listOf(accent, system, info.config.materialYouFlags)
        val made = palette?.takeIf { it.first == key }?.second
            ?: themePalette(accent, system, info.config.materialYouFlags).also { palette = key to it }
        val state = keyboard.activeState
        val scale = service.resources.configuration.fontScale
        return (if (stateful) rendererTheme(info, made, density, scale, fontMultiplier(), state.keyboardMode.toString(), state.inputShiftState.toString())
        else rendererTheme(info, made, density, scale, fontMultiplier())).copy(fonts = themeFontFiles(service, info))
    }

    /** Everything a rebuild reads that can change while the keyboard shows. */
    fun sources(): List<Flow<*>> {
        val k = prefs.keyboard
        val bar = prefs.smartbar
        val emoji = prefs.emoji
        val clips = prefs.clipboard
        return listOf(keyboard.activeState.flow, keyboard.activeEvaluator, keyboard.lastCharactersEvaluator,
            keyboard.activeSmartbarEvaluator, themes.activeThemeInfo, nlp.activeCandidatesFlow,
            nlp.activeCandidatesWordsFlow, Dictation.preview, Dictation.status, Dictation.level, Dictation.warming,
            ReadAloud.status, NlpInlineAutofill.suggestions, GlobalStateNumPopupsShowing, GifSearch.typing,
            GifSearch.query, GifSearch.showingGifs, GifSearch.results, GifSearch.shown, GifSearch.chips,
            editor.activeInfoFlow, FlorisEmojiCompat.instance, clipboard.historyFlow, subtypes.subtypesFlow,
            subtypes.activeSubtypeFlow) + listOf(prefs.theme.accentColor, k.fontSizeMultiplierPortrait,
            k.fontSizeMultiplierLandscape, k.oneHandedMode, k.oneHandedModeEnabled, k.oneHandedModeScaleFactor,
            k.heightFactorPortrait, k.heightFactorLandscape, k.bottomOffsetPortrait, k.bottomOffsetLandscape,
            k.incognitoDisplayMode, k.keySpacingHorizontal, k.keySpacingVertical, bar.enabled, bar.layout,
            bar.flipToggles, bar.sharedActionsExpanded, bar.extendedActionsExpanded, bar.extendedActionsPlacement,
            bar.showReadAloud, bar.actionArrangement, prefs.suggestion.displayMode, emoji.historyData,
            emoji.historyEnabled, emoji.preferredSkinTone, emoji.historyPinnedUpdateStrategy,
            emoji.historyRecentUpdateStrategy, clips.historyEnabled, clips.historyNumGridColumnsPortrait,
            clips.historyNumGridColumnsLandscape).map { it.asFlow() }
    }
}

/**
 * Runs [rebuild] once on the frame after any watched flow changes, as Compose recomposed once a
 * frame; watching from [start] to [stop], the view's attachment.
 */
internal class RootWatch(private val rebuild: () -> Unit) : Choreographer.FrameCallback {
    private var scope: CoroutineScope? = null
    private var pending = false

    fun start(flows: List<Flow<*>>): CoroutineScope = MainScope().also { scope ->
        this.scope?.cancel()
        this.scope = scope
        for (flow in flows) scope.launch { flow.collect { request() } }
    }

    fun request() {
        if (pending || scope == null) return
        pending = true
        Choreographer.getInstance().postFrameCallback(this)
    }

    override fun doFrame(frameTimeNanos: Long) {
        pending = false
        if (scope != null) rebuild()
    }

    fun stop() {
        scope?.cancel()
        scope = null
        pending = false
        Choreographer.getInstance().removeFrameCallback(this)
    }
}

/**
 * How long the strip keeps drawing its words while their refill is due. In the CPU trials of
 * 2026-11-23, on a loaded host, the strip stayed empty for at most 129 ms before the new words came.
 */
internal const val STRIP_HOLD_MS = 200L

/**
 * The words the strip draws across a refill. A key empties the row and Rust holds its place
 * (`wordsShown`) until the new words come, milliseconds later. A frame drawn in between shows the
 * row empty and then full: two changes for one key. While the refill is due the strip keeps the
 * words it drew, for at most [STRIP_HOLD_MS], so a key changes it once. A tap on a kept word is
 * refused, as on a frame drawn before the refill: the session dropped those words when the key
 * emptied the row. [wake] asks for a rebuild in so many milliseconds, replacing any asked before;
 * [cancel] withdraws it.
 */
internal class StripHold(private val wake: (Long) -> Unit, private val cancel: () -> Unit) {
    private var drawn = emptyList<SuggestionCandidate>()
    private var since = -1L

    /** What the strip draws at [now] (ms) for Rust's [candidates], with [due] whether their refill is. */
    fun words(candidates: List<SuggestionCandidate>, due: Boolean, now: Long): List<SuggestionCandidate> {
        if (candidates.isEmpty() && due && drawn.isNotEmpty()) {
            if (since < 0) since = now
            val left = since + STRIP_HOLD_MS - now
            if (left > 0) return drawn.also { wake(left) }
        }
        if (since >= 0) cancel()
        since = -1
        drawn = candidates
        return candidates
    }
}
