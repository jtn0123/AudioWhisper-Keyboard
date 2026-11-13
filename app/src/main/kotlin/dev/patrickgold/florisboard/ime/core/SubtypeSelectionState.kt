package dev.patrickgold.florisboard.ime.core

import dev.patrickgold.florisboard.ime.keyboard.KeyboardState

/** Shared modal visibility state; neither renderer owns this Android host predicate. */
fun KeyboardState.isSubtypeSelectionShowing(): Boolean = isSubtypeSelectionVisible
