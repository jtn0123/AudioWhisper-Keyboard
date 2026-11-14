package com.jtn0123.audiowhisper.dictation

import java.security.MessageDigest
import java.util.IdentityHashMap
import org.json.JSONArray

/** Both owners consume actual decoder results in memory. Evidence is hashes and state only. */
internal class LiveSessionComparison {
    private class Side(reference: Boolean) {
        val flows = DictationFlows()
        val oldFlows = ReferenceDictationFlows()
        private val main = ArrayDeque<Runnable>()
        private val worker = ArrayDeque<Runnable>()
        private val texts = IdentityHashMap<FloatArray, String>()
        private val editorText = StringBuilder()
        private val errors = mutableListOf<Int>()
        private val schedules = mutableListOf<Long>()
        private var now = 1_000L
        private lateinit var callbacks: DictationSession.Callbacks
        private val model = object : DictationSession.SpeechModel {
            override fun transcribe(samples: FloatArray): String = texts[samples] ?: error("Unknown fixture audio")
            override fun close() = Unit
        }
        private val editor = object : Dictation.Editor {
            override fun allowsDictation() = true
            override fun textBeforeCursor(): CharSequence = editorText
            override fun type(text: String, replacing: Int) { editorText.setLength(editorText.length-replacing);editorText.append(text) }
            override fun isName(word: String) = false
            override fun tidy(phrase: String) = phrase
            override fun undoLastPhrase() = false
            override fun problem(message: Int) { errors.add(message) }
            override fun tick() = Unit
        }
        private val platform = object : DictationSession.Platform {
            override fun post(action: Runnable) { main.addLast(action) }
            override fun postAtTime(action: Runnable, uptimeMs: Long) { schedules.add(uptimeMs) }
            override fun removeCallbacks(action: Runnable) { schedules.clear() }
            override fun uptimeMs() = now
            override fun work(action: Runnable) { worker.addLast(action) }
            override fun shutDownWorker() = Unit
            override fun hasMicPermission() = true
            override fun openAppSettings() = Unit
            override fun modelState(): ModelState = ModelState.Ready
            override fun openModelSettings() = Unit
            override fun listen(callbacks: DictationSession.Callbacks): DictationSession.Listening {
                this@Side.callbacks=callbacks
                return object : DictationSession.Listening { override fun stop(keepLastPhrase: Boolean) = Unit }
            }
            override fun loadModel() = model
            override fun log(level: DictationSession.LogLevel, message: String, error: Throwable?) = Unit
        }
        private val old = if (reference) ReferenceDictationSession(editor,platform,oldFlows) else null
        private val native = if (!reference) DictationSession(editor,platform,flows) else null
        init { old?.pressMic();native?.pressMic();drain() }
        private fun drain() { while (true) (main.removeFirstOrNull() ?: worker.removeFirstOrNull() ?: break).run() }
        fun accept(kind: Int, samples: FloatArray, text: String, atSamples: Long) {
            texts[samples]=text;now=1_000L+atSamples*1000/16_000
            when (kind) { 0->callbacks.onLevel(.25f);1->callbacks.onPreview(samples);2->callbacks.onPhrase(samples);3->{old?.stopListening();native?.stopListening()};4->callbacks.onSpeech(true);5->callbacks.onSpeech(false) }
            drain()
        }
        fun snapshot(): String {
            val status=native?.let {flows.status.value} ?: oldFlows.status.value
            val warming=native?.let {flows.warming.value} ?: oldFlows.warming.value
            val level=native?.let {flows.level.value} ?: oldFlows.level.value
            val preview=native?.let {flows.preview.value} ?: oldFlows.preview.value
            fun hash(text:String)=MessageDigest.getInstance("SHA-256").digest(text.toByteArray()).joinToString(""){"%02x".format(it)}
            return "$status|$warming|$level|${hash(preview)}|${hash(editorText.toString())}|$errors|$schedules"
        }
    }
    private val sides=listOf(Side(true),Side(false))
    val states=JSONArray()
    fun accept(kind:Int,samples:FloatArray=FloatArray(0),text:String="",atSamples:Long=0) {
        sides.forEach {it.accept(kind,samples,text,atSamples)}
        check(sides[0].snapshot()==sides[1].snapshot()) {"Real session trace mismatch"}
        states.put(sides[1].snapshot())
    }
}
