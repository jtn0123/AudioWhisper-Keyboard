package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Compose's 48dp minimum target, later-sibling occlusion and viewport intersection. Test only. */
class ClipboardTouchMatchTest {
    private fun reference(c: FloatArray): FloatArray {
        val min=48*c[0]
        val x=c[2]+c[4]/2;val y=c[3]+c[5]/2
        var left=x-maxOf(min,c[4])/2;var right=x+maxOf(min,c[4])/2
        var top=y-maxOf(min,c[5])/2;var bottom=y+maxOf(min,c[5])/2
        if(c[1]!=0f) {
            val next=c[6]+c[8]/2-maxOf(min,c[8])/2
            if(next>left)right=minOf(right,next)
        }else {
            val next=c[7]+c[9]/2-maxOf(min,c[9])/2
            if(c[6]<right && c[6]+c[8]>left && next>top)bottom=minOf(bottom,next)
        }
        left=maxOf(left,c[10]);top=maxOf(top,c[11]);right=minOf(right,c[12]);bottom=minOf(bottom,c[13])
        return floatArrayOf(left,top,right,bottom)
    }
    @Test fun minimumTouchAndClippedSiblingTargets() {
        for(d in listOf(.75f,1f,2f,3f)) for(kind in 0..1) for(w in listOf(20f,40f,80f,160f))
            for(h in listOf(20f,30f,48f,100f)) for(offset in listOf(0f,30f,60f,200f)) {
                val c=floatArrayOf(d,kind.toFloat(),10f,20f,w,h,if(kind==1)10f+offset else 10f,if(kind==0)20f+offset else 20f,w,h,0f,0f,800f,600f)
                assertContentEquals(reference(c),PanelRules.clipboardTargets(c))
            }
    }
}
