package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Test-only callbacks/state from ClipboardInputLayout, with four opaque synthetic entries. */
class ClipboardMenuMatchTest {
    private fun reference(c: IntArray): IntArray {
        val enabled=c[0]!=0;val locked=c[1]!=0
        var row=c[2]!=0;var filter=c[3];var popup=c[4];var clear=c[5]!=0;var scroll=50
        val op=c[6];val index=c[7];val actions=mutableListOf<Int>()
        fun action(op: Int,index: Int){actions.addAll(listOf(58,op,index))}
        if(op==0) action(0,index)
        else if(!locked) {
            var changed=true
            val modal=popup>=0 || clear
            when {
                op==1->{action(1,if(enabled)0 else 1);changed=false}
                op==2 && enabled && !modal && (filter==0 || filter and 14!=0)->clear=true
                op==3 && enabled && !modal->{row=!row;if(!row)filter=0;scroll=0}
                op==4 && enabled && !modal->{filter=filter xor (1 shl index);scroll=0}
                op==5 && enabled && index<4->{popup=-1;action(5,index);changed=false}
                op==6 && enabled && index<4->popup=index
                op in 7..9 && popup>=0->{action(if(op==7 && popup==0)14 else op,popup);popup=-1;changed=false}
                op==10->{popup=-1;clear=false}
                op==11 && clear->{clear=false;for(i in 1..3)if(filter==0 || filter and (1 shl (i%3+1))!=0) action(12,i);action(13,0);changed=false}
            }
            if(changed)actions.addAll(listOf(57,0,0))
        }
        return (listOf(if(row)1 else 0,filter,popup,if(clear)1 else 0,scroll)+actions).toIntArray()
    }
    @Test fun filtersModalSelectionAndClearPlans() {
        for(enabled in 0..1) for(locked in 0..1) for(row in 0..1) for(filters in 0..7)
            for(popup in -1..3) for(clear in 0..1) for(op in 0..11) for(index in 0..4) {
                val c=intArrayOf(enabled,locked,row,filters,popup,clear,op,index)
                assertContentEquals(reference(c),PanelRules.clipboardMenu(c),"config=${c.joinToString()}")
            }
    }
}
