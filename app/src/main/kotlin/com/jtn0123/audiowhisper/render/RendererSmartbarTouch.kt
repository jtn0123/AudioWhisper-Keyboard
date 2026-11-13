package com.jtn0123.audiowhisper.render

import android.widget.FrameLayout
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.subtypeManager

/** Platform calls only. Pointer ownership, toggle consumption and hold deadlines are native. */
internal class RendererSmartbarTouch(view: FrameLayout, private val onCandidateRemoved: (Int) -> Unit) {
    private val context = view.context
    private val manager by context.keyboardManager()
    private val nlp by context.nlpManager()
    private val subtype by context.subtypeManager()
    private val tooltip = RendererActionTooltip(view)

    fun dispatch(type: Int, index: Int, pointer: Int, s: KeyboardSnapshot) {
        val action = s.smartbar.actions.getOrNull(index)
        val enabled = s.smartbar.codes.getOrNull(index * 2 + 1)?.and(BAR_ENABLED) == BAR_ENABLED
        when (type) {
            BAR_DOWN -> if (enabled) {
                FlorisImeService.inputFeedbackController()?.keyPress(TextKeyData.UNSPECIFIED)
                action?.onPointerDown(context)
            }
            BAR_UP -> { tooltip.up(); if (enabled) action?.onPointerUp(context) }
            BAR_CANCEL -> { tooltip.clear(); if (enabled) action?.onPointerCancel(context) }
            BAR_TOGGLE -> s.smartbar.toggle()
            TOOLTIP_SHOW -> tooltip.show(index, s)
            CANDIDATE_HOLD -> s.candidates.getOrNull(index)?.let { candidate ->
                if (candidate.isEligibleForUserRemoval && nlp.removeSuggestion(subtype.activeSubtype, candidate)) {
                    onCandidateRemoved(pointer)
                }
            }
            CANDIDATE_UP -> s.candidates.getOrNull(index)?.let(manager::commitCandidate)
            CANDIDATE_DOWN, CANDIDATE_CANCEL -> Unit
        }
    }
    fun cancel() = tooltip.clear()
}
