package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.*
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import kotlinx.serialization.json.*
import kotlin.random.Random
import kotlin.test.*

/** The retained Compose editor itself is the independent, test-only Kotlin reference. */
class SettingsEditorMatchTest {
    @Test fun invalidInputConnectionRequestsAreRejectedWithoutChangingEditor() {
        KeyboardRenderer(emptyArray()).use { n ->
            val before = n.settings(SettingsUiOp.EDIT, arrayOf("commit", "fixture", "1"))
            for (args in listOf(arrayOf("delete", "-1", "0"), arrayOf("codepoints", "0", "-1"), arrayOf("select", "-1", "2"))) {
                val reply = n.settings(SettingsUiOp.EDIT, args)
                val state = Json.parseToJsonElement(reply[0]).jsonObject
                assertFalse(state.getValue("accepted").jsonPrimitive.boolean)
                assertTrue(before[1] == reply[1])
                assertEquals(Json.parseToJsonElement(before[0]).jsonObject.getValue("editor"), state.getValue("editor"))
            }
        }
    }
    /**
     * The keyboard's batch edits, as SettingsInputView passes them on: the cursor is reported
     * only once the outermost one ends, endBatchEdit answers whether one is still open, and a
     * closed connection ends those it left open. Opening and closing a batch changes nothing else.
     */
    @Test fun batchEditsHoldBackCursorReportsUntilTheOutermostEnds() {
        KeyboardRenderer(emptyArray()).use { n ->
            fun editor(reply: Array<String>) = Json.parseToJsonElement(reply[0]).jsonObject.getValue("editor").jsonObject
            fun accepted(reply: Array<String>) = Json.parseToJsonElement(reply[0]).jsonObject.getValue("accepted").jsonPrimitive.boolean
            fun report(reply: Array<String>) = editor(reply).getValue("report").jsonPrimitive.boolean
            val idle = n.settings(SettingsUiOp.EDIT, arrayOf("commit", "fixture", "1"))
            assertTrue(report(idle))
            assertTrue(accepted(n.settings(SettingsUiOp.EDIT, arrayOf("begin"))))
            assertFalse(report(n.settings(SettingsUiOp.EDIT, arrayOf("begin"))))
            assertFalse(report(n.settings(SettingsUiOp.EDIT, arrayOf("commit", " text", "1"))))
            val inner = n.settings(SettingsUiOp.EDIT, arrayOf("end"))
            assertTrue(accepted(inner), "one is still open")
            assertFalse(report(inner))
            val outer = n.settings(SettingsUiOp.EDIT, arrayOf("end"))
            assertFalse(accepted(outer), "none is open")
            assertTrue(report(outer))
            assertEquals("fixture text", outer[1])
            n.settings(SettingsUiOp.EDIT, arrayOf("begin")); n.settings(SettingsUiOp.EDIT, arrayOf("begin"))
            assertTrue(report(n.settings(SettingsUiOp.EDIT, arrayOf("close"))))
            assertFalse(accepted(n.settings(SettingsUiOp.EDIT, arrayOf("end"))), "an unmatched end opens nothing")
            val before = editor(n.settings(SettingsUiOp.EDIT, arrayOf("select", "2", "5")))
            n.settings(SettingsUiOp.EDIT, arrayOf("begin"))
            assertEquals(before - "report", editor(n.settings(SettingsUiOp.EDIT, arrayOf("end"))) - "report")
        }
    }
    @Test fun nativeUtf16EditorMatchesComposeEditProcessor() {
        KeyboardRenderer(emptyArray()).use { n -> editorMatches(n, Random(1404), EditProcessor()) }
    }

