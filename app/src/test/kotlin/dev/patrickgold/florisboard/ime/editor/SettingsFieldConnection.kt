package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.boolean
import kotlinx.serialization.json.int
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

/**
 * The Rust settings screen's text field as the keyboard's InputConnection reaches it: each call
 * goes to the native settings editor as SettingsInputView sends it, and the cursor is reported as
 * RustSettingsView.receive reports it (when the editor's state says to) and as InputMethodManager
 * passes it on (only when it changed).
 */
class SettingsFieldConnection(private val native: KeyboardRenderer) : PlanConnection {
    private val updates = ArrayDeque<IntArray>()
    private var lastUpdate: List<Int>? = null

    override var text = ""
        private set

    private fun edit(vararg input: String): Boolean {
        val reply = native.settings(9, arrayOf(*input))
        text = reply[1]
        val state = Json.parseToJsonElement(reply[0]).jsonObject
        val editor = state.getValue("editor").jsonObject
        if (editor.getValue("report").jsonPrimitive.boolean) {
            val composing = editor["composing"]?.takeUnless { it == JsonNull }?.jsonArray
            val update = listOf(editor.getValue("anchor").jsonPrimitive.int, editor.getValue("focus").jsonPrimitive.int,
                composing?.get(0)?.jsonPrimitive?.int ?: -1, composing?.get(1)?.jsonPrimitive?.int ?: -1)
            if (update != lastUpdate) updates += update.toIntArray()
            lastUpdate = update
        }
        return state.getValue("accepted").jsonPrimitive.boolean
    }

    private fun query(kind: String, length: Int) = native.settings(9, arrayOf("query", kind, length.toString()))[2]

    override fun getTextBeforeCursor(n: Int, flags: Int): String = query("before", n)
    override fun getTextAfterCursor(n: Int, flags: Int): String = query("after", n)
    override fun getSelectedText(flags: Int): String = query("selected", 0)

    override fun apply(op: FakeInputConnection.Operation): Boolean = when (op.name) {
        "begin" -> edit("begin")
        "end" -> edit("end")
        "finish" -> edit("finish")
        "composing" -> edit("region", op.a.toString(), op.b.toString())
        "composingText" -> edit("compose", op.text, op.a.toString())
        "commit" -> edit("commit", op.text, op.a.toString())
        "delete" -> edit("delete", op.a.toString(), op.b.toString())
        "selection" -> edit("select", op.a.toString(), op.b.toString())
        else -> error("Not sent in these flows: ${op.name}")
    }

    override fun cursorUpdates() = updates.toList().also { updates.clear() }
}
