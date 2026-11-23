package com.jtn0123.audiowhisper.render

import android.content.Context
import android.content.res.Configuration
import android.content.res.Resources
import android.graphics.Rect
import android.util.Log
import android.util.Size
import android.view.MotionEvent
import android.view.View
import android.view.ViewGroup
import android.view.WindowInsets
import android.view.WindowManager
import android.widget.FrameLayout
import androidx.core.graphics.Insets
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.jtn0123.audiowhisper.dictation.Dictation
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.rust.ImeWindowFacts as F
import com.jtn0123.audiowhisper.rust.ImeWindowPlan as P
import com.jtn0123.audiowhisper.rust.ImeWindowRules
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.nlp.NlpInlineAutofill
import dev.patrickgold.florisboard.ime.onehanded.OneHandedMode
import dev.patrickgold.florisboard.ime.popup.GlobalStateNumPopupsShowing
import kotlinx.coroutines.launch

private const val TRACE_TAG = "aw-profile"

/**
 * The input view while the Rust renderer is on: plain Views and no composition. Android measures
 * the window and reads the settings; Rust plans where the background, the one-handed panels, the
 * keyboard and the inline autofill chips go (`ImeWindowRules.layout`), as Compose's root put them.
 */
internal class ImeRootView(context: Context) : FrameLayout(context) {
    private val service = context as FlorisImeService
    private val reads = RootReads(service)
    private val watch = RootWatch(::rebuild)
    private val emoji = EmojiSource(reads, watch::request)
    private val gif = GifSource(reads)
    private val backdrop = ImeBackdrop(service)
    private val sides = listOf(OneHandedMode.START, OneHandedMode.END)
    private val panels = sides.map { side -> RustChromeView(service).also { it.action = { op -> oneHanded(side, op) } } }
    private val chips = ImeInlineChips(service)
    private var keyboard: RustKeyboardView? = null
    private val windowState = InputViewVisibility(::dispatchWindowVisibilityChanged)
    private var plan = FloatArray(0)
    private var safe = Insets.NONE
    private var specs = intArrayOf(0, 0)
    /** The status bar's height, asked of the window manager once per insets or configuration. */
    private var status = -1
    /** A finger is down on this view. */
    private var held = false

    init {
        layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
        isHapticFeedbackEnabled = true
        addView(backdrop)
    }

    override fun onAttachedToWindow() {
        // A root is attached once; should it come back, it gets a fresh keyboard, as Compose's would.
        keyboard?.let(::removeView)
        keyboard = RustKeyboardView(context).also { addView(it, 1) }
        super.onAttachedToWindow()
        reads.scope = watch.start(reads.sources())
        emoji.load()
        service.updateSoftInputWindowLayoutParameters()
        // The keyboard's surface reaches above this view for popups; nothing above may clip it.
        generateSequence(parent as? ViewGroup) { it.parent as? ViewGroup }.forEach { it.clipChildren = false }
        // Insets reach a view at the window's first traversal or when they change. A root swapped
        // in on a window that already exists gets neither, so it reads them and asks again.
        rootWindowInsets?.let { safe = safeDrawing(it, this) }
        requestApplyInsets()
        // The first frame draws keys, as Compose's did: a snapshot before the first layout.
        rebuild()
    }

    override fun onDetachedFromWindow() {
        watch.stop()
        keyboard?.release()
        super.onDetachedFromWindow()
    }

    override fun onApplyWindowInsets(insets: WindowInsets): WindowInsets {
        // The window's own insets, as Compose's safe drawing read them: in full-screen mode an
        // ancestor consumes the navigation bar's before they reach this view.
        val next = safeDrawing(rootWindowInsets ?: insets, this)
        if (next != safe) {
            safe = next
            refresh()
        }
        return super.onApplyWindowInsets(insets)
    }

    override fun onConfigurationChanged(newConfig: Configuration?) {
        super.onConfigurationChanged(newConfig)
        refresh()
    }

