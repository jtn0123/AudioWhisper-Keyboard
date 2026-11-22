package dev.patrickgold.florisboard.app

import android.content.Context
import android.content.Intent
import androidx.core.net.toUri

/** The native theme editor (stage4-theme-editor.md, part 2) over the extension editor that opened it. Its own activity
 * because FlorisAppActivity is singleTask: a theme editor link there would navigate the settings holding the extension
 * editor, so the two keep apart back stacks and saved states. Rust draws everything; this only gives it a standard task. */
class ThemeEditorActivity : FlorisAppActivity() {
    companion object {
        /** Opens the editor on component [index] of the theme extension in editor workspace [uuid]. */
        fun open(context: Context, uuid: String, index: String) {
            val link = "ui://florisboard/settings/theme/edit/$uuid/$index".toUri()
            context.startActivity(Intent(Intent.ACTION_VIEW, link).setClass(context, ThemeEditorActivity::class.java))
        }
    }
}
