package dev.patrickgold.florisboard.ime.core

import kotlinx.serialization.json.Json

// Original persisted codec, retained only for differential tests.
val SubtypeJsonConfig = Json { encodeDefaults = true; ignoreUnknownKeys = true; isLenient = false }
