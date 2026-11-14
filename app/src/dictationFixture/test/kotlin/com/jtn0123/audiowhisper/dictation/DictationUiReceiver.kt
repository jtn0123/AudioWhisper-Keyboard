package com.jtn0123.audiowhisper.dictation

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.widget.Toast
import java.io.File
import java.nio.file.Files
import kotlinx.coroutines.flow.MutableStateFlow
import org.json.JSONObject

/** Compiled only with -PdictationFixture=true; shell-only, no microphone/model/audio executor. */
class DictationUiReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val op = intent.getStringExtra("op") ?: return
        active?.release(); active = null
        if (op == "reset") { publish(DictationFlows()); return }
        if (op !in setOf("listening", "processing", "no-model", "no-permission")) return
        val flows = DictationFlows()
        val editor = object : Dictation.Editor {
            override fun allowsDictation() = true
            override fun textBeforeCursor() = ""
            override fun type(text: String, replacing: Int) = error("UI fixture cannot type")
            override fun isName(word: String) = false
            override fun tidy(phrase: String) = phrase
            override fun undoLastPhrase() = false
            override fun problem(message: Int) { problem = message; Toast.makeText(context, message, Toast.LENGTH_LONG).show() }
            override fun tick() = Unit
        }
        val platform = object : DictationSession.Platform {
            override fun post(action: Runnable) { action.run() }
            override fun postAtTime(action: Runnable, uptimeMs: Long) = Unit
            override fun removeCallbacks(action: Runnable) = Unit
            override fun uptimeMs() = android.os.SystemClock.uptimeMillis()
            override fun work(action: Runnable) = Unit // no model work or audio can run
            override fun shutDownWorker() = Unit
            override fun hasMicPermission() = context.checkSelfPermission(android.Manifest.permission.RECORD_AUDIO) == android.content.pm.PackageManager.PERMISSION_GRANTED
            override fun openAppSettings() = Unit
            override fun modelState(): ModelState = ModelFiles.quickState(context)
            override fun openModelSettings() = Unit
            override fun listen(callbacks: DictationSession.Callbacks) = object : DictationSession.Listening {
                override fun stop(keepLastPhrase: Boolean) = Unit
            }
            override fun loadModel(): DictationSession.SpeechModel = error("UI fixture cannot load a model")
            override fun log(level: DictationSession.LogLevel, message: String, error: Throwable?) = Unit
        }
        if (op == "no-model") check(platform.modelState() is ModelState.Missing)
        if (op == "no-permission") check(!platform.hasMicPermission())
        problem = 0
        val session = DictationSession(editor, platform, flows).also { active = it }
        session.pressMic()
        if (op == "processing") session.stopListening()
        publish(flows)
        val pending = File(context.filesDir, "dictation-ui.json.tmp")
        Files.write(pending.toPath(), JSONObject().put("state", op).put("status", flows.status.value.name)
            .put("problem", problem).toString().toByteArray())
        check(pending.renameTo(File(context.filesDir, "dictation-ui.json")))
    }
    companion object {
        private var active: DictationSession? = null
        private var problem = 0
        private fun set(flow: Any, value: Any) {
            val delegate = flow.javaClass.declaredFields.map { field -> field.isAccessible = true; field.get(flow) }
                .single { it is MutableStateFlow<*> }
            MutableStateFlow::class.java.getMethod("setValue", Any::class.java).invoke(delegate, value)
        }
        private fun publish(flows: DictationFlows) {
            set(Dictation.status, flows.status.value); set(Dictation.warming, flows.warming.value)
            set(Dictation.level, flows.level.value); set(Dictation.preview, "")
        }
    }
}
