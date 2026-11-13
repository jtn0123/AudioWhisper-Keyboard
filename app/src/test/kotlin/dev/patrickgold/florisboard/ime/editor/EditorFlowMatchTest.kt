package dev.patrickgold.florisboard.ime.editor

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class EditorFlowMatchTest {
    private class FlowPair : AutoCloseable {
        val pair=EditorPlanMatchTest.Pairing()
        val flow=ReferenceEditorFlow(pair.old)
        init{pair.start()}
        fun context() {
            val s=flow.spacing
            val rule=flow.punctuationRule
            pair.new.phantom=s.phantomSpace.isActive
            pair.new.determine=pair.old.determine
            pair.new.session.call("context",s.autoSpace.raw.toString(),s.phantomSpace.raw.toString(),flow.keyVariation.value.toString(),if(flow.autoSpacePunctuation)"1"else"0",
                "en",rule.symbolsPrecedingAutoSpace,rule.symbolsFollowingAutoSpace,rule.symbolsPrecedingPhantomSpace,rule.symbolsFollowingPhantomSpace,if(flow.immediateWord)"1"else"0")
        }
        fun run(op:String,vararg args:String) {
            context()
            val a=flow.run(op,*args)
            val b=pair.new.run(op,*args)
            assertEquals(a,b,"event result $op")
            pair.compare(op)
            assertEquals(listOf(flow.spacing.autoSpace.raw,flow.spacing.phantomSpace.raw),pair.new.session.call("spaces").take(2).map(String::toInt),"spacing $op")
        }
        fun update() {
            flow.update();pair.new.phantom=flow.spacing.phantomSpace.isActive;pair.new.determine=pair.old.determine
            pair.reconcile()
        }
        override fun close()=pair.close()
    }
    @Test fun `autocorrect suggestion phantom word delete and dictation traces match`() {
        FlowPair().use { p ->
            for(c in "teh") {p.run("typeChar",c.toString());p.update()}
            p.run("completion","the");p.update();p.run("typeText"," ");p.update()
            p.run("revert","4","teh","1");p.update();p.run("typeChar","y");p.update()
            p.run("gesture","world");p.update();p.run("backward","0");p.update()
            p.run("dictation","Hello family.","0");p.update();p.run("undoDictation");p.update()
            p.run("dictation","Hello family.","0");p.update();p.run("typeChar","a");p.update()
            p.run("typeText","a ");p.update();p.run("doubleSpace");p.update()
            p.run("typeText","more 👨‍👩‍👧‍👦 words");p.update()
            p.run("selectAround","2","0","0");p.update()
            p.run("selectAround","1","1","1");p.update()
            p.run("selectAround","0","0","0");p.update()
            p.flow.autoSpacePunctuation=true
            p.run("typeChar",".");p.update();p.run("typeChar",",");p.update()
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
                    0,1,2 -> p.run("typeChar",value)
                    3 -> p.run("typeText",value)
                    4 -> p.run("completion",value)
                    5 -> p.run("gesture",value)
                    6 -> p.run("backward",if(r.nextBoolean())"1"else"0")
                    7 -> p.run("forward",if(r.nextBoolean())"1"else"0")
                    8 -> p.run("dictation",value,r.nextInt(3).toString())
                    9 -> p.run("undoDictation")
                    10 -> p.run("enter")
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
