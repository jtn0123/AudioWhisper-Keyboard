package dev.patrickgold.florisboard.ime.input.reference

import com.jtn0123.audiowhisper.rust.InputProtocol.*
import dev.patrickgold.florisboard.ime.keyboard.*
import dev.patrickgold.florisboard.ime.input.*
import dev.patrickgold.florisboard.ime.editor.OperationUnit
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.text.key.KeyCode

/** Platform facts/effects for the frozen routing source, without Android runtime dependencies. */
internal class ReferenceRoutingManager(val facts: LongArray) {
    val actions=mutableListOf<Long>()
    val activeState=State()
    val inputEventDispatcher=Facts()
    val alphabetic get()=facts[11]!=0L
    val isTypingIntoGifSearch get()=facts[10]>=0
    fun record(action:Int,arg:Long=0) {
        val value=when(action) {
            ONE_HANDED->((1-facts[13]) shl 32) or (arg and 0xffffffffL)
            SMARTBAR->1-facts[14]
            INCOGNITO->{
                val next=!activeState.reference.isIncognitoMode
                activeState.reference.isIncognitoMode=next
                (1-facts[12]) or (if(next)2L else 0L)
            }
            else->arg
        }
        actions.addAll(listOf(action.toLong(),value,0))
    }
    fun heldByGifSearch(data:KeyData)=isTypingIntoGifSearch && facts[10]!=1L
    fun typeIntoGifSearch(data:KeyData):Boolean {
        if(facts[10]!=1L) {
            if(facts[10]==0L) when(data.code) {
                KeyCode.DELETE->record(GIF_DELETE_KEY)
                KeyCode.DELETE_WORD->record(GIF_DELETE_WORD_KEY)
                KeyCode.SPACE,KeyCode.CJK_SPACE->record(GIF_SPACE)
                KeyCode.ENTER->{record(GIF_SUBMIT);activeState.imeUiMode=ImeUiMode.MEDIA}
                else->{record(GIF_TYPE);activeState.inputShiftState=keyTyped()}
            }
            return true
        }
        if(data.code==KeyCode.IME_UI_MODE_MEDIA || data.code==KeyCode.IME_UI_MODE_CLIPBOARD) record(GIF_STOP)
        return false
    }
    fun handleArrow(code:Int)=record(ARROW,code.toLong())
    fun handleShiftDown(data:KeyData) {
        val ref=ReferenceShift(activeState.inputShiftState)
        ref.handleShiftDown(CapitalizationBehavior.entries[facts[8].toInt()],facts[6]!=0L)
        activeState.inputShiftState=ref.inputShiftState
    }
    fun handleShiftUp(data:KeyData) {
        if(activeState.inputShiftState!=InputShiftState.CAPS_LOCK && facts[5]==0L && facts[7]==0L)
            activeState.inputShiftState=InputShiftState.UNSHIFTED
    }
    fun handleShiftCancel() {activeState.inputShiftState=InputShiftState.UNSHIFTED}
    fun handleCapsLock() {activeState.inputShiftState=InputShiftState.CAPS_LOCK}
    fun keyTyped():InputShiftState {
        val ref=ReferenceShift(activeState.inputShiftState)
        ref.afterKeyTyped(facts[4]!=0L)
        return ref.inputShiftState
    }
    fun handleCharWidthSwitch() {activeState.isCharHalfWidth=!activeState.isCharHalfWidth}
    fun handleCharWidthFull() {activeState.isCharHalfWidth=false}
    fun handleCharWidthHalf() {activeState.isCharHalfWidth=true}
    fun handleClipboardSelect()=record(SELECT)
    fun handleBackwardDelete(unit:OperationUnit)=record(if(facts[4]!=0L) DELETE_FORWARD else DELETE_BACK,unit.ordinal.toLong())
    fun handleForwardDelete(unit:OperationUnit)=record(DELETE_FORWARD,unit.ordinal.toLong())
    fun handleEnter()=record(ENTER_KEY)
    fun handleLanguageSwitch() {if(facts[9]==1L || facts[9]==3L) record(SUBTYPE,1) else record(NEXT_IME)}
    fun handleSpace(data:KeyData)=record(SPACE_KEY)
    fun onInputKeyUp(data:KeyData)=handleInputKeyUp(data)
    inner class Facts {fun isPressed(code:Int)=facts[4]!=0L}
    inner class State {
        val reference=ReferenceKeyboardState.new(facts[3].toULong())
        fun state(field:Int,value:Int)=record(STATE,(field.toLong() shl 32) or value.toLong())
        var keyboardMode:KeyboardMode
            get()=reference.keyboardMode
            set(v) {reference.keyboardMode=v;state(0,v.toInt())}
        var imeUiMode:ImeUiMode
            get()=reference.imeUiMode
            set(v) {reference.imeUiMode=v;state(3,v.toInt())}
        var inputShiftState:InputShiftState
            get()=reference.inputShiftState
            set(v) {reference.inputShiftState=v;state(2,v.toInt())}
        var isCharHalfWidth:Boolean
            get()=reference.isCharHalfWidth
            set(v) {reference.isCharHalfWidth=v;state(14,if(v)1 else 0)}
        var isActionsOverflowVisible:Boolean
            get()=reference.isActionsOverflowVisible
            set(v) {reference.isActionsOverflowVisible=v;state(10,if(v)1 else 0)}
        var isActionsEditorVisible:Boolean
            get()=reference.isActionsEditorVisible
            set(v) {reference.isActionsEditorVisible=v;state(11,if(v)1 else 0)}
        var isSubtypeSelectionVisible:Boolean
            get()=reference.isSubtypeSelectionVisible
            set(v) {reference.isSubtypeSelectionVisible=v;state(12,if(v)1 else 0)}
        inline fun batchEdit(block:(State)->Unit)=block(this)
    }
    fun run(data:KeyData):LongArray {
        when(facts[0].toInt()) {
            PHASE_DOWN->handleInputKeyDown(data); PHASE_UP->handleInputKeyUp(data)
            PHASE_CANCEL->handleInputKeyCancel(data); PHASE_REPEAT->handleInputKeyRepeat(data)
        }
        return actions.toLongArray()
    }
}
