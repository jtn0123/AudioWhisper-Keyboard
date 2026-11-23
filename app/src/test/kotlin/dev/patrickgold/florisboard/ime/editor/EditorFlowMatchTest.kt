package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.ime.nlp.PunctuationRule
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * Rust's editor events, checked against golden tapes of the frozen reference flow's answers to the
 * same events (docs/phase6-kotlin-references.md).
 */
class EditorFlowMatchTest {
    /**
     * The native editor driven as EditorInstance drives it: it keeps the spacing flags Rust hands
     * back after each event, clears them on each selection update, and gives them back with the
     * settings as the next event's context.
     */
    internal class FlowPair(tape: GoldenTape) : AutoCloseable {
        val pair=EditorPlanMatchTest.Pairing(tape)
        val autoSpace=EditorInstance.AutoSpaceState()
        val phantomSpace=EditorInstance.PhantomSpaceState()
        var autoSpacePunctuation=false
        var immediateWord=true
        init{pair.start()}
        /** EditorInstance.shouldDetermineComposingRegion, for this field. */
        private fun determine()=!pair.new.raw&&!InputAttributes.wrap(pair.new.input).flagTextNoSuggestions&&
            (phantomSpace.isInactive||phantomSpace.showComposingRegion)
        private fun configure() {pair.new.phantom=phantomSpace.isActive;pair.new.determine=determine()}
        fun context() {
            val rule=PunctuationRule.Fallback
            configure()
            pair.new.session.call(EditorOp.CONTEXT,autoSpace.bits.toString(),phantomSpace.bits.toString(),KeyVariation.NORMAL.value.toString(),if(autoSpacePunctuation)"1"else"0",
                "en",rule.symbolsPrecedingAutoSpace,rule.symbolsFollowingAutoSpace,rule.symbolsPrecedingPhantomSpace,rule.symbolsFollowingPhantomSpace,if(immediateWord)"1"else"0")
        }
        fun run(op:EditorOp,vararg args:String) {
            context()
            pair.tape.answer("event result $op",pair.new.run(op,*args))
            pair.compare(op.code)
            val spaces=pair.new.session.call(EditorOp.SPACES).let{s->listOf(EditorSpaces.AUTO_SPACE,EditorSpaces.PHANTOM_SPACE).map{s[it].toInt()}}
            pair.tape.answer("spacing $op",spaces)
            autoSpace.bits=spaces[0];phantomSpace.bits=spaces[1]
        }
        private fun settle() {autoSpace.setInactiveFromUpdate();phantomSpace.setInactiveFromUpdate();configure()}
        /** The selection update the field sends after an event. */
        fun update() {settle();pair.reconcile()}
        /** An edit from outside the keyboard, and its selection update. */
        fun external(text:String,cursor:Int) {settle();pair.external(text,cursor)}
        override fun close()=pair.close()
    }
    @Test fun `autocorrect suggestion phantom word delete and dictation traces match`() {
        GoldenTape.check("editor-flow-public") { tape -> FlowPair(tape).use { p ->
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
            p.autoSpacePunctuation=true
            p.run(EditorOp.TYPE_CHAR,".");p.update();p.run(EditorOp.TYPE_CHAR,",");p.update()
        } }
    }
    @Test fun `25000 seeded editor events match final text state and ordered operations`() {
        val values=listOf("a","e"," ",".","\n","\u0301","😀","👨‍👩‍👧‍👦","אבג","hello")
        var count=0
        var operations=0
        var checkpoints=0
        GoldenTape.check("editor-flow-seeded") { tape -> for(seed in 100 until 200) FlowPair(tape).use { p ->
            val r=Random(20261113+seed)
            p.autoSpacePunctuation=r.nextBoolean()
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
                        p.external(text,r.nextInt(text.length+1))
                    }
                }
                p.update();count++
            }
            operations+=p.pair.operationCount;checkpoints+=p.pair.calls
        } }
        println("editor flows: seeds=100 events=$count checkpoints=$checkpoints InputConnection_ops=$operations differences=0")
        assertEquals(25000,count)
    }
}
