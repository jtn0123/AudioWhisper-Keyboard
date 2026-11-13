package dev.patrickgold.florisboard.ime.input.reference

import com.jtn0123.audiowhisper.rust.InputProtocol.*

/** Original KeyboardEditing backward/forward branches, recording the Android/provider edge. */
internal object ReferenceInputDelete {
    fun effects(words:Boolean,forward:Boolean,autoUndo:Boolean,dictationUndo:Boolean,composing:Boolean):List<Long> {
        val out=mutableListOf<Long>()
        fun effect(kind:Int,arg:Long=0){out.addAll(listOf(kind.toLong(),arg))}
        if(!forward)effect(DELETE_DROP)
        for(field in listOf(6L,7L,8L))effect(DELETE_STATE,field shl 32)
        if(!forward && !words) {
            effect(DELETE_UNDO_AUTO)
            if(autoUndo)return out
            effect(DELETE_UNDO_DICTATION)
            if(dictationUndo)return out
        }
        effect(DELETE_NOTIFY)
        if(!forward && !words && composing)effect(DELETE_TAP)
        effect(if(forward)DELETE_FORWARD_EDITOR else DELETE_BACKWARD_EDITOR,if(words)1 else 0)
        return out
    }
}
