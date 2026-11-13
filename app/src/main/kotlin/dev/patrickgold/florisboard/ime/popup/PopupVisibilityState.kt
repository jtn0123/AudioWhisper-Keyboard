package dev.patrickgold.florisboard.ime.popup

import kotlinx.coroutines.flow.MutableStateFlow

/** Shared Android inline-autofill/native-popup z-order state, independent of Compose UI. */
val GlobalStateNumPopupsShowing = MutableStateFlow(0)
