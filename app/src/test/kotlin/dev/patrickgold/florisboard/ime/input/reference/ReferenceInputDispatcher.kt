package dev.patrickgold.florisboard.ime.input.reference

import com.jtn0123.audiowhisper.rust.InputProtocol.*
import dev.patrickgold.florisboard.ime.text.key.KeyCode

/** Original dispatcher decisions with a supplied clock and explicit coroutine wakeups. */
internal class ReferenceInputDispatcher(private val repeatable:IntArray) {
    private val pressed=mutableMapOf<Int,Boolean>()
    private var generation=0L
    private val generations=mutableMapOf<Int,Long>()
    private var lastDown=0 to 0L
    private var lastUp=0 to 0L
    fun call(operation:Int,vararg v:Long):LongArray {
        val code=v[0].toInt()
        return when(operation) {
            DOWN -> if(pressed.containsKey(code)) longArrayOf() else {
                pressed[code]=false
                generation++
                generations[code]=generation
                longArrayOf(EMIT_DOWN.toLong(),v[0],v[1],v[2],generation)
            }
            UP -> if(pressed.containsKey(code)) {
                val blocked=pressed.remove(code)==true
                longArrayOf((if(blocked)EMIT_CANCEL else EMIT_UP).toLong(),v[0])
            } else longArrayOf()
            CANCEL -> if(pressed.remove(code)!=null) longArrayOf(EMIT_CANCEL.toLong(),v[0]) else longArrayOf()
            DOWN_UP -> {pressed.remove(code);longArrayOf(EMIT_DOWN_UP.toLong(),v[0])}
            ACK_DOWN -> {lastDown=code to v[1];longArrayOf()}
            ACK_UP -> {lastUp=code to v[1];longArrayOf()}
            HOLD_RESULT -> if(!pressed.containsKey(code) || generations[code]!=v[3]) longArrayOf() else if(v[1]!=0L) {
                pressed[code]=true;longArrayOf()
            } else if(code in repeatable) longArrayOf(EMIT_REPEAT.toLong(),repeatData(code,v[2]!=0L).toLong(),bit(code==KeyCode.DELETE || code==KeyCode.FORWARD_DELETE)) else longArrayOf()
            REPEAT_RESULT -> {
                val emit=pressed.containsKey(code) && generations[code]==v[2] && v[1]!=0L
                if(emit) pressed[code]=true
                longArrayOf(bit(emit))
            }
            FACTS -> longArrayOf(bit(pressed.containsKey(code)),bit(pressed.isNotEmpty()),
                bit(lastDown.first==code && v[1]-lastDown.second<v[2]),
                bit(lastUp.first==code && v[1]-lastUp.second<v[2]),bit(lastDown.first==code),
                bit(code in repeatable),bit(lastDown.first in repeatable),if(pressed.containsKey(code))generations.getValue(code) else 0)
            DELAYS -> {
                val repeat=repeatData(code,v[3]!=0L)
                val longFactor=when(code) {KeyCode.SPACE,KeyCode.CJK_SPACE,KeyCode.SHIFT->2.5f;KeyCode.LANGUAGE_SWITCH->2.0f;else->1.0f}
                val repeatFactor=when(repeat) {KeyCode.DELETE_WORD,KeyCode.FORWARD_DELETE_WORD,KeyCode.UNDO,KeyCode.REDO->5.0f;else->1.0f}
                longArrayOf((v[1]*longFactor).toLong(),(v[2]*repeatFactor).toLong(),repeat.toLong())
            }
            else -> error("Unexpected reference operation")
        }
    }
    private fun repeatData(code:Int,words:Boolean)=when(code) {
        KeyCode.DELETE->if(words)KeyCode.DELETE_WORD else KeyCode.DELETE
        KeyCode.FORWARD_DELETE->if(words)KeyCode.FORWARD_DELETE_WORD else KeyCode.FORWARD_DELETE
        else->code
    }
    private fun bit(value:Boolean)=if(value)1L else 0L
}
