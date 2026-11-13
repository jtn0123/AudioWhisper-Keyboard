package com.jtn0123.audiowhisper.render

import android.widget.FrameLayout
import dev.patrickgold.florisboard.ime.smartbar.quickaction.keyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.subtypeManager

/** Calls exactly the same QuickAction handlers as QuickActionButton, including cancellation. */
internal class RendererSmartbarTouch(private val view: FrameLayout) {
    private val context = view.context
    private val prefs by FlorisPreferenceStore
    private val manager by context.keyboardManager()
    private val nlp by context.nlpManager()
    private val subtype by context.subtypeManager()
    private val actions = mutableMapOf<Int, QuickAction>()
    private val actionIndices = mutableMapOf<Int, Int>()
    private val candidateTimers = mutableMapOf<Int, Runnable>()
    private val timedOut = mutableSetOf<Int>()
    private val tooltip = RendererActionTooltip(view)
    private val tooltipConsumed = mutableSetOf<Int>()
    private val toggles = mutableSetOf<Int>()

    fun dispatch(type: Int, index: Int, pointer: Int, s: KeyboardSnapshot) {
        when (type) {
            5 -> {
                if (index == 4) { toggles.add(pointer); return }
                s.smartbar.actions.getOrNull(index)?.let {
                    actions[pointer] = it
                    actionIndices[pointer] = index
                    FlorisImeService.inputFeedbackController()?.keyPress(TextKeyData.UNSPECIFIED)
                    it.onPointerDown(context)
                    if (it.keyData().code != KeyCode.READ_ALOUD) tooltip.down(index,s) {
                        tooltipConsumed.add(pointer)
                    }
                }
            }
            6 -> {
                actionIndices.remove(pointer)
                if (toggles.remove(pointer)) s.smartbar.toggle()
                tooltip.up()
                if (tooltipConsumed.remove(pointer)) actions.remove(pointer)?.onPointerCancel(context)
                else actions.remove(pointer)?.onPointerUp(context)
            }
            7 -> { actionIndices.remove(pointer); tooltip.up(); tooltipConsumed.remove(pointer); toggles.remove(pointer); actions.remove(pointer)?.onPointerCancel(context) }
            8 -> {
                val candidate = s.candidates.getOrNull(index) ?: return
                val timer = Runnable {
                    timedOut.add(pointer)
                    if (candidate.isEligibleForUserRemoval) nlp.removeSuggestion(subtype.activeSubtype, candidate)
                }
                candidateTimers.put(pointer, timer)?.let(view::removeCallbacks)
                view.postDelayed(timer, prefs.keyboard.longPressDelay.get().toLong())
            }
            9, 10 -> {
                candidateTimers.remove(pointer)?.let(view::removeCallbacks)
                val held = timedOut.remove(pointer)
                if (type == 9 && !held) s.candidates.getOrNull(index)?.let(manager::commitCandidate)
            }
        }
    }
    fun sync(s: KeyboardSnapshot) {
        for ((pointer, action) in actions.toMap()) {
            val index = actionIndices.getValue(pointer)
            if (s.smartbar.actions.getOrNull(index) != action || s.smartbar.codes[index * 2 + 1] and 1 == 0) {
                actions.remove(pointer)?.onPointerCancel(context)
                actionIndices.remove(pointer)
                tooltip.clear()
            }
        }
    }
    fun cancel() {
        tooltip.clear()
        tooltipConsumed.clear()
        actions.values.forEach { it.onPointerCancel(context) }
        actions.clear()
        actionIndices.clear()
        candidateTimers.values.forEach(view::removeCallbacks)
        candidateTimers.clear()
        timedOut.clear()
        toggles.clear()
    }
}
