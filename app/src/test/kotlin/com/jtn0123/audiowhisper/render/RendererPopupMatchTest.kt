package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import dev.patrickgold.florisboard.ime.popup.PopupKeys
import dev.patrickgold.florisboard.ime.popup.PopupUiController
import dev.patrickgold.florisboard.ime.popup.ReferencePopupGrid
import dev.patrickgold.florisboard.lib.FlorisRect
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals

class RendererPopupMatchTest {
    @Test fun nativePopupOwnershipSelectionAndExitMatchComposeGridReference() {
        val random=Random(0x3b0304)
        val key=FlorisRect.new(left=100f,top=100f,right=200f,bottom=200f)
        KeyboardRenderer(emptyArray()).use {r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
            r.update(floatArrayOf(400f,400f,40f,80f,72f),arrayOf("characters","unshifted","character"),intArrayOf(),
                floatArrayOf(100f,100f,200f,200f,100f,100f,200f,200f,1f,97f),arrayOf("a","",""),emptyArray())
            for(count in 1..14) for(prioritized in 0..minOf(3,count)) for(landscape in listOf(false,true)) {
                val w=80f*(if(landscape) 1f else 1.1f);val h=72f*(if(landscape) 3f else 2.5f)
                val left=key.left+(key.width-w)/2
                val base=FlorisRect.new(left=left,top=key.bottom-h,right=left+w,bottom=key.bottom)
                val ext=ReferencePopupGrid.extend(key,base,400f,PopupKeys(null,List(prioritized) {it},List(count-prioritized) {it}))!!
                val info=PopupUiController.ExtRenderInfo(emptyList(),base,ext.bounds,ext.anchorLeft,ext.anchorRight,ext.anchorOffset,ext.row0count,ext.row1count)
                repeat(50) {sample ->
                    r.touch(intArrayOf(3,0),longArrayOf(),floatArrayOf())
                    r.touch(intArrayOf(0,0,0),longArrayOf(0,1),floatArrayOf(150f,190f))
                    r.popup(0,Array(count) {"fixture"},IntArray(count) {233+it});r.popupGrid(prioritized,landscape)
                    val x=if(sample%5==0) -1000f else random.nextInt(30,371).toFloat()
                    val y=if(sample%7==0) 390f else random.nextInt(0,241).toFloat()
                    val ui=ReferencePopupGrid.propagateMotionEvent(key,info,x,y)
                    val moved=r.touch(intArrayOf(2,0,0),longArrayOf(0,2),floatArrayOf(x,y+40))
                    val actual=r.touch(intArrayOf(1,0,0),longArrayOf(0,3),floatArrayOf(x,y+40))
                    if(ui==null) {
                        assertContentEquals(intArrayOf(KEY_CANCEL,0,0),moved)
                        assertContentEquals(intArrayOf(),actual)
                    } else {
                        val popupIndex=ext.popupIndices[ui]
                        val flat=if(popupIndex<0) -popupIndex-1 else prioritized+popupIndex
                        assertContentEquals(intArrayOf(),moved)
                        assertContentEquals(intArrayOf(POPUP_SELECT,flat,0),actual,"popup $count/$prioritized/$landscape/$sample")
                    }
                }
            }
        }
    }
}