    /**
     * DictationScreen's vocabulary OutlinedTextField is the same editor: the InputConnection
     * edits it once it is focused, every change is saved to its preference (onValueChange), Enter
     * is a new line, and the preview field keeps its own text across the hand-overs.
     */
    @Test fun vocabularyFieldIsTheSameEditorAndSavesEveryChange() {
        KeyboardRenderer(emptyArray()).use { n ->
            n.settings(SettingsUiOp.PREFERENCES, arrayOf("dictation__vocabulary", "Kubernetes"))
            n.settings(SettingsUiOp.CONFIG, arrayOf("1080", "8192", "1", "1", "4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814", "true", "true"))
            n.settings(SettingsUiOp.EDIT, arrayOf("commit", "preview", "1"))
            assertEquals(listOf(listOf("checkModels")), effects(n.settings(SettingsUiOp.NAVIGATE, arrayOf("settings/dictation"))))
            val field = 1000 + 6
            val opened = Json.parseToJsonElement(n.settings(SettingsUiOp.ACTIVATE, arrayOf(field.toString()))[0]).jsonObject
            assertEquals(listOf(listOf("restartInput"), listOf("editor")), effects(opened))
            assertTrue(opened.getValue("editor").jsonObject.getValue("multiLine").jsonPrimitive.boolean)
            val reference = EditProcessor()
            reference.reset(TextFieldValue("Kubernetes", TextRange(10)), null)
            editorMatches(n, Random(1505), reference) { reply, text ->
                val writes = effects(Json.parseToJsonElement(reply[0]).jsonObject)
                assertTrue(writes.isEmpty() || writes == listOf(listOf("write", "dictation__vocabulary", lossy(text))), "a change is saved as it is")
            }
            val end = n.settings(SettingsUiOp.EDIT, arrayOf("select", "0", "0"))[1].length.toString()
            n.settings(SettingsUiOp.EDIT, arrayOf("select", end, end))
            val before = n.settings(SettingsUiOp.EDIT, arrayOf("query", "before", "65536"))[2]
            n.settings(SettingsUiOp.KEY, arrayOf("0", "66", "10", "false"))
            assertEquals(before + "\n", n.settings(SettingsUiOp.EDIT, arrayOf("query", "before", "65536"))[2], "Enter is a new line")
            // The preview field gets its own editor back, and the vocabulary its saved text.
            val preview = Json.parseToJsonElement(n.settings(SettingsUiOp.ACTIVATE, arrayOf("2"))[0]).jsonObject
            assertEquals(listOf(listOf("restartInput"), listOf("editor")), effects(preview))
            assertFalse(preview.getValue("editor").jsonObject.getValue("multiLine").jsonPrimitive.boolean)
            assertEquals("preview", n.settings(SettingsUiOp.SAVE)[1])
        }
    }
    /**
     * Rust's `String::from_utf16_lossy`, which the saved text goes through: an unpaired
     * surrogate (only a malformed InputConnection sends one) is saved as U+FFFD.
     */
    private fun lossy(s: String) = buildString {
        var i = 0
        while (i < s.length) {
            val pair = s[i].isHighSurrogate() && i + 1 < s.length && s[i + 1].isLowSurrogate()
            if (pair) { append(s, i, i + 2); i += 2 } else { append(if (s[i].isSurrogate()) '\uFFFD' else s[i]); i++ }
        }
    }
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun effects(reply: Array<String>) = effects(Json.parseToJsonElement(reply[0]).jsonObject)

    private fun editorMatches(n: KeyboardRenderer, random: Random, reference: EditProcessor,
                              saved: (Array<String>, String) -> Unit = { _, _ -> }) {
        run {
            repeat(10000) {
                val previous=reference.toTextFieldValue();val length=previous.text.length
                val args:Array<String>; val command:EditCommand
                when(random.nextInt(7)) {
                    0,1 -> {
                        val value=listOf("a","Z"," ","é","e\u0301","😀","👨‍👩‍👧‍👦","\ud800","\udfff","").random(random)
                        val cursor=random.nextInt(-2,4);val composing=random.nextBoolean()
                        args=arrayOf(if(composing)"compose"else"commit",value,cursor.toString())
                        command=if(composing)SetComposingTextCommand(value,cursor)else CommitTextCommand(value,cursor)
                    }
                    2 -> {val a=random.nextInt(length+1);val b=random.nextInt(length+1);args=arrayOf("select",a.toString(),b.toString());command=SetSelectionCommand(a,b)}
                    3 -> {val a=random.nextInt(5);val b=random.nextInt(5);args=arrayOf("delete",a.toString(),b.toString());command=DeleteSurroundingTextCommand(a,b)}
                    4 -> {args=arrayOf("finish");command=FinishComposingTextCommand()}
                    5 -> {val a=random.nextInt(length+1);val b=random.nextInt(length+1);args=arrayOf("region",a.toString(),b.toString());command=SetComposingRegionCommand(a,b)}
                    else -> {val a=random.nextInt(3);val b=random.nextInt(3);args=arrayOf("codepoints",a.toString(),b.toString());command=DeleteSurroundingTextInCodePointsCommand(a,b)}
                }
                val expected=reference.apply(listOf(command));val reply=n.settings(SettingsUiOp.EDIT,args)
                val editor=Json.parseToJsonElement(reply[0]).jsonObject.getValue("editor").jsonObject
                val numbers = if (args[0] in listOf("commit", "compose")) listOf(args[1].length.toString(), args[2]) else args.drop(1)
                assertTrue(expected.text == reply[1], "editor delta step=$it op=${args[0]} args=$numbers beforeUnits=${previous.text.length} beforeSelection=${previous.selection} beforeComposition=${previous.composition} expectedUnits=${expected.text.length} nativeUnits=${reply[1].length} expectedSelection=${expected.selection} nativeAnchor=${editor.getValue("anchor")} nativeFocus=${editor.getValue("focus")} expectedComposition=${expected.composition} nativeComposition=${editor["composing"]}")
                assertEquals(expected.selection.start,editor.getValue("anchor").jsonPrimitive.int,"anchor step=$it op=${args[0]} beforeSelection=${previous.selection} beforeComposition=${previous.composition} argLengths=${args.map { it.length }} expectedSelection=${expected.selection} nativeFocus=${editor.getValue("focus")}")
                assertEquals(expected.selection.end,editor.getValue("focus").jsonPrimitive.int,"focus $it")
                val comp=editor["composing"]?.takeUnless { it==JsonNull }?.jsonArray?.map { it.jsonPrimitive.int }
                assertEquals(expected.composition?.let { listOf(it.start,it.end) },comp,"composition $it")
                saved(reply, expected.text)
            }
        }
    }
}
