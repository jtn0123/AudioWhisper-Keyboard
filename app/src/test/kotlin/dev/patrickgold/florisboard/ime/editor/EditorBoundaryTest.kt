package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.rust.EditorSession
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class EditorBoundaryTest {
    @Test fun `malformed and out of order requests throw without touching the editor`() {
        EditorSession().use { session ->
            val before=session.call(EditorOp.SNAPSHOT).toList()
            // An unknown operation can't be named from Kotlin; editor's Rust tests cover its refusal.
            for((op,args) in listOf(EditorOp.COMMIT to emptyArray(),EditorOp.WORD to arrayOf("0","1"),EditorOp.READ to arrayOf("","",""))) {
                assertFailsWith<IllegalArgumentException>{session.call(op,*args)}
                assertEquals(before==session.call(EditorOp.SNAPSHOT).toList(),true)
            }
        }
    }
    @Test fun `raw editors invalid updates and hardware modifier batches match`() {
        val tape=GoldenTape.ofReference("editor-boundary-raw")
        EditorPlanMatchTest.Pairing(tape).use { p ->
            p.start(raw=true,input=0)
            p.commit("😀");p.char("é");p.delete(false,false);p.delete(true,true)
            for(meta in listOf(0,0x3000,0x12,0x41,0x3053)) {
                p.result("keys",p.old.sendDownUpKeyEvent(67,meta,2),p.new.run(EditorOp.KEYS,"67",meta.toString(),"2"))
                p.compare("hardware key events")
            }
            p.reconcile()
        }
        tape.finish()
    }
    @Test fun `fake EditText replaces composing and selection in UTF16`() {
        val ic=FakeInputConnection()
        ic.external("A😀B",3)
        ic.setComposingRegion(1,3);ic.setComposingText("e\u0301",1)
        assertEquals(listOf("Ae\u0301B",3,3,1,3,0),ic.state())
        ic.commitText("👩‍💻",1)
        assertEquals(listOf("A👩‍💻B",6,6,-1,-1,0),ic.state())
        ic.deleteSurroundingTextInCodePoints(3,0)
        assertEquals("AB",ic.text)
        ic.setSelection(0,2);ic.commitText("אב",1)
        assertEquals(listOf("אב",2,2,-1,-1,0),ic.state())
    }
    @Test fun `full and partial extracted reports invalidate stale same selection windows`() {
        val tape=GoldenTape.ofReference("editor-boundary-extracted")
        EditorPlanMatchTest.Pairing(tape).use { p ->
            p.start("old",3);p.commit("!")
            p.oldIc.external("new!",4);p.newIc.external("new!",4)
            // The old package had no extracted-text callback: its canonical recovery is a fresh read.
            kotlinx.coroutines.runBlocking {p.old.expectedContentQueue.clear()}
            p.old.handleSelectionUpdate(ReferenceEditorRange.Unspecified,p.oldIc.selection,p.oldIc.composing)
            p.new.run(EditorOp.EXTRACTED,"4","4","0","-1","-1","new!","-1","-1")
            p.compare("full extracted")
            p.oldIc.external("next!",5);p.newIc.external("next!",5)
            p.old.handleSelectionUpdate(ReferenceEditorRange.Unspecified,p.oldIc.selection,p.oldIc.composing)
            p.new.run(EditorOp.EXTRACTED,"5","5","0","0","3","next","-1","-1")
            p.compare("partial extracted")
        }
        tape.finish()
    }
    @Test fun `mass selection and arrow plans preserve modifier ordering`() {
        val tape=GoldenTape.ofReference("editor-boundary-mass")
        EditorPlanMatchTest.Pairing(tape).use { p ->
            p.start("alpha beta",4)
            p.new.session.call(EditorOp.MASS_BEGIN)
            assertEquals("1",p.new.session.call(EditorOp.MASS_STATE)[0])
            p.old.handleMassSelectionUpdate(ReferenceEditorRange(1,5),ReferenceEditorRange(0,4))
            p.new.run(EditorOp.UPDATE,"1","5","0","4","1")
            p.compare("mass update")
            assertEquals("1",p.new.session.call(EditorOp.MASS_END)[0])
            val mappings=listOf(-21 to 21,-22 to 22,-23 to 19,-24 to 20,-25 to 19,-26 to 20,-27 to 21,-28 to 22)
            for((code,key) in mappings) for(shift in listOf(false,true)) {
                val meta=(if(code<=-25)0x12 else 0) or (if(shift)0x41 else 0)
                p.old.sendDownUpKeyEvent(key,meta,2)
                p.new.run(EditorOp.ARROW,code.toString(),"2",if(shift)"1"else"0")
                p.compare("arrow")
            }
        }
        tape.finish()
    }
}