    /** Something the plan reads changed outside the watched flows: the window's mode or its language. */
    fun refresh() {
        status = -1
        requestLayout()
        watch.request()
    }

    override fun onWindowVisibilityChanged(visibility: Int) {
        super.onWindowVisibilityChanged(visibility)
        windowState.told(visibility)
    }

    /** The input method window is being shown (see [InputViewVisibility]). */
    fun windowShown() {
        val retold = windowState.shown(windowVisibility)
        if (Log.isLoggable(TRACE_TAG, Log.DEBUG)) Log.d(TRACE_TAG, "input view shown retold=${if (retold) 1 else 0}")
    }

    /** Rust's plan for this view measured by [widthSpec] and [heightSpec], or null if it has no window yet. */
    private fun layout(widthSpec: Int, heightSpec: Int): FloatArray? {
        if (!isAttachedToWindow) return null
        return ImeWindowRules.layout(facts(widthSpec, heightSpec) + smartbarFacts(reads,
            Dictation.preview.value.isNotEmpty(), NlpInlineAutofill.suggestions.value.isNotEmpty()))
    }

    private fun facts(widthSpec: Int, heightSpec: Int): FloatArray {
        val res = service.resources
        val metrics = res.displayMetrics
        val k = reads.prefs.keyboard
        val state = reads.keyboard.activeState
        val keyboard = reads.keyboard.activeEvaluator.value.keyboard
        val nav = rootWindowInsets?.let { WindowInsetsCompat.toWindowInsetsCompat(it).getInsets(WindowInsetsCompat.Type.navigationBars()).bottom } ?: 0
        // The IME's own insets have no status bar, so Compose asked the window manager for it.
        if (status < 0) status = service.getSystemService(WindowManager::class.java).currentWindowMetrics.windowInsets
            .getInsets(WindowInsets.Type.statusBars()).top
        fun bit(on: Boolean) = if (on) 1 else 0
        val values = mapOf<Int, Number>(F.DENSITY to metrics.density, F.SYSTEM_DENSITY to Resources.getSystem().displayMetrics.density,
            F.WIDTH to (MeasureSpec.getSize(widthSpec).takeIf { it > 0 } ?: metrics.widthPixels), F.HEIGHT to MeasureSpec.getSize(heightSpec),
            F.HEIGHT_MODE to when (MeasureSpec.getMode(heightSpec)) { MeasureSpec.EXACTLY -> 2; MeasureSpec.AT_MOST -> 1; else -> 0 },
            F.FULLSCREEN to bit(service.fullscreenUiMode.value), F.EXTRACT_SHOWN to bit(service.extractUiShown.value),
            F.SAFE_LEFT to safe.left, F.SAFE_TOP to safe.top, F.SAFE_RIGHT to safe.right, F.SAFE_BOTTOM to safe.bottom,
            F.LANDSCAPE to bit(reads.landscape), F.OFFSET_PORTRAIT to k.bottomOffsetPortrait.get(), F.OFFSET_LANDSCAPE to k.bottomOffsetLandscape.get(),
            F.ONE_HANDED to bit(k.oneHandedModeEnabled.get()), F.ONE_HANDED_END to bit(k.oneHandedMode.get() == OneHandedMode.END),
            F.ONE_HANDED_SCALE to k.oneHandedModeScaleFactor.get(), F.HEIGHT_PORTRAIT to k.heightFactorPortrait.get(),
            F.HEIGHT_LANDSCAPE to k.heightFactorLandscape.get(), F.DISPLAY_WIDTH to metrics.widthPixels, F.DISPLAY_HEIGHT to metrics.heightPixels,
            F.NAVIGATION_BAR to nav, F.STATUS_BAR to status, F.MIN_FRACTION to res.getFraction(R.fraction.inputView_minHeightFraction, 1, 1),
            F.MAX_FRACTION to res.getFraction(R.fraction.inputView_maxHeightFraction, 1, 1), F.BASE_HEIGHT to res.getDimension(R.dimen.inputView_baseHeight),
            F.MODE to keyboard.mode.value, F.ROWS to keyboard.rowCount, F.CHARACTER_ROWS to reads.keyboard.lastCharactersEvaluator.value.keyboard.rowCount,
            F.EDITOR to bit(state.isActionsEditorVisible), F.OVERFLOW to bit(state.isActionsOverflowVisible), F.UI_MODE to state.imeUiMode.value,
            F.GIFS to bit(GifSearch.showingGifs.value), F.SEARCH to bit(GifSearch.typing.value))
        return FloatArray(F.SIZE).also { for ((i, v) in values) it[i] = v.toFloat() }
    }

