package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.rust.*
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class EditorFlowMatchTest {
    internal class FlowPair : AutoCloseable {
        val pair=EditorPlanMatchTest.Pairing()
        val flow=ReferenceEditorFlow(pair.old)
        init{pair.start()}
        fun context() {
            val s=flow.spacing
            val rule=flow.punctuationRule
            pair.new.phantom=s.phantomSpace.isActive
            pair.new.determine=pair.old.determine
            pair.new.session.call(EditorOp.CONTEXT,s.autoSpace.raw.toString(),s.phantomSpace.raw.toString(),flow.keyVariation.value.toString(),if(flow.autoSpacePunctuation)"1"else"0",
                "en",rule.symbolsPrecedingAutoSpace,rule.symbolsFollowingAutoSpace,rule.symbolsPrecedingPhantomSpace,rule.symbolsFollowingPhantomSpace,if(flow.immediateWord)"1"else"0")
        }
        fun run(op:EditorOp,vararg args:String) {
            context()
            val a=flow.run(op.code,*args)
            val b=pair.new.run(op,*args)
            assertEquals(a,b,"event result $op")
            pair.compare(op.code)
            assertEquals(listOf(flow.spacing.autoSpace.raw,flow.spacing.phantomSpace.raw),pair.new.session.call(EditorOp.SPACES).let{s->listOf(EditorSpaces.AUTO_SPACE,EditorSpaces.PHANTOM_SPACE).map{s[it].toInt()}},"spacing $op")
        }
        fun update() {
            flow.update();pair.new.phantom=flow.spacing.phantomSpace.isActive;pair.new.determine=pair.old.determine
            pair.reconcile()
        }
        override fun close()=pair.close()
    }
    @Test fun `autocorrect suggestion phantom word delete and dictation traces match`() {
        FlowPair().use { p ->
            for(c in "teh") {p.run(EditorOp.TYPE_CHAR,c.toString());p.update()}
            p.run(EditorOp.COMPLETION,"the");p.update();p.run(EditorOp.TYPE_TEXT," ");p.update()
            p.run(EditorOp.REVERT,"4","teh","1");p.update();p.run(EditorOp.TYPE_CHAR,"y");p.update()
            p.run(EditorOp.GESTURE,"world");p.update();p.run(EditorOp.BACKWARD,"0");p.update()
            p.run(EditorOp.DICTATION,"Hello family.","0");p.update();p.run(EditorOp.UNDO_DICTATION);p.update()
            p.run(EditorOp.DICTATION,"Hello family.","0");p.update();p.run(EditorOp.TYPE_CHAR,"a");p.update()
            p.run(EditorOp.TYPE_TEXT,"a ");p.update();p.run(EditorOp.DOUBLE_SPACE);p.update()
            p.run(EditorOp.TYPE_TEXT,"more 👨‍👩‍👧‍👦 words");p.update()
            p.run(EditorOp.SELECT_AROUND,"2","0","0");p.update()
            p.run(EditorOp.SELECT_AROUND,"1","1","1");p.update()
            p.run(EditorOp.SELECT_AROUND,"0","0","0");p.update()
            p.flow.autoSpacePunctuation=true
            p.run(EditorOp.TYPE_CHAR,".");p.update();p.run(EditorOp.TYPE_CHAR,",");p.update()
        }
    }
    @Test fun `25000 seeded editor events match final text state and ordered operations`() {
        val values=listOf("a","e"," ",".","\n","\u0301","😀","👨‍👩‍👧‍👦","אבג","hello")
        var count=0
        var operations=0
        var checkpoints=0
        for(seed in 100 until 200) FlowPair().use { p ->
            val r=Random(20261113+seed)
            p.flow.autoSpacePunctuation=r.nextBoolean()
            repeat(250) {
                val value=values[r.nextInt(values.size)]
                when(r.nextInt(12)) {
                    0,1,2 -> p.run(EditorOp.TYPE_CHAR,value)
                    3 -> p.run(EditorOp.TYPE_TEXT,value)
                    4 -> p.run(EditorOp.COMPLETION,value)
                    5 -> p.run(EditorOp.GESTURE,value)
                    6 -> p.run(EditorOp.BACKWARD,if(r.nextBoolean())"1"else"0")
                    7 -> p.run(EditorOp.FORWARD,if(r.nextBoolean())"1"else"0")
                    8 -> p.run(EditorOp.DICTATION,value,r.nextInt(3).toString())
                    9 -> p.run(EditorOp.UNDO_DICTATION)
                    10 -> p.run(EditorOp.ENTER)
                    11 -> {
                        val text=values.shuffled(r).take(4).joinToString("")
                        val pos=r.nextInt(text.length+1)
                        p.flow.update();p.pair.new.phantom=p.flow.spacing.phantomSpace.isActive;p.pair.new.determine=p.pair.old.determine
                        p.pair.external(text,pos)
                    }
                }
                p.update();count++
            }
            operations+=p.pair.operationCount;checkpoints+=p.pair.calls
        }
        println("editor flows: seeds=100 events=$count checkpoints=$checkpoints InputConnection_ops=$operations differences=0")
        assertEquals(25000,count)
    }
}
