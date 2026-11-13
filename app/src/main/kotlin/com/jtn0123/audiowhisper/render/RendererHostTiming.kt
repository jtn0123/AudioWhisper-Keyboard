package com.jtn0123.audiowhisper.render

import android.os.SystemClock
import android.util.Log
import android.view.View
import android.view.ViewTreeObserver

/** Opt-in numeric timestamps only; no editor, candidate, clipboard or query payloads. */
internal object RendererHostTiming {
    private const val TAG = "aw-host"
    fun start(): Long = if (Log.isLoggable(TAG, Log.DEBUG)) SystemClock.elapsedRealtimeNanos() else 0L
    fun created(view: View, start: Long, rust: Boolean): View {
        if (start == 0L) return view
        val created = SystemClock.elapsedRealtimeNanos()
        Log.d(TAG, "renderer=${if (rust) 1 else 0} start_ns=$start created_ns=$created")
        view.viewTreeObserver.addOnGlobalLayoutListener(object : ViewTreeObserver.OnGlobalLayoutListener {
            override fun onGlobalLayout() {
                if (view.width <= 0 || view.height <= 0) return
                view.viewTreeObserver.removeOnGlobalLayoutListener(this)
                Log.d(TAG, "renderer=${if (rust) 1 else 0} start_ns=$start layout_ns=${SystemClock.elapsedRealtimeNanos()}")
            }
        })
        return view
    }
}
