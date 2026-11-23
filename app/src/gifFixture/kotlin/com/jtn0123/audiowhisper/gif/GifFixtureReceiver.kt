package com.jtn0123.audiowhisper.gif

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/**
 * Compiled only with -PgifFixture=true; shell-only (DUMP). `op=on` makes files/gif-fixture the
 * GIF source, `op=off` takes it away. Says nothing back but whether it worked.
 */
class GifFixtureReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        when (intent.getStringExtra("op")) {
            "on" -> GifFixtureSource.open(context.filesDir)?.let { GifSearch.source = it } ?: return fail()
            "off" -> GifSearch.source = null
            else -> return fail()
        }
        resultCode = OK
    }

    private fun fail() {
        resultCode = FAILED
    }

    private companion object {
        const val OK = 1
        const val FAILED = 2
    }
}
