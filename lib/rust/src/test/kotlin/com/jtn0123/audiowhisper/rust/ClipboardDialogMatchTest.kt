package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Frozen Compose column + trailing ButtonDefaults row placement, using measured text sizes. */
class ClipboardDialogMatchTest {
    private fun reference(c: FloatArray): FloatArray {
        val w=minOf(c[0],c[3]);val row=maxOf(c[15],c[17])+c[10]+c[12]
        val h=c[6]+c[13]+row+c[8]
        val x=c[1]+(c[3]-w)/2;val y=c[2]+(c[4]-h)/2
        val left=x+c[5];val right=x+w-c[7];val top=y+c[6]
        val bx=right-c[11]-c[14]-c[16];val by=top+c[13]+c[10]
        return floatArrayOf(x,y,x+w,y+h,left,top,right,top+c[13],bx,by,bx+c[14],by+c[15],
            bx+c[14],by,bx+c[14]+c[16],by+c[17])
    }
    @Test fun naturalConfirmationHeightAndTrailingButtons() {
        for(d in listOf(.75f,1f,2f,2.625f,3f)) for(width in listOf(160f,240f,360f))
            for(message in listOf(40f,100f,180f)) for(root in listOf(0f,8f)) for(row in listOf(0f,4f,12f)) {
                val c=floatArrayOf(240f,10f,48f,width,300f,root,root,root,root,row,row,row,row,
                    message,66f,34f,70f,36f).map {it*d}.toFloatArray()
                assertContentEquals(reference(c),PanelRules.clipboardDialog(c))
            }
    }
}
