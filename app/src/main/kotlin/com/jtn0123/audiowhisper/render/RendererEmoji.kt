package com.jtn0123.audiowhisper.render

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.PushPin
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import android.app.KeyguardManager
import android.graphics.Paint
import android.graphics.Typeface
import androidx.compose.runtime.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.media.emoji.*
import dev.patrickgold.jetpref.datastore.model.observeAsState
import org.florisboard.lib.compose.stringRes

internal data class RendererEmoji(val emojis: List<Emoji>, val config: IntArray, val groups: IntArray,
    val history: IntArray, val labels: Array<String>) {
    fun bind(native: KeyboardRenderer, refresh: Boolean = false) {
        val c = config.copyOf(); c[7] = if (refresh) 1 else 0
        native.emoji(c, groups, emojis.map { it.value }.toTypedArray(), history, labels)
    }
}

/** Android glyph support, font metadata and preferences only. Rust selects bases/variants/pages. */
@Composable
internal fun rendererEmoji(): RendererEmoji = with(LocalDensity.current) {
    val context = LocalContext.current
    val prefs by FlorisPreferenceStore
    val editor by context.editorInstance()
    val info by editor.activeInfoFlow.collectAsState()
    val compat by FlorisEmojiCompat.instance.collectAsState()
    val font = remember(compat, info.emojiCompatReplaceAll) { compat?.let { EmojiFont(it, info.emojiCompatReplaceAll) } }
    var data by remember { mutableStateOf(EmojiData.Fallback) }
    LaunchedEffect(Unit) { data = EmojiData.get(context, "ime/media/emoji/root.txt") }
    val paint = remember { Paint().apply { typeface = Typeface.DEFAULT } }
    val supported = remember(data, font, info.emojiCompatMetadataVersion) {
        data.byCategory.flatMap { (category, sets) -> sets.mapNotNull { set ->
            set.emojis.filter { font?.isSupported(it.value, info.emojiCompatMetadataVersion) == true || paint.hasGlyph(it.value) }
                .takeIf { it.isNotEmpty() }?.let { category.ordinal to it }
        } }
    }
    val history by prefs.emoji.historyData.observeAsState()
    val enabled by prefs.emoji.historyEnabled.observeAsState()
    val tone by prefs.emoji.preferredSkinTone.observeAsState()
    val ps by prefs.emoji.historyPinnedUpdateStrategy.observeAsState()
    val rs by prefs.emoji.historyRecentUpdateStrategy.observeAsState()
    // Stable opaque ids preserve a frozen recents page even when a history limit
    // drops an unsupported glyph that remains visible until the next category entry.
    val emojis = remember(supported) { supported.flatMap { it.second }.toMutableList() }
    val groups = mutableListOf<Int>()
    var start = 0
    for ((category, entries) in supported) { groups.addAll(listOf(category, start, entries.size)); start += entries.size }
    for (emoji in history.pinned + history.recent) if (emoji !in emojis) emojis.add(emoji)
    val keyguard = context.getSystemService(KeyguardManager::class.java)
    val config = listOf(if (enabled) 1 else 0, if (keyguard.isDeviceLocked || keyguard.isKeyguardLocked) 1 else 0,
        tone.ordinal, ps.ordinal, rs.ordinal, (FlorisImeSizing.keyboardRowBaseHeight.toPx() * 0.8f).toInt(),
        if (GifSearch.available) 1 else 0, 0) + emojis.map { it.skinTone.ordinal }
    val labels = EmojiCategory.entries.map { it.id.replace('_', ' ') } + listOf(
        stringRes(R.string.emoji__history__phone_locked_message), stringRes(R.string.emoji__history__empty_message),
        stringRes(R.string.emoji__history__usage_tip), stringRes(R.string.emoji__history__pinned), stringRes(R.string.emoji__history__recent)) +
        EmojiCategory.entries.map { iconPath(it.icon()) } + listOf(Icons.Outlined.PushPin,
            Icons.AutoMirrored.Filled.KeyboardArrowLeft, Icons.AutoMirrored.Filled.KeyboardArrowRight, Icons.Outlined.Delete).map(::iconPath)
    RendererEmoji(emojis, config.toIntArray(), groups.toIntArray(),
        (listOf(history.pinned.size) + history.pinned.map { emojis.indexOf(it) } + history.recent.map { emojis.indexOf(it) }).toIntArray(), labels.toTypedArray())
}