    /** Builds everything from the state now: the plan, the keyboard's snapshot, the panels, chips and background. */
    private fun rebuild() {
        val next = layout(specs[0], specs[1]) ?: return
        if (!next.contentEquals(plan)) requestLayout().also { plan = next }
        FlorisImeSizing.Static.keyboardRowBaseHeightPx = next[P.ROW_PIXELS].toInt()
        FlorisImeSizing.Static.smartbarHeightPx = next[P.SMARTBAR_PIXELS].toInt()
        reads.rowHeight = next[P.ROW_HEIGHT]
        val rtl = service.resourcesContext.resources.configuration.layoutDirection == View.LAYOUT_DIRECTION_RTL
        if (reads.keyboard.activeState.isRtl != rtl) reads.keyboard.activeState.isRtl = rtl
        keyboard?.update(snapshot(next))
        val chrome = reads.theme(stateful = true)
        for ((i, panel) in panels.withIndex()) {
            val shown = next[if (i == 0) P.START_WIDTH else P.END_WIDTH] > 0
            if (shown != (panel.parent != null)) if (shown) addView(panel) else removeView(panel)
            panel.update(chrome, 0, arrayOf(reads.string(R.string.one_handed__close_btn_content_description), reads.string(
                if (i == 0) R.string.one_handed__move_start_btn_content_description else R.string.one_handed__move_end_btn_content_description)), side = i)
        }
        val inline = next[P.INLINE_WIDTH] > 0
        if (inline != (chips.parent != null)) if (inline) addView(chips) else removeView(chips)
        if (inline) chips.update(NlpInlineAutofill.suggestions.value, GlobalStateNumPopupsShowing.value)
        backdrop.update(chrome, reads.themes.activeThemeInfo.value.loadedDir)?.let(::bars)
        rise()
    }

    override fun dispatchTouchEvent(event: MotionEvent): Boolean {
        val handled = super.dispatchTouchEvent(event)
        held = event.actionMasked != MotionEvent.ACTION_UP && event.actionMasked != MotionEvent.ACTION_CANCEL
        rise()
        return handled
    }

    /** Lifts the keyboard's surface over the window where Rust's plan says to, while it may draw above the keys. */
    private fun rise() {
        keyboard?.rise(plan.getOrElse(P.RISE) { 0f } != 0f && (held || GlobalStateNumPopupsShowing.value > 0))
    }

    private fun snapshot(plan: FloatArray): KeyboardSnapshot {
        val evaluator = reads.keyboard.activeEvaluator.value
        val keyboard = RustKeyboard.of(evaluator.keyboard)
        val preview = Dictation.preview.value
        val search = rendererSearch(reads)
        val base = smartbarFacts(reads, preview.isNotEmpty(), NlpInlineAutofill.suggestions.value.isNotEmpty())
        base[10] = reads.density
        base[11] = plan[P.SMARTBAR_ROW]
        val bar = rendererSmartbar(reads, base).let {
            if (search.active) it.copy(config = plan.copyOfRange(P.SIZE, plan.size), animating = false) else it
        }
        val panels = rendererPanels(reads, plan[P.PANEL].toInt(), emoji.read(), rendererClipboard(reads), gif.read())
        val candidates = if (preview.isEmpty() && !search.active) reads.nlp.activeCandidatesFlow.value else emptyList()
        val spacing = reads.prefs.keyboard
        return KeyboardSnapshot(keyboard, keyboard.keys, candidates, reads.theme(stateful = false), evaluator.state.inputShiftState.toString(),
            plan[P.KEYBOARD_PX], plan[P.STRIP_PX], plan[P.ROW_HEIGHT], spacing.keySpacingHorizontal.get() * reads.density,
            spacing.keySpacingVertical.get() * reads.density, bar, preview.isNotEmpty(), panels,
            intArrayOf(evaluator.editorInfo.imeOptions.raw, evaluator.editorInfo.inputAttributes.raw), search, rendererWatermark(reads), preview)
    }

