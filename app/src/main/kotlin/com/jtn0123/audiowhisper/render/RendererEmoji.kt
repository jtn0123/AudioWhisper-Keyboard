package com.jtn0123.audiowhisper.render

import android.app.KeyguardManager
import android.graphics.Paint
import android.graphics.Typeface
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.media.emoji.*
import kotlinx.coroutines.launch

internal data class RendererEmoji(val emojis: List<Emoji>, val config: IntArray, val groups: IntArray,
    val history: IntArray, val labels: Array<String>) {
    fun bind(native: KeyboardRenderer, refresh: Boolean = false) {
        val c = config.copyOf(); c[7] = if (refresh) 1 else 0
        native.emoji(c, groups, emojis.map { it.value }.toTypedArray(), history, labels)
    }
}

/**
 * Android glyph support, font metadata and preferences only. Rust selects bases/variants/pages.
 * The emoji list loads once, and the supported glyphs and their ids are kept while what they come
 * from is unchanged, as Compose remembered them.
 */
internal class EmojiSource(private val r: RootReads, private val loaded: () -> Unit) {
    private var data = EmojiData.Fallback
    private var font: Pair<List<Any?>, EmojiFont?>? = null
    private var supported: Pair<List<Any?>, List<Pair<Int, List<Emoji>>>>? = null
    private var emojis: Pair<Any, MutableList<Emoji>>? = null
    private val paint = Paint().apply { typeface = Typeface.DEFAULT }

    fun load() = r.scope.launch { data = EmojiData.get(r.service, "ime/media/emoji/root.txt"); loaded() }

    private fun supported(): List<Pair<Int, List<Emoji>>> {
        val info = r.editor.activeInfo
        val compat = FlorisEmojiCompat.instance.value
        val fontKey = listOf(compat, info.emojiCompatReplaceAll)
        val font = font?.takeIf { it.first == fontKey }?.second
            ?: compat?.let { EmojiFont(it, info.emojiCompatReplaceAll) }.also { font = fontKey to it }
        val key = listOf(data, font, info.emojiCompatMetadataVersion)
        supported?.takeIf { it.first == key }?.let { return it.second }
        return data.byCategory.flatMap { (category, sets) -> sets.mapNotNull { set ->
            set.emojis.filter { font?.isSupported(it.value, info.emojiCompatMetadataVersion) == true || paint.hasGlyph(it.value) }
                .takeIf { it.isNotEmpty() }?.let { category.ordinal to it }
        } }.also { supported = key to it }
    }

    fun read(): RendererEmoji {
        val prefs = r.prefs.emoji
        val supported = supported()
        // Stable opaque ids preserve a frozen recents page even when a history limit
        // drops an unsupported glyph that remains visible until the next category entry.
        val emojis = emojis?.takeIf { it.first === supported }?.second
            ?: supported.flatMap { it.second }.toMutableList().also { emojis = supported to it }
        val groups = mutableListOf<Int>()
        var start = 0
        for ((category, entries) in supported) { groups.addAll(listOf(category, start, entries.size)); start += entries.size }
        val history = prefs.historyData.get()
        for (emoji in history.pinned + history.recent) if (emoji !in emojis) emojis.add(emoji)
        val keyguard = r.service.getSystemService(KeyguardManager::class.java)
        val config = listOf(if (prefs.historyEnabled.get()) 1 else 0, if (keyguard.isDeviceLocked || keyguard.isKeyguardLocked) 1 else 0,
            prefs.preferredSkinTone.get().ordinal, prefs.historyPinnedUpdateStrategy.get().ordinal,
            prefs.historyRecentUpdateStrategy.get().ordinal, (r.rowHeight * 0.8f).toInt(),
            if (GifSearch.available) 1 else 0, 0) + emojis.map { it.skinTone.ordinal }
        val labels = EmojiCategory.entries.map { it.id.replace('_', ' ') } + listOf(R.string.emoji__history__phone_locked_message,
            R.string.emoji__history__empty_message, R.string.emoji__history__usage_tip, R.string.emoji__history__pinned,
            R.string.emoji__history__recent).map { r.string(it) }
        return RendererEmoji(emojis, config.toIntArray(), groups.toIntArray(),
            (listOf(history.pinned.size) + history.pinned.map { emojis.indexOf(it) } + history.recent.map { emojis.indexOf(it) }).toIntArray(), labels.toTypedArray())
    }
}
