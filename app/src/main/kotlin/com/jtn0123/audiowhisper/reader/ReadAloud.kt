package com.jtn0123.audiowhisper.reader

import android.content.ActivityNotFoundException
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.os.SystemClock
import android.provider.Settings
import android.util.Log
import android.widget.Toast
import dev.patrickgold.florisboard.R
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Reading the screen aloud, as the keyboard and the "Read from here" text action see it. The
 * reading itself is done by [ReadAloudService], which Android only runs once the user turns it on
 * in its accessibility settings; until then, asking for a reading takes them there.
 */
object ReadAloud {
    enum class Status { IDLE, READING }

    /** Where a reading starts. */
    sealed interface From {
        /** The first line on the screen, carrying on to the end of the page. */
        data object Screen : From

        /** The very top of the page, scrolling back to it first if need be. */
        data object Top : From

        /** Where [text], selected in the app, is on the screen. */
        data class Selection(val text: String) : From
    }

    private val mutableStatus = MutableStateFlow(Status.IDLE)
    val status: StateFlow<Status> = mutableStatus.asStateFlow()

    @Volatile
    private var service: ReadAloudService? = null
    private val key = SpeakerKey()

    /** A reading asked for while the service is turned on but not yet running, and when. */
    private class Waiting(val from: From, val at: Long)

    @Volatile
    private var waiting: Waiting? = null

    fun read(context: Context, from: From) {
        service?.let { return it.read(from) }
        if (!isTurnedOn(context)) return askToTurnOn(context)
        waiting = Waiting(from, SystemClock.uptimeMillis())
        Toast.makeText(context, R.string.read_aloud__starting, Toast.LENGTH_SHORT).show()
    }

    fun stop() {
        waiting = null
        service?.stopReading()
    }

    /** The speaker button went down. */
    fun keyDown() {
        if (key.down(status.value == Status.READING) == SpeakerKey.Action.STOP) stop()
    }

    /** The speaker button came up. */
    fun keyUp(context: Context) {
        when (key.up()) {
            SpeakerKey.Action.FROM_SCREEN -> read(context, From.Screen)
            SpeakerKey.Action.FROM_TOP -> read(context, From.Top)
            SpeakerKey.Action.STOP, null -> Unit
        }
    }

    fun keyCancel() = key.cancel()

    internal fun attach(service: ReadAloudService) {
        this.service = service
        val asked = waiting?.takeIf { SystemClock.uptimeMillis() - it.at < WAIT_MS }
        waiting = null
        asked?.let { service.read(it.from) }
    }

    internal fun detach(service: ReadAloudService) {
        if (this.service !== service) return
        this.service = null
        mutableStatus.value = Status.IDLE
    }

    internal fun reading(on: Boolean) {
        mutableStatus.value = if (on) Status.READING else Status.IDLE
    }

    /**
     * Turned on but not running yet: Android binds it a moment after the app is updated, and holds
     * it off while a test harness drives the phone. The reading starts once it is up, rather than
     * sending the user to turn on what is already on.
     */
    private fun isTurnedOn(context: Context): Boolean {
        val component = ComponentName(context, ReadAloudService::class.java)
        val enabled = Settings.Secure.getString(context.contentResolver, Settings.Secure.ENABLED_ACCESSIBILITY_SERVICES)
        return enabled.orEmpty().split(':').any { ComponentName.unflattenFromString(it) == component }
    }

    /**
     * Opens the accessibility settings with the service picked out in the list (its own page is a
     * system-only shortcut). Settings that don't know the highlight hint just open the list.
     */
    private fun askToTurnOn(context: Context) {
        Toast.makeText(context, R.string.read_aloud__turn_on, Toast.LENGTH_LONG).show()
        val component = ComponentName(context, ReadAloudService::class.java).flattenToString()
        val settings = Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)
            .putExtra(FRAGMENT_ARG_KEY, component)
            .putExtra(SHOW_FRAGMENT_ARGS, Bundle().apply { putString(FRAGMENT_ARG_KEY, component) })
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        try {
            context.startActivity(settings)
        } catch (_: ActivityNotFoundException) {
            Log.w(TAG, "No accessibility settings to open")
        }
    }

    private const val TAG = "ReadAloud"

    /** How long a reading waits for the service to come up before it is dropped. */
    private const val WAIT_MS = 5_000L
    private const val FRAGMENT_ARG_KEY = ":settings:fragment_args_key"
    private const val SHOW_FRAGMENT_ARGS = ":settings:show_fragment_args"
}
