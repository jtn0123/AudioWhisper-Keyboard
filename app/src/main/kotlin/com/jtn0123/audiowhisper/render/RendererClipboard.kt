package com.jtn0123.audiowhisper.render

import android.app.KeyguardManager
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import java.time.Instant

internal data class RendererClipboard(val entries: List<ClipboardItem>, val config: LongArray, val labels: Array<String>) {
    fun bind(native: KeyboardRenderer) = native.clipboard(config,
        entries.flatMap { listOf(it.id,it.creationTimestampMs,it.type.value.toLong(),if(it.isPinned) 1L else 0L) }.toLongArray(),labels)
}

/** Android storage, resources and date formatting. Native owns filters, grids, menus and scrolling. */
internal fun rendererClipboard(r: RootReads): RendererClipboard {
    val history = r.clipboard.historyFlow.value
    val prefs = r.prefs.clipboard
    val columns = (if (r.landscape) prefs.historyNumGridColumnsLandscape else prefs.historyNumGridColumnsPortrait).get()
    val keyguard = r.service.getSystemService(KeyguardManager::class.java)
    val formatter = r.dates()
    val labels = listOf(r.string(R.string.clipboard__header_title), "", "") + listOf(
        R.string.clipboard__empty__title, R.string.clipboard__empty__message, R.string.clipboard__disabled__title,
        R.string.clipboard__disabled__message, R.string.clipboard__disabled__enable_button, R.string.clipboard__locked__title,
        R.string.clipboard__locked__message, R.string.clipboard__group_pinned, R.string.clipboard__group_recent,
        R.string.clipboard__group_other, R.string.clip__pin_item, R.string.clip__unpin_item, R.string.clip__delete_item,
        R.string.clip__paste_item, R.string.clipboard__confirm_clear_unfiltered_history__message,
        R.string.clipboard__confirm_clear_filtered_history__message, R.string.action__no, R.string.action__yes).map { r.string(it) }
    val descriptions=history.all.map { item ->
        val text=item.stringRepresentation()
        val network=dev.patrickgold.florisboard.lib.util.NetworkUtils
        when {
            network.isEmailAddress(text)->r.string(R.string.clipboard__item_description_email)
            network.isUrl(text)->r.string(R.string.clipboard__item_description_url)
            network.isPhoneNumber(text)->r.string(R.string.clipboard__item_description_phone)
            else->""
        }
    }
    return RendererClipboard(history.all,longArrayOf(System.currentTimeMillis(),if(prefs.historyEnabled.get())1 else 0,
        if(keyguard.isDeviceLocked || keyguard.isKeyguardLocked)1 else 0,columns.toLong()),
        (labels+history.all.flatMapIndexed { index,it -> listOf(it.displayText(r.service),descriptions[index],formatter.format(Instant.ofEpochMilli(it.creationTimestampMs))) }).toTypedArray())
}
