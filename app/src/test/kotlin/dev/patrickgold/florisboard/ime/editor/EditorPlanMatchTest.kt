package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.rust.EditorAttributes
import kotlinx.coroutines.runBlocking
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class EditorPlanMatchTest {
    class Pairing : AutoCloseable {
        val oldIc=FakeInputConnection()
        val newIc=FakeInputConnection()
        val old=ReferenceAbstractEditorInstance(oldIc)
        val new=NativeEditorDriver(newIc)
        var calls=0
        var operationCount=0
        fun start(text: String="", cursor: Int=text.length, input: Int=0x4001, raw: Boolean=false) {
            oldIc.external(text,cursor);newIc.external(text,cursor)
            old.activeInfo=ReferenceFlorisEditorInfo(ReferenceInputAttributes.wrap(input),ReferenceEditorRange.cursor(cursor),raw)
            new.raw=raw;new.input=input
            old.handleStartInputView(old.activeInfo,false)
            new.run("start",cursor.toString(),cursor.toString(),"1")
            compare("start")
        }
        fun compare(label: String) {
            calls++
            operationCount+=oldIc.operations.size
            // Failure messages deliberately contain no editor text, candidate or operation payload.
            assertEquals(oldIc.state()==newIc.state(),true,"$label final editor state differs")
            assertEquals(oldIc.operations==newIc.operations,true,"$label InputConnection operation sequence differs at $calls")
            val c=old.activeContent
            val expected=listOf(c.text,c.offset,c.localSelection.start,c.localSelection.end,c.localComposing.start,c.localComposing.end,c.localCurrentWord.start,c.localCurrentWord.end).map{it.toString()}
            assertEquals(expected==new.snapshot().take(8),true,"$label expected UTF-16 window differs at $calls")
            val published=old.activeContentFlow.value
            val oldPublished=listOf(published.text,published.offset,published.localSelection.start,published.localSelection.end,published.localComposing.start,published.localComposing.end,published.localCurrentWord.start,published.localCurrentWord.end).map{it.toString()}
            assertEquals(oldPublished==new.session.call("published").take(8),true,"$label published state")
            assertEquals(old.lastCommitPosition.pos,new.snapshot()[9].toInt(),"$label last commit")
            assertEquals(old.expectedContent()!=null,new.snapshot()[10]=="1","$label expected queue")
            oldIc.operations.clear();newIc.operations.clear()
        }
        fun reconcile() {
            old.handleSelectionUpdate(ReferenceEditorRange.Unspecified,oldIc.selection,oldIc.composing)
            new.run("update",newIc.selection.start.toString(),newIc.selection.end.toString(),newIc.composingStart.toString(),newIc.composingEnd.toString(),"1")
            compare("reconcile")
            assertEquals(old.activeCursorCapsMode.toInt(),new.snapshot()[8].toInt(),"caps")
        }
        fun external(text: String, cursor: Int) {
            oldIc.external(text,cursor);newIc.external(text,cursor);reconcile()
        }
        fun commit(value: String) {
            assertEquals(old.commitText(value),new.run("commit",value));compare("commit")
        }
        fun char(value: String, delete: Boolean=false, before: Boolean=false, after: Boolean=false) {
            assertEquals(old.commitChar(value,delete,before,after),new.run("char",value,if(delete)"1"else"0",if(before)"1"else"0",if(after)"1"else"0"));compare("char")
        }
        fun delete(after: Boolean, words: Boolean, n: Int=1) = runBlocking {
            val result=old.deleteAroundCursor(if(words)OperationUnit.WORDS else OperationUnit.CHARACTERS,if(after)OperationScope.AFTER_CURSOR else OperationScope.BEFORE_CURSOR,n)
            assertEquals(result,new.run("delete",if(after)"1"else"0",if(words)"1"else"0",n.toString()));compare("delete")
        }
        fun select(a: Int,b: Int) {
            assertEquals(old.setSelection(ReferenceEditorRange(a,b)),new.run("select",a.toString(),b.toString()));compare("select")
        }
        fun finalize(value: String) {
            assertEquals(old.finalizeComposingText(value),new.run("finalize",value));compare("finalize")
        }
        fun replace(length: Int,value: String) {
            assertEquals(old.replaceTextBeforeCursor(length,value),new.run("replace",length.toString(),value));compare("replace")
        }
        override fun close()=new.close()
    }

    @Test fun `public typing and Unicode traces match exact operation sequences`() {
        Pairing().use { p ->
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
        }
    }
    @Test fun `25000 seeded operations mix windows selection external edits composing and actions`() {
        val samples=listOf("a","b"," ",".","\n","😀","👩‍💻","e\u0301","אב","a\uD800")
        var count=0
        var operations=0
        var checkpoints=0
        for(seed in 0 until 100) Pairing().use { p ->
            val r=Random(20261113+seed);p.start("prefix ")
            repeat(250) {
                val value=samples[r.nextInt(samples.size)]
                when(r.nextInt(12)) {
                    0,1 -> p.commit(value)
                    2,3 -> p.char(value,r.nextBoolean(),r.nextBoolean(),r.nextBoolean())
                    4,5 -> p.delete(r.nextBoolean(),r.nextBoolean(),r.nextInt(1,4))
                    6 -> {val a=r.nextInt(p.oldIc.text.length+1);val b=r.nextInt(a,p.oldIc.text.length+1);p.select(a,b)}
                    7 -> {val text=samples.shuffled(r).take(5).joinToString("");p.external(text,r.nextInt(text.length+1))}
                    8 -> p.replace(r.nextInt(0,p.old.activeContent.textBeforeSelection.length+2),value)
                    9 -> if(p.old.activeContent.composing.isValid)p.finalize(value)else p.commit(value)
                    10 -> {p.old.refreshComposing();p.new.run("refresh");p.compare("refresh")}
                    11 -> {val action=r.nextInt(8);p.oldIc.performEditorAction(action);p.new.run("action",action.toString());p.compare("action")}
                }
                // Leave some expected states pending, then reconcile a whole batch of notifications.
                if(r.nextInt(4)!=0)p.reconcile()
                count++
            }
            operations+=p.operationCount;checkpoints+=p.calls
        }
        println("editor low-level: seeds=100 events=$count checkpoints=$checkpoints InputConnection_ops=$operations differences=0")
        assertEquals(25000,count)
    }
    @Test fun `cache safe margin queue draining and invalid finalization retain legacy behavior`() {
        Pairing().use { p ->
            p.start("x".repeat(800),400)
            p.select(800,800);p.reconcile()
            p.start("x".repeat(600),590)
            repeat(5){p.commit("😀")}
            p.reconcile();p.external("external "+"y".repeat(400),300)
            p.start("",0)
            p.commit("word")
            p.oldIc.composingStart=-1;p.oldIc.composingEnd=-1
            p.newIc.composingStart=-1;p.newIc.composingEnd=-1
            p.reconcile() // A matching selection alone must not accept a mismatched composing region.
            p.start("",0)
            p.finalize("unavailable")
            assertEquals(1,p.newIc.batches) // Documents the pre-existing invalid-finalize batch leak.
        }
    }
    @Test fun `closed handles reject use and close is idempotent`() {
        val session=com.jtn0123.audiowhisper.rust.EditorSession()
        session.close();session.close();assertFailsWith<IllegalStateException>{session.call("snapshot")}
    }
    @Test fun `input attributes and IME options match all masks and seeded flags`() {
        val r=Random(20261113)
        repeat(50000) { index ->
            val raw=if(index<4096)index else r.nextInt()
            val old=ReferenceInputAttributes.wrap(raw);val new=InputAttributes.wrap(raw)
            assertEquals(listOf(old.type.toInt(),old.variation.toInt(),old.capsMode.toInt(),old.flagNumberDecimal,old.flagNumberSigned,old.flagTextAutoComplete,old.flagTextAutoCorrect,old.flagTextImeMultiLine,old.flagTextMultiLine,old.flagTextNoSuggestions),
                listOf(new.type.toInt(),new.variation.toInt(),new.capsMode.toInt(),new.flagNumberDecimal,new.flagNumberSigned,new.flagTextAutoComplete,new.flagTextAutoCorrect,new.flagTextImeMultiLine,new.flagTextMultiLine,new.flagTextNoSuggestions))
            val a=ReferenceImeOptions.wrap(raw);val b=ImeOptions.wrap(raw)
            assertEquals(listOf(a.action.toInt(),a.flagForceAscii,a.flagNavigateNext,a.flagNavigatePrevious,a.flagNoAccessoryAction,a.flagNoEnterAction,a.flagNoExtractUi,a.flagNoFullScreen,a.flagNoPersonalizedLearning),
                listOf(b.action.toInt(),b.flagForceAscii,b.flagNavigateNext,b.flagNavigatePrevious,b.flagNoAccessoryAction,b.flagNoEnterAction,b.flagNoExtractUi,b.flagNoFullScreen,b.flagNoPersonalizedLearning))
        }
        for(c in Char.MIN_VALUE..Char.MAX_VALUE) for(mode in 0..7) {
            val value="alpha. $c"
            val raw=mode shl 12
            assertEquals(ReferenceCaps.getCapsMode(value,value.length,raw),EditorAttributes.capsAt(value,value.length,raw),"caps UTF-16 category")
        }
    }
}
