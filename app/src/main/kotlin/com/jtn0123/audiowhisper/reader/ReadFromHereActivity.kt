package com.jtn0123.audiowhisper.reader

import android.app.Activity
import android.content.Intent
import android.os.Bundle

/**
 * "Read from here" in the menu over selected text, in any app: reads from the selection to the end
 * of the page. It shows nothing; it hands the selection to the reader and closes, and the reader
 * waits for the app to be back in front before it looks at the screen.
 */
class ReadFromHereActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val selected = intent.getCharSequenceExtra(Intent.EXTRA_PROCESS_TEXT)?.toString()
        if (!selected.isNullOrBlank()) ReadAloud.read(this, ReadAloud.From.Selection(selected))
        finish()
    }
}
