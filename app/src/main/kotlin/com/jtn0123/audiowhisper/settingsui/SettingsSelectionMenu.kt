package com.jtn0123.audiowhisper.settingsui

import android.content.ClipData
import android.content.ClipboardManager
import android.graphics.Rect
import android.view.ActionMode
import android.view.Menu
import android.view.MenuItem
import android.view.View
import com.jtn0123.audiowhisper.rust.*

/**
 * The floating selection toolbar is Android-owned; Rust owns selection and edits. It serves the
 * editor, or the project licence's read-only text, which offers only Copy and Select all and
 * which Rust places. The two never share a screen.
 */
internal class SettingsSelectionMenu(private val endpoint: View, private val host: RustSettingsView) : ActionMode.Callback2() {
    private var mode: ActionMode? = null
    /** The read-only text's toolbar, where Rust puts it: left, top, right, bottom, then 1 when it offers Select all. */
    private var place: List<Float>? = null
    private val clipboard = endpoint.context.getSystemService(ClipboardManager::class.java)
    fun update(requested: Boolean) {
        if (place != null) return
        if (requested && endpoint.hasFocus() && mode == null) mode = endpoint.startActionMode(this, ActionMode.TYPE_FLOATING)
        if (!requested) close()
    }
    /** The reply's `textToolbar`: null closes the read-only text's toolbar, a place shows or moves it. */
    fun place(next: List<Float>?) {
        val old = place
        place = next
        val m = mode
        when {
            next == null -> if (old != null) close()
            m == null -> mode = host.startActionMode(this, ActionMode.TYPE_FLOATING)
            old != next -> { if (old?.get(4) != next[4]) m.invalidate(); m.invalidateContentRect() }
        }
    }
    fun close() { mode?.finish(); mode = null }
    override fun onCreateActionMode(mode: ActionMode, menu: Menu): Boolean {
        val all = listOf(android.R.id.cut to android.R.string.cut, android.R.id.copy to android.R.string.copy,
            android.R.id.paste to android.R.string.paste, android.R.id.selectAll to android.R.string.selectAll)
        val readOnly = setOf(android.R.id.copy, android.R.id.selectAll)
        for ((id, label) in all) if (place == null || id in readOnly) menu.add(Menu.NONE, id, Menu.NONE, label)
        return true
    }
    override fun onPrepareActionMode(mode: ActionMode, menu: Menu): Boolean {
        val p = place
        if (p == null) menu.findItem(android.R.id.paste).isEnabled = clipboard.hasPrimaryClip()
        else menu.findItem(android.R.id.selectAll).isVisible = p[4] == 1f
        return true
    }
    override fun onActionItemClicked(mode: ActionMode, item: MenuItem): Boolean {
        if (place != null) return readOnly(item.itemId)
        when (item.itemId) {
            android.R.id.copy, android.R.id.cut -> {
                clipboard.setPrimaryClip(ClipData.newPlainText("", host.query(SettingsUiQuery.SELECTED, 0)))
                host.edit(if (item.itemId == android.R.id.cut) SettingsUiEdit.CUT else SettingsUiEdit.DISMISS_TOOLBAR)
            }
            android.R.id.paste -> host.edit(SettingsUiEdit.COMMIT, clipboard.primaryClip?.getItemAt(0)?.coerceToText(endpoint.context)?.toString().orEmpty(), "1")
            android.R.id.selectAll -> host.edit(SettingsUiEdit.SELECT_ALL)
            else -> return false
        }
        if (item.itemId != android.R.id.selectAll) mode.finish()
        return true
    }
    /** Rust copies the read-only selection with a clip effect and clears it, or selects it all. */
    private fun readOnly(id: Int): Boolean {
        host.send(SettingsUiOp.SELECTION, when (id) {
            android.R.id.copy -> SettingsUiSelection.COPY
            android.R.id.selectAll -> SettingsUiSelection.SELECT_ALL
            else -> return false
        })
        return true
    }
    override fun onDestroyActionMode(mode: ActionMode) { if (this.mode === mode) this.mode = null }
    override fun onGetContentRect(mode: ActionMode, view: View, outRect: Rect) {
        val p = place ?: return super.onGetContentRect(mode, view, outRect)
        outRect.set(p[0].toInt(), p[1].toInt(), p[2].toInt(), p[3].toInt())
    }
}
