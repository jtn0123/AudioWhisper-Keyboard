package dev.patrickgold.florisboard.ime.sheet

import dev.patrickgold.florisboard.ime.keyboard.KeyboardState

/** Shared modal visibility state; neither renderer owns this Android host predicate. */
fun KeyboardState.isBottomSheetShowing(): Boolean = isActionsEditorVisible
