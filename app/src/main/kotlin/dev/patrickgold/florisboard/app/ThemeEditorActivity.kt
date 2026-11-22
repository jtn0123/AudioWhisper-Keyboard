package dev.patrickgold.florisboard.app

import android.content.Context
import android.content.Intent
import androidx.core.net.toUri
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentEditor
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionEditor
import dev.patrickgold.florisboard.lib.cache.CacheManager

/** The native theme editor (stage4-theme-editor.md, part 2) over the Compose extension editor that opened it. Its own
 * activity because FlorisAppActivity is singleTask: starting that would clear the fallback activity holding the
 * extension editor, and the workspace with it. Rust draws everything; this only gives the editor a standard task. */
class ThemeEditorActivity : FlorisAppActivity() {
    companion object {
        /** Opens the editor on [component] of [workspace]'s theme extension; false when it is not one of its themes. */
        fun open(context: Context, workspace: CacheManager.ExtEditorWorkspace<*>, component: ThemeExtensionComponentEditor): Boolean {
            val index = (workspace.editor as? ThemeExtensionEditor)?.themes?.indexOf(component) ?: -1
            if (index < 0) return false
            val link = "ui://florisboard/settings/theme/edit/${workspace.uuid}/$index".toUri()
            context.startActivity(Intent(Intent.ACTION_VIEW, link).setClass(context, ThemeEditorActivity::class.java))
            return true
        }
    }
}
