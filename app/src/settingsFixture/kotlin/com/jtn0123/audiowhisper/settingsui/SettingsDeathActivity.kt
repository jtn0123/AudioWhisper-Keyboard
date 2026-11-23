package com.jtn0123.audiowhisper.settingsui

import android.os.Bundle
import android.view.View
import android.view.ViewGroup
import com.jtn0123.audiowhisper.rust.SettingsUiEdit
import dev.patrickgold.florisboard.app.FlorisAppActivity
import org.json.JSONObject
import java.io.File
import java.security.MessageDigest

/** Only compiled with -PsettingsFixture=true. Exercises the real Activity saved-state path. */
class SettingsDeathActivity : FlorisAppActivity() {
    @Volatile var savedForDeath = false
    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        savedForDeath = outState.getStringArray("rust-settings-state") != null
        if (savedForDeath && File(filesDir, "settings-death-expected.json").exists())
            File(filesDir, "settings-death-ready").writeText("ready")
    }
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val expected = File(filesDir, "settings-death-expected.json")
        val arm = File(filesDir, "settings-death-arm")
        if (savedInstanceState == null && arm.exists()) {
            arm.delete()
            prepareOrdinaryDeath(expected)
            return
        }
        if (savedInstanceState == null || !expected.exists()) return
        window.decorView.visibility = View.INVISIBLE
        val original = JSONObject(expected.readText())
        val handler = android.os.Handler(mainLooper)
        var attempts = 0
        val check = object : Runnable {
            override fun run() {
                fun find(v: View): RustSettingsView? {
                    if (v is RustSettingsView) return v
                    if (v is ViewGroup) for (i in 0 until v.childCount) find(v.getChildAt(i))?.let { return it }
                    return null
                }
                val host = find(window.decorView)
                if (host == null && attempts++ < 200) { handler.postDelayed(this, 30); return }
                val state = host?.save()
                val hash = state?.let { MessageDigest.getInstance("SHA-256").digest(it[8].toByteArray()).joinToString("") { byte -> "%02x".format(byte) } }
                val passed = state != null && original.getInt("pid") != android.os.Process.myPid() &&
                    hash == original.getString("sha256") && state[1] == original.getString("route") && state[9] == original.getString("anchor") &&
                    state[10] == original.getString("focus") && state[11] == original.getString("composing_start") && state[12] == original.getString("composing_end")
                File(filesDir, "settings-death-result.json").writeText(JSONObject().put("passed", passed)
                    .put("old_pid", original.getInt("pid")).put("new_pid", android.os.Process.myPid()).put("restored_bundle", true).toString())
                // Clear the synthetic draft immediately. No screenshot/tree may capture it.
                host?.edit(SettingsUiEdit.SET, "")
                host?.edit(SettingsUiEdit.BLUR)
                window.decorView.visibility = View.VISIBLE
            }
        }
        handler.post(check)
    }
    private fun prepareOrdinaryDeath(expected: File) {
        val handler = android.os.Handler(mainLooper)
        var attempts = 0
        val prepare = object : Runnable {
            override fun run() {
                fun find(v: View): RustSettingsView? {
                    if (v is RustSettingsView) return v
                    if (v is ViewGroup) for (i in 0 until v.childCount) find(v.getChildAt(i))?.let { return it }
                    return null
                }
                val host = find(window.decorView)
                if (host == null && attempts++ < 200) { handler.postDelayed(this, 30); return }
                checkNotNull(host)
                window.decorView.visibility = View.INVISIBLE
                host.edit(SettingsUiEdit.FOCUS)
                val endpoint = (0 until host.childCount).map(host::getChildAt).filterIsInstance<SettingsInputView>().single()
                val connection = checkNotNull(endpoint.onCreateInputConnection(android.view.inputmethod.EditorInfo()))
                check(connection.setComposingText("ab\uD83D\uDE80cdef", 1))
                check(connection.setSelection(1, 4))
                val state = host.save()
                val hash = MessageDigest.getInstance("SHA-256").digest(state[8].toByteArray()).joinToString("") { "%02x".format(it) }
                expected.writeText(JSONObject().put("route", state[1]).put("pid", android.os.Process.myPid()).put("sha256", hash)
                    .put("anchor", state[9]).put("focus", state[10]).put("composing_start", state[11]).put("composing_end", state[12]).toString())
                moveTaskToBack(true)
            }
        }
        handler.post(prepare)
    }

}
