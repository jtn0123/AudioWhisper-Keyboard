package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.text.input.*
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import kotlinx.serialization.json.*
import kotlin.random.Random
import kotlin.test.*

/** The retained Compose editor itself is the independent, test-only Kotlin reference. */
class SettingsEditorMatchTest {
    @Test fun invalidInputConnectionRequestsAreRejectedWithoutChangingEditor() {
        KeyboardRenderer(emptyArray()).use { n ->
            val before = n.settings(9, arrayOf("commit", "fixture", "1"))
            for (args in listOf(arrayOf("delete", "-1", "0"), arrayOf("codepoints", "0", "-1"), arrayOf("select", "-1", "2"))) {
                val reply = n.settings(9, args)
                val state = Json.parseToJsonElement(reply[0]).jsonObject
                assertFalse(state.getValue("accepted").jsonPrimitive.boolean)
                assertTrue(before[1] == reply[1])
                assertEquals(Json.parseToJsonElement(before[0]).jsonObject.getValue("editor"), state.getValue("editor"))
            }
        }
    }
    @Test fun nativeUtf16EditorMatchesComposeEditProcessor() {
        val random=Random(1404)
        val reference=EditProcessor()
        KeyboardRenderer(emptyArray()).use { n ->
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
                val expected=reference.apply(listOf(command));val reply=n.settings(9,args)
                val editor=Json.parseToJsonElement(reply[0]).jsonObject.getValue("editor").jsonObject
                val numbers = if (args[0] in listOf("commit", "compose")) listOf(args[1].length.toString(), args[2]) else args.drop(1)
                assertTrue(expected.text == reply[1], "editor delta step=$it op=${args[0]} args=$numbers beforeUnits=${previous.text.length} beforeSelection=${previous.selection} beforeComposition=${previous.composition} expectedUnits=${expected.text.length} nativeUnits=${reply[1].length} expectedSelection=${expected.selection} nativeAnchor=${editor.getValue("anchor")} nativeFocus=${editor.getValue("focus")} expectedComposition=${expected.composition} nativeComposition=${editor["composing"]}")
                assertEquals(expected.selection.start,editor.getValue("anchor").jsonPrimitive.int,"anchor step=$it op=${args[0]} beforeSelection=${previous.selection} beforeComposition=${previous.composition} argLengths=${args.map { it.length }} expectedSelection=${expected.selection} nativeFocus=${editor.getValue("focus")}")
                assertEquals(expected.selection.end,editor.getValue("focus").jsonPrimitive.int,"focus $it")
                val comp=editor["composing"]?.takeUnless { it==JsonNull }?.jsonArray?.map { it.jsonPrimitive.int }
                assertEquals(expected.composition?.let { listOf(it.start,it.end) },comp,"composition $it")
            }
        }
    }
}
