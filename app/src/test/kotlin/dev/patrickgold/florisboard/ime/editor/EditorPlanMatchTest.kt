package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.rust.EditorAttributes
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

/**
 * Rust's editor plans, run against a fake field, checked against golden tapes of the frozen
 * reference editor's answers to the same steps (docs/phase6-kotlin-references.md).
 */
class EditorPlanMatchTest {
    /** The native editor over a fake field; every step's answers go on [tape]. */
    class Pairing(val tape: GoldenTape) : AutoCloseable {
        val newIc=FakeInputConnection()
        val new=NativeEditorDriver(newIc)
        var calls=0
        var operationCount=0
        /** What the editor holds now, as EditorInstance reads it back. */
        val content get()=EditorSnapshot(new.session.call(EditorOp.SNAPSHOT)).content
        fun start(text: String="", cursor: Int=text.length, input: Int=0x4001, raw: Boolean=false) {
            newIc.external(text,cursor)
            new.raw=raw;new.input=input
            new.run(EditorOp.START,cursor.toString(),cursor.toString(),"1")
            compare("start")
        }
        /** The field, the operations that reached it, and the editor's windows, queue and last commit. */
        fun compare(label: String) {
            calls++
            operationCount+=newIc.operations.size
            tape.answer("$label state",newIc.state())
            tape.answer("$label operations",newIc.operations)
            tape.answer("$label window",new.snapshot().take(EditorState.CAPS))
            tape.answer("$label published",new.session.call(EditorOp.PUBLISHED).take(EditorState.CAPS))
            tape.answer("$label last commit",new.snapshot()[EditorState.LAST_COMMIT].toInt())
            tape.answer("$label expected queue",WireFlag.decode(new.snapshot()[EditorState.EXPECTED]))
            newIc.operations.clear()
        }
        fun reconcile() {
            new.run(EditorOp.UPDATE,newIc.selection.start.toString(),newIc.selection.end.toString(),newIc.composingStart.toString(),newIc.composingEnd.toString(),"1")
            compare("reconcile")
            tape.answer("caps",new.snapshot()[EditorState.CAPS].toInt())
        }
        fun external(text: String, cursor: Int) {
            newIc.external(text,cursor);reconcile()
        }
        fun commit(value: String) {
            tape.answer("commit",new.run(EditorOp.COMMIT,value));compare("commit")
        }
        fun char(value: String, delete: Boolean=false, before: Boolean=false, after: Boolean=false) {
            tape.answer("char",new.run(EditorOp.CHAR,value,if(delete)"1"else"0",if(before)"1"else"0",if(after)"1"else"0"));compare("char")
        }
        fun delete(after: Boolean, words: Boolean, n: Int=1) {
            tape.answer("delete",new.run(EditorOp.DELETE,if(after)"1"else"0",if(words)"1"else"0",n.toString()));compare("delete")
        }
        fun select(a: Int,b: Int) {
            tape.answer("select",new.run(EditorOp.SELECT,a.toString(),b.toString()));compare("select")
        }
        fun finalize(value: String) {
            tape.answer("finalize",new.run(EditorOp.FINALIZE,value));compare("finalize")
        }
        fun replace(length: Int,value: String) {
            tape.answer("replace",new.run(EditorOp.REPLACE,length.toString(),value));compare("replace")
        }
        override fun close()=new.close()
    }

