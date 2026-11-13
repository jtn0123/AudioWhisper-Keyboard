package dev.patrickgold.florisboard.ime.input.reference

import dev.patrickgold.florisboard.ime.keyboard.*
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.ImeUiMode

/** Original manager, subtype and space branches, with value-only platform replies. */
internal object ReferenceInputControl {
    fun subtype(active:Long,default:Long,ids:List<Long>):LongArray {
        val index=ids.indexOfFirst {it==active}.takeIf{it>=0} ?: if(ids.isEmpty())-1 else 0
        val chosen=ids.getOrNull(index) ?: default
        return longArrayOf(index.toLong(),if(chosen!=active)1 else 0)
    }
    fun cycle(flags:List<Boolean>,direction:Long):LongArray {
        val order=flags.indices.toList().let{if(direction<0)it.asReversed() else it}
        var triggerNextSubtype=false
        var newActiveSubtype = -1
        for(index in order) {
            if(triggerNextSubtype) {triggerNextSubtype=false;newActiveSubtype=index}
            else if(flags[index]) triggerNextSubtype=true
        }
        if(triggerNextSubtype) newActiveSubtype=order.first()
        return longArrayOf(newActiveSubtype.toLong())
    }
    fun space(mode:Long,switch:Boolean,double:Boolean,consecutive:Boolean,autoSpace:Boolean,candidate:Boolean,text:String):LongArray {
        var result=mode
        if(switch) when(mode.toInt()) {5,2,3->result=0}
        if(double && consecutive && ReferenceShift.isDoubleSpacePeriod(text)) return longArrayOf(result,1,0)
        return longArrayOf(result,0,if(!autoSpace && candidate)0 else 1)
    }
    fun state(operation:Int,raw:Long,hide:Boolean,start:Int,end:Int):Long {
        val state=ReferenceKeyboardState.new(raw.toULong())
        when(operation) {
            0->if(state.keyboardMode!=KeyboardMode.CHARACTERS) state.inputShiftState=InputShiftState.UNSHIFTED
            4->{
                if(state.imeUiMode!=ImeUiMode.CLIPBOARD || hide) state.imeUiMode=ImeUiMode.TEXT
                state.isSelectionMode=start>=0 && end>=0 && start!=end
            }
            5->{state.imeUiMode=ImeUiMode.TEXT;state.isActionsOverflowVisible=false;state.isActionsEditorVisible=false}
        }
        return state.rawValue.toLong()
    }
}
