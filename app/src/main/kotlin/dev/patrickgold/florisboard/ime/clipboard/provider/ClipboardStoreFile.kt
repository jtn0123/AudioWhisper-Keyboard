package dev.patrickgold.florisboard.ime.clipboard.provider

import android.content.Context
import androidx.core.net.toUri
import com.jtn0123.audiowhisper.rust.ClipFlag
import com.jtn0123.audiowhisper.rust.ClipboardStore
import com.jtn0123.audiowhisper.rust.StoredClip
import com.jtn0123.audiowhisper.stores.LegacyRoom
import java.io.File

/**
 * The clipboard's store, one per process, which [dev.patrickgold.florisboard.ime.clipboard.ClipboardManager] and
 * [ClipboardMediaProvider] share: the history and its media files' records, kept by Rust (crates/stores) in
 * no_backup/clipboard.store, left out of backups as the Room databases were. Opening it moves main's Room
 * databases into it, once.
 */
internal object ClipboardStoreFile {
    const val NAME = "clipboard.store"

    @Volatile private var store: ClipboardStore? = null

    fun get(context: Context): ClipboardStore = store ?: synchronized(this) {
        store ?: open(context.applicationContext ?: context).also { store = it }
    }

    private fun open(context: Context) = ClipboardStore(File(context.noBackupFilesDir, NAME).path).also {
        LegacyRoom.importClipboard(context, it)
    }
}

/** Copies [item] into the history as [ClipboardStore.add] does; its new row's id. */
internal fun ClipboardStore.addItem(item: ClipboardItem, now: Long, limitEnabled: Boolean, limit: Int): Long = add(
    item.type.value, item.creationTimestampMs, item.storeFlags(), item.text, item.uri?.toString(),
    item.mimeTypes.joinToString(","), now, limitEnabled, limit)

/** Adds each of [items] the history doesn't have yet, as [ClipboardStore.restore] does; how many it added. */
internal fun ClipboardStore.restoreItems(items: List<ClipboardItem>, now: Long, limitEnabled: Boolean, limit: Int): Int =
    restore(items.map { it.type.value }.toIntArray(), items.map { it.creationTimestampMs }.toLongArray(),
        items.map { it.storeFlags() }.toIntArray(), items.map { it.text }.toTypedArray(),
        items.map { it.uri?.toString() }.toTypedArray(), items.map { it.mimeTypes.joinToString(",") }.toTypedArray(),
        now, limitEnabled, limit)

/** The [ClipFlag] codes of this clip's flags. */
private fun ClipboardItem.storeFlags(): Int = (if (isPinned) ClipFlag.PINNED.code else 0) or
    (if (isSensitive) ClipFlag.SENSITIVE.code else 0) or (if (isRemoteDevice) ClipFlag.REMOTE.code else 0)

/** This stored clip as the history shows it, its MIME types split as Room's converter split them; null for a
 * type this build doesn't know. */
internal fun StoredClip.toItem(): ClipboardItem? {
    val type = ItemType.entries.firstOrNull { it.value == kind } ?: return null
    fun has(flag: ClipFlag) = flags and flag.code != 0
    return ClipboardItem(id, type, text.takeIf { has(ClipFlag.HAS_TEXT) }, uri.takeIf { has(ClipFlag.HAS_URI) }?.toUri(),
        created, has(ClipFlag.PINNED), mimeTypes.split(","), has(ClipFlag.SENSITIVE), has(ClipFlag.REMOTE))
}