    @Test fun `public typing and Unicode traces match exact operation sequences`() {
        GoldenTape.check("editor-plan-public") { tape -> Pairing(tape).use { p ->
            p.start()
            for (c in "qwerasdfzxcv") repeat(25){p.char(c.toString());p.reconcile()}
            repeat(8){p.char("3");p.reconcile()}
            for (sample in listOf("a\u0301","😀","👨‍👩‍👧‍👦","🇺🇸","👍🏽","אבג","مرحبا")) {
                p.external("left $sample right",5+sample.length)
                p.delete(false,false);p.reconcile()
                p.delete(true,true);p.reconcile()
            }
            p.external("teh",3);p.finalize("the");p.reconcile();p.commit(" ");p.reconcile();p.replace(4,"teh");p.reconcile()
            p.external("helo",4);p.finalize("help");p.reconcile()
            p.external("alpha beta",5);p.select(0,5);p.commit("RTL אבג");p.reconcile()
            p.external("a",1);p.char("\u0301");p.reconcile()
        } }
    }
    @Test fun `25000 seeded operations mix windows selection external edits composing and actions`() {
        val samples=listOf("a","b"," ",".","\n","😀","👩‍💻","e\u0301","אב","a\uD800")
        var count=0
        var operations=0
        var checkpoints=0
        GoldenTape.check("editor-plan-seeded") { tape -> for(seed in 0 until 100) Pairing(tape).use { p ->
            val r=Random(20261113+seed);p.start("prefix ")
            repeat(250) {
                val value=samples[r.nextInt(samples.size)]
                when(r.nextInt(12)) {
                    0,1 -> p.commit(value)
                    2,3 -> p.char(value,r.nextBoolean(),r.nextBoolean(),r.nextBoolean())
                    4,5 -> p.delete(r.nextBoolean(),r.nextBoolean(),r.nextInt(1,4))
                    6 -> {val a=r.nextInt(p.newIc.text.length+1);val b=r.nextInt(a,p.newIc.text.length+1);p.select(a,b)}
                    7 -> {val text=samples.shuffled(r).take(5).joinToString("");p.external(text,r.nextInt(text.length+1))}
                    8 -> p.replace(r.nextInt(0,p.content.textBeforeSelection.length+2),value)
                    9 -> if(p.content.composing.isValid)p.finalize(value)else p.commit(value)
                    10 -> {p.new.run(EditorOp.REFRESH);p.compare("refresh")}
                    11 -> {p.new.run(EditorOp.ACTION,r.nextInt(8).toString());p.compare("action")}
                }
                // Leave some expected states pending, then reconcile a whole batch of notifications.
                if(r.nextInt(4)!=0)p.reconcile()
                count++
            }
            operations+=p.operationCount;checkpoints+=p.calls
        } }
        println("editor low-level: seeds=100 events=$count checkpoints=$checkpoints InputConnection_ops=$operations differences=0")
        assertEquals(25000,count)
    }
    @Test fun `cache safe margin queue draining and invalid finalization keep batches balanced`() {
        GoldenTape.check("editor-plan-margins") { tape -> Pairing(tape).use { p ->
            p.start("x".repeat(800),400)
            p.select(800,800);p.reconcile()
            p.start("x".repeat(600),590)
            repeat(5){p.commit("😀")}
            p.reconcile();p.external("external "+"y".repeat(400),300)
            p.start("",0)
            p.commit("word")
            p.newIc.composingStart=-1;p.newIc.composingEnd=-1
            p.reconcile() // A matching selection alone must not accept a mismatched composing region.
            p.start("",0)
            p.finalize("unavailable")
            assertEquals(0,p.newIc.batches)
            p.start("raw",3,raw=true)
            p.finalize("unavailable")
            assertEquals(0,p.newIc.batches)
        } }
    }
    @Test fun `closed handles reject use and close is idempotent`() {
        val session=com.jtn0123.audiowhisper.rust.EditorSession()
        session.close();session.close();assertFailsWith<IllegalStateException>{session.call(EditorOp.SNAPSHOT)}
    }
    @Test fun `input attributes and IME options match all masks and seeded flags`() {
        val r=Random(20261113)
        GoldenTape.check("editor-attributes") { tape -> repeat(50000) { index ->
            val raw=if(index<4096)index else r.nextInt()
            val new=InputAttributes.wrap(raw)
            tape.answer("attributes",listOf(new.type.toInt(),new.variation.toInt(),new.capsMode.toInt(),new.flagNumberDecimal,new.flagNumberSigned,new.flagTextAutoComplete,new.flagTextAutoCorrect,new.flagTextImeMultiLine,new.flagTextMultiLine,new.flagTextNoSuggestions))
            val b=ImeOptions.wrap(raw)
            tape.answer("options",listOf(b.action.toInt(),b.flagForceAscii,b.flagNavigateNext,b.flagNavigatePrevious,b.flagNoAccessoryAction,b.flagNoEnterAction,b.flagNoExtractUi,b.flagNoFullScreen,b.flagNoPersonalizedLearning))
        } }
        for(c in Char.MIN_VALUE..Char.MAX_VALUE) for(mode in 0..7) {
            val value="alpha. $c"
            val raw=mode shl 12
            assertEquals(ReferenceCaps.getCapsMode(value,value.length,raw),EditorAttributes.capsAt(value,value.length,raw),"caps UTF-16 category")
        }
    }
}
