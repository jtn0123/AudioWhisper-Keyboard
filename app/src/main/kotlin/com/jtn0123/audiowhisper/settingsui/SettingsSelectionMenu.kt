package com.jtn0123.audiowhisper.settingsui

import android.content.ClipData
import android.content.ClipboardManager
import android.view.ActionMode
import android.view.Menu
import android.view.MenuItem
import android.view.View

/** The floating selection toolbar is Android-owned; Rust owns selection and edits. */
internal class SettingsSelectionMenu(private val endpoint: View, private val host: RustSettingsView) : ActionMode.Callback {
    private var mode: ActionMode? = null
    private val clipboard = endpoint.context.getSystemService(ClipboardManager::class.java)
    fun update(requested: Boolean) {
        if (requested && endpoint.hasFocus() && mode == null) mode = endpoint.startActionMode(this, ActionMode.TYPE_FLOATING)
        if (!requested) close()
    }
    fun close() { mode?.finish(); mode = null }
    override fun onCreateActionMode(mode: ActionMode, menu: Menu): Boolean {
        for ((id, label) in listOf(android.R.id.cut to android.R.string.cut, android.R.id.copy to android.R.string.copy,
            android.R.id.paste to android.R.string.paste, android.R.id.selectAll to android.R.string.selectAll)) {
            menu.add(Menu.NONE, id, Menu.NONE, label)
        }
        return true
    }
    override fun onPrepareActionMode(mode: ActionMode, menu: Menu): Boolean {
        menu.findItem(android.R.id.paste).isEnabled = clipboard.hasPrimaryClip()
        return true
    }
    override fun onActionItemClicked(mode: ActionMode, item: MenuItem): Boolean {
        when (item.itemId) {
            android.R.id.copy, android.R.id.cut -> {
                clipboard.setPrimaryClip(ClipData.newPlainText("", host.query("selected", 0)))
                host.edit(if (item.itemId == android.R.id.cut) "cut" else "dismissToolbar")
            }
            android.R.id.paste -> host.edit("commit", clipboard.primaryClip?.getItemAt(0)?.coerceToText(endpoint.context)?.toString().orEmpty(), "1")
            android.R.id.selectAll -> host.edit("selectAll")
            else -> return false
        }
        if (item.itemId != android.R.id.selectAll) mode.finish()
        return true
    }
    override fun onDestroyActionMode(mode: ActionMode) { this.mode = null }
}
