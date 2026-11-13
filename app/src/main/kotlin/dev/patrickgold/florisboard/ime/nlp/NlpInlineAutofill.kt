/* Copyright (C) 2024-2025 The FlorisBoard Contributors. Licensed under Apache-2.0. */
package dev.patrickgold.florisboard.ime.nlp

import android.content.Context
import android.util.Size
import android.view.ViewGroup
import android.view.inputmethod.InlineSuggestion
import android.view.inputmethod.InlineSuggestionInfo
import android.widget.inline.InlineContentView
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

data class NlpInlineAutofillSuggestion(val info: InlineSuggestionInfo, val view: InlineContentView?)

/** Android inflates views; native generations, pinned ordering and shared visibility own decisions. */
object NlpInlineAutofill {
    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
    val suggestions = NlpState.inline
    fun showInlineSuggestions(context: Context, rawSuggestions: List<InlineSuggestion>): Boolean {
        val plan = NlpState.inlineBegin(rawSuggestions.size)
        if (plan[1] != "1") return false
        scope.launch {
            val size = Size(ViewGroup.LayoutParams.WRAP_CONTENT, FlorisImeSizing.Static.smartbarHeightPx)
            val latch = CountDownLatch(rawSuggestions.size)
            val views = Array<NlpInlineAutofillSuggestion?>(rawSuggestions.size) { null }
            rawSuggestions.forEachIndexed { index, raw ->
                raw.inflate(context, size, context.mainExecutor) { view ->
                    views[index] = NlpInlineAutofillSuggestion(raw.info, view)
                    latch.countDown()
                }
            }
            if (latch.await(plan[2].toLong(), TimeUnit.MILLISECONDS)) {
                NlpState.inlineComplete(plan[0], views.filterNotNull())
            } else { NlpState.call("inlineCancel", plan[0]) }
        }
        return true
    }
    fun clearInlineSuggestions() { NlpState.inlineBegin(0) }
}
