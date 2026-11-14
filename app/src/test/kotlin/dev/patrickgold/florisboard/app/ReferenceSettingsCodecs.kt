package dev.patrickgold.florisboard.app

import dev.patrickgold.florisboard.ime.core.*
import dev.patrickgold.florisboard.ime.media.emoji.*
import dev.patrickgold.florisboard.ime.smartbar.quickaction.*
import kotlinx.serialization.json.Json

internal object ReferenceSettingsCodecs {
    fun history(h: EmojiHistory) = Json.encodeToString(h)
    fun history(raw: String): EmojiHistory = try { Json.decodeFromString(raw) } catch (_: Exception) { EmojiHistory(emptyList(),emptyList()) }
    fun arrangement(a: QuickActionArrangement) = QuickActionJsonConfig.encodeToString(a)
    fun arrangement(raw: String) = QuickActionJsonConfig.decodeFromString<QuickActionArrangement>(raw)
    fun subtypes(values: List<Subtype>) = SubtypeJsonConfig.encodeToString(values)
    fun subtypes(raw: String) = SubtypeJsonConfig.decodeFromString<List<Subtype>>(raw)
}
