package com.jtn0123.audiowhisper.render

import android.app.KeyguardManager
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.outlined.Backspace
import androidx.compose.material.icons.filled.*
import androidx.compose.runtime.*
import androidx.compose.ui.platform.LocalContext
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.jetpref.datastore.model.observeAsState
import org.florisboard.lib.compose.stringRes
import java.time.Instant
import org.florisboard.lib.compose.LocalLocalizedDateTimeFormatter

internal data class RendererClipboard(val entries: List<ClipboardItem>, val config: LongArray, val labels: Array<String>) {
    fun bind(native: KeyboardRenderer) = native.clipboard(config,
        entries.flatMap { listOf(it.id,it.creationTimestampMs,it.type.value.toLong(),if(it.isPinned) 1L else 0L) }.toLongArray(),labels)
}

/** Android storage, resources and date formatting. Native owns filters, grids, menus and scrolling. */
@Composable
internal fun rendererClipboard(): RendererClipboard {
    val context = LocalContext.current
    val manager by context.clipboardManager()
    val prefs by FlorisPreferenceStore
    val history by manager.historyFlow.collectAsState()
    val enabled by prefs.clipboard.historyEnabled.observeAsState()
    val columns by prefs.clipboard.historyNumGridColumns().observeAsState()
    val keyguard = context.getSystemService(KeyguardManager::class.java)
    val formatter = LocalLocalizedDateTimeFormatter.current
    val labels = listOf(stringRes(R.string.clipboard__header_title), "", "",
        stringRes(R.string.clipboard__empty__title), stringRes(R.string.clipboard__empty__message),
        stringRes(R.string.clipboard__disabled__title), stringRes(R.string.clipboard__disabled__message),
        stringRes(R.string.clipboard__disabled__enable_button), stringRes(R.string.clipboard__locked__title),
        stringRes(R.string.clipboard__locked__message), stringRes(R.string.clipboard__group_pinned),
        stringRes(R.string.clipboard__group_recent), stringRes(R.string.clipboard__group_other),
        stringRes(R.string.clip__pin_item),stringRes(R.string.clip__unpin_item),stringRes(R.string.clip__delete_item),
        stringRes(R.string.clip__paste_item),stringRes(R.string.clipboard__confirm_clear_unfiltered_history__message),
        stringRes(R.string.clipboard__confirm_clear_filtered_history__message),stringRes(R.string.action__no),stringRes(R.string.action__yes))
    val icons = listOf(Icons.AutoMirrored.Filled.ArrowBack,Icons.Default.ToggleOn,Icons.Default.ToggleOff,
        Icons.Default.DeleteSweep,Icons.Default.FilterList,Icons.Default.FilterListOff,Icons.AutoMirrored.Outlined.Backspace).map(::iconPath)
    val descriptions=history.all.map { item ->
        val text=item.stringRepresentation()
        val network=dev.patrickgold.florisboard.lib.util.NetworkUtils
        when {
            network.isEmailAddress(text)->stringRes(R.string.clipboard__item_description_email)
            network.isUrl(text)->stringRes(R.string.clipboard__item_description_url)
            network.isPhoneNumber(text)->stringRes(R.string.clipboard__item_description_phone)
            else->""
        }
    }
    return RendererClipboard(history.all,longArrayOf(System.currentTimeMillis(),if(enabled)1 else 0,
        if(keyguard.isDeviceLocked || keyguard.isKeyguardLocked)1 else 0,columns.toLong()),
        (labels+icons+history.all.flatMapIndexed { index,it -> listOf(it.displayText(context),descriptions[index],formatter.format(Instant.ofEpochMilli(it.creationTimestampMs))) }).toTypedArray())
}
