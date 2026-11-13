package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import java.util.IdentityHashMap

/** Opaque platform-object handles. Rust retains the handles owned by in-flight holds, so a
 * suggestion update cannot make a timer remove a different candidate. No pointer state lives here.
 */
internal class RendererCandidateBindings {
    private val objects = mutableMapOf<Int, SuggestionCandidate>()
    private val identities = IdentityHashMap<SuggestionCandidate, Int>()
    private var next = 0
    operator fun get(handle: Int) = objects[handle]
    fun bind(native: KeyboardRenderer, candidates: List<SuggestionCandidate>) {
        val handles = candidates.map { candidate -> identities.getOrPut(candidate) {
            (++next).also { objects[it] = candidate }
        } }.toIntArray()
        val retained = native.candidateBindings(handles).toSet() + handles.toSet()
        objects.keys.retainAll(retained)
        identities.entries.removeAll { it.value !in retained }
    }
    // Do not recycle a virtual candidate id after surface recreation while TalkBack retains it.
    fun clear() { objects.clear(); identities.clear() }
}