    /** The navigation bar's buttons and contrast, as Compose's `SystemUiIme` set them. */
    private fun bars(flags: Pair<Boolean, Boolean>) {
        val window = service.window?.window ?: return
        WindowInsetsControllerCompat(window, this).isAppearanceLightNavigationBars = flags.first
        window.isNavigationBarContrastEnforced = flags.second
    }

    private fun oneHanded(side: OneHandedMode, op: Int) {
        reads.scope.launch {
            service.inputFeedbackController.keyPress()
            if (op == 0) reads.prefs.keyboard.oneHandedModeEnabled.set(false) else reads.prefs.keyboard.oneHandedMode.set(side)
        }
    }

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        specs = intArrayOf(widthMeasureSpec, heightMeasureSpec)
        layout(widthMeasureSpec, heightMeasureSpec)?.let { if (!it.contentEquals(plan)) watch.request().also { _ -> plan = it } }
        val width = MeasureSpec.getSize(widthMeasureSpec)
        setMeasuredDimension(width, plan.getOrElse(P.VIEW_HEIGHT) { 0f }.toInt())
        for ((child, rect) in placements()) {
            child.measure(MeasureSpec.makeMeasureSpec(rect.width(), MeasureSpec.EXACTLY), MeasureSpec.makeMeasureSpec(rect.height(), MeasureSpec.EXACTLY))
        }
    }

    override fun onLayout(changed: Boolean, left: Int, top: Int, right: Int, bottom: Int) {
        for ((child, rect) in placements()) child.layout(rect.left, rect.top, rect.right, rect.bottom)
        if (plan.isNotEmpty()) reportSize(right - left, plan[P.BOX_HEIGHT].toInt())
    }

    /** The input view's size for the service's insets; a new [Size] only when it changed, since layout runs often. */
    private fun reportSize(width: Int, height: Int) {
        val size = service.inputViewSize
        if (size.width != width || size.height != height) service.inputViewSize = Size(width, height)
    }

    /** Each child's rectangle in this view, from the plan. */
    private fun placements(): List<Pair<View, Rect>> {
        if (plan.isEmpty()) return emptyList()
        val p = plan.map { it.toInt() }
        val box = p[P.BOX_TOP]
        val keys = box + p[P.KEYBOARD_TOP]
        fun rect(left: Int, top: Int, width: Int, height: Int) = Rect(left, top, left + width, top + height)
        return listOfNotNull(backdrop to Rect(0, box, measuredWidth, box + p[P.BOX_HEIGHT]),
            keyboard?.let { it to rect(p[P.KEYBOARD_LEFT], keys + p[P.SURFACE_TOP], p[P.KEYBOARD_WIDTH], p[P.SURFACE_HEIGHT]) },
            panels[0] to rect(p[P.START_LEFT], box + p[P.START_TOP], p[P.START_WIDTH], p[P.START_HEIGHT]),
            panels[1] to rect(p[P.END_LEFT], box + p[P.END_TOP], p[P.END_WIDTH], p[P.END_HEIGHT]),
            chips to rect(p[P.KEYBOARD_LEFT] + p[P.INLINE_LEFT], keys + p[P.INLINE_TOP], p[P.INLINE_WIDTH], p[P.INLINE_HEIGHT]))
            .filter { it.first.parent === this }
    }
}
