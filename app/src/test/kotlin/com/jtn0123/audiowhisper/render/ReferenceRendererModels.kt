package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import dev.patrickgold.florisboard.ime.editor.OperationScope
import dev.patrickgold.florisboard.ime.editor.OperationUnit
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.popup.ExceptionsForKeyCodes
import dev.patrickgold.florisboard.lib.Pointer

internal class RefValue<T>(val value: T) { fun get()=value }
internal class RefGesturePrefs(actions: List<SwipeAction>) {
    val swipeUp=RefValue(actions[0]);val swipeDown=RefValue(actions[1]);val swipeLeft=RefValue(actions[2]);val swipeRight=RefValue(actions[3])
    val spaceBarSwipeUp=RefValue(actions[4]);val spaceBarSwipeLeft=RefValue(actions[5]);val spaceBarSwipeRight=RefValue(actions[6])
    val spaceBarLongPress=RefValue(actions[7]);val deleteKeySwipeLeft=RefValue(actions[8])
}
internal class RefPrefs(actions: List<SwipeAction>) {val gestures=RefGesturePrefs(actions)}
internal class RefKey(val index: Int, code: Int) {
    val computedData=TextKeyData(type=KeyType.CHARACTER,code=code,label="")
    val computedDataOnDown get()=computedData
    val left get()=index*100f
    val right get()=left+100f
}
internal class RefPointer: Pointer() {
    var initialKey: RefKey?=null;var activeKey: RefKey?=null
    var hasTriggeredGestureMove=false;var hasTriggeredLongPress=false;var hasTriggeredMassSelection=false
    override fun reset() {super.reset();initialKey=null;activeKey=null;hasTriggeredGestureMove=false;hasTriggeredLongPress=false;hasTriggeredMassSelection=false}
}
internal class RefPopup {
    var isShowingExtendedPopup=false
    fun isSuitableForPopups(key: RefKey)=key.computedData.code.let {it>KeyCode.SPACE && it!=KeyCode.CJK_SPACE || it in ExceptionsForKeyCodes}
    fun getActiveKeyData(key: RefKey): TextKeyData?=key.computedData
    fun hide() {isShowingExtendedPopup=false}
}
internal class RefDispatch(private val owner: ReferenceRendererTouch) {
    private val pressed=mutableSetOf<Int>()
    fun sendDown(data: TextKeyData) {pressed.add(data.code);owner.emit(KEY_DOWN,owner.index(data),owner.currentPointer)}
    fun sendUp(data: TextKeyData) {pressed.remove(data.code);owner.emit(KEY_UP,owner.index(data),owner.currentPointer)}
    fun sendCancel(data: TextKeyData) {pressed.remove(data.code);owner.emit(KEY_CANCEL,owner.index(data),owner.currentPointer)}
    fun cancelCode(code: Int) {pressed.remove(code)}
    fun isPressed(code: Int)=code in pressed
}
internal class RefFeedback(private val owner: ReferenceRendererTouch) {
    fun gestureMovingSwipe(data: TextKeyData)=owner.emit(GESTURE_FEEDBACK,data.code,owner.currentPointer)
}
internal class RefKeyboardManager(private val owner: ReferenceRendererTouch,val isTypingIntoGifSearch: Boolean) {
    fun executeSwipeAction(action: SwipeAction)=owner.emit(SWIPE_ACTION,action.ordinal,0)
    fun handleArrow(code: Int,count: Int)=owner.emit(if(code==KeyCode.ARROW_LEFT) CURSOR_LEFT else CURSOR_RIGHT,count,owner.currentPointer)
}
internal class RefEditor(private val owner: ReferenceRendererTouch,raw: Boolean) {
    class Info(val isRawInputEditor: Boolean)
    class Selection {val isValid=true;val isSelectionMode=true}
    class Content {val selection=Selection()}
    val activeInfo=Info(raw);val activeContent=Content()
    val massSelection=Mass()
    inner class Mass {
        fun begin()=owner.emit(MASS_BEGIN,0,owner.currentPointer)
        fun end()=owner.emit(MASS_END,0,owner.currentPointer)
    }
    fun setSelectionSurrounding(n: Int,unit: OperationUnit,scope: OperationScope)=owner.emit(
        if(unit==OperationUnit.WORDS) SELECT_WORDS else SELECT_CHARACTERS,n,if(scope==OperationScope.AFTER_CURSOR) 1 else 0)
    fun deleteBackwards(unit: OperationUnit) {check(unit==OperationUnit.CHARACTERS);owner.emit(DELETE_SELECTION,0,owner.currentPointer)}
}
