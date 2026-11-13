package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import dev.patrickgold.florisboard.ime.text.gestures.*
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.lib.PointerMap

/** Test-only transcription of 7c0b8183 TextKeyboardLayoutController. Android drawing, coroutine
 * jobs and editor calls become explicit inputs and action traces; swipe policy stays verbatim.
 */
internal class ReferenceRendererTouch(val codes: IntArray,val settings: List<SwipeAction>,val isGlideEnabled: Boolean,
    val density: Float=1f,val raw: Boolean=false,val gif: Boolean=false): SwipeGesture.Listener, GlideTypingGesture.Listener {
    val keys=codes.mapIndexed { i,code -> RefKey(i,code) }
    val prefs=RefPrefs(settings)
    val pointerMap=PointerMap { RefPointer() }
    val popupUiController=RefPopup()
    val inputEventDispatcher=RefDispatch(this)
    val inputFeedbackController: RefFeedback?=RefFeedback(this)
    val keyboardManager=RefKeyboardManager(this,gif)
    val editorInstance=RefEditor(this,raw)
    var currentPointer=0
    private var now=0L
    private val swipe=ReferenceSwipeDetector(this)
    private val glide=ReferenceGlideDetector(32f) {now/1_000_000}
    private val velocity=ReferenceRendererVelocity()
    private val output=mutableListOf<Int>()
    init {glide.registerListener(this)}
    fun index(data: TextKeyData)=keys.indexOfFirst {it.computedData.code==data.code}
    fun emit(type: Int,value: Int,extra: Int) {output.addAll(listOf(type,value,extra))}
    override fun onSwipe(event: SwipeGesture.Event)=handleSwipe(event)
    override fun onGlideAddPoint(point: GlideTypingGesture.Detector.Position) {emit(GLIDE_POINT,point.x.toRawBits(),point.y.toRawBits())}
    override fun onGlideComplete(data: GlideTypingGesture.Detector.PointerData) {emit(GLIDE_COMPLETE,0,0)}
    override fun onGlideCancelled() {emit(GLIDE_CANCEL,0,0)}
    fun longPress(id: Int,popup: Boolean,uninterrupted: Boolean): IntArray {
        output.clear();currentPointer=id
        val pointer=pointerMap.findById(id) ?: return intArrayOf()
        val key=pointer.activeKey ?: return intArrayOf()
        pointer.hasTriggeredLongPress=true
        when(key.computedData.code) {
            KeyCode.SPACE,KeyCode.CJK_SPACE -> {
                val action=prefs.gestures.spaceBarLongPress.get()
                if(action!=SwipeAction.NO_ACTION && action!=SwipeAction.INSERT_SPACE) keyboardManager.executeSwipeAction(action)
                emit(LONG_PRESS_BLOCK,0,id)
            }
            KeyCode.SHIFT -> if(uninterrupted) {emit(CAPS_LOCK,0,id);emit(LONG_PRESS_FEEDBACK,KeyCode.SHIFT,id)}
            KeyCode.LANGUAGE_SWITCH -> {emit(INPUT_METHOD_PICKER,0,id);emit(LONG_PRESS_BLOCK,0,id)}
            else -> if(popup && popupUiController.isSuitableForPopups(key)) {
                emit(OPEN_POPUP,key.index,id);emit(LONG_PRESS_FEEDBACK,key.computedData.code,id);emit(LONG_PRESS_BLOCK,0,id)
            }
        }
        return output.toIntArray()
    }
    fun event(action: Int,index: Int,ids: IntArray,times: LongArray,xy: FloatArray): IntArray {
        output.clear();now=times.last();currentPointer=ids[index]
        swipe.onTouchEvent(action);velocity.add(action,index,ids,times,xy)
        val count=ids.size;val history=times.size-2
        val xs=ids.indices.map { p -> FloatArray(history+1) {h->xy[(h*count+p)*2]} }
        val ys=ids.indices.map { p -> FloatArray(history+1) {h->xy[(h*count+p)*2+1]-20f} }
        val event=ReferenceMotionEvent(action,index,ids,xs,ys)
        if(isGlideEnabled && pointerMap.findById(0)?.hasTriggeredLongPress != true && glide.onTouchEvent(event,pointerMap.findById(0)?.initialKey?.computedData?.code,density)) {
            for(p in pointerMap) if(p.activeKey!=null) cancel(p)
            if(action==1 || action==3) pointerMap.clear()
            return output.toIntArray()
        }
        fun down(p: RefPointer) {
            val x=event.getX(p.index);val y=event.getY(p.index)
            val key=keys.firstOrNull {x>=it.left && x<it.right && y>=0 && y<100}
            if(key!=null) {
                inputEventDispatcher.sendDown(key.computedData)
                if(p.initialKey==null) p.initialKey=key
                p.activeKey=key
            } else p.activeKey=null
        }
        fun up(p: RefPointer) {
            if(p.hasTriggeredMassSelection) {p.hasTriggeredMassSelection=false;editorInstance.massSelection.end()}
            if(p.activeKey!=null) {
                if(p.hasTriggeredGestureMove) inputEventDispatcher.sendCancel(p.activeKey!!.computedDataOnDown)
                else inputEventDispatcher.sendUp(p.activeKey!!.computedDataOnDown)
                p.activeKey=null
            }
            p.hasTriggeredGestureMove=false
        }
        fun moved(p: RefPointer) {
            val key=p.activeKey ?: return
            val x=event.getX(p.index);val y=event.getY(p.index)
            if(x<key.left-10f || x>key.right+10f || y< -35f || y>135f) {cancel(p);down(p)}
        }
        fun swipedUp(p: RefPointer): Boolean {
            val (vx,vy)=velocity.get(p.id)
            return swipe.onTouchUp(p.id,event.getX(p.index),event.getY(p.index),vx,vy,density,32,1900)
        }
        when(action) {
            0 -> pointerMap.add(ids[index],index)?.let {p ->
                swipe.onTouchDown(p.id,p.index,event.getX(p.index),event.getY(p.index),density);down(p)
            }
            5 -> {
                pointerMap.findById(ids[index])?.let {p ->swipe.onTouchCancel(p.id);cancel(p);pointerMap.removeById(p.id)}
                for(p in pointerMap) if(p.activeKey?.let {popupUiController.isSuitableForPopups(it)}==true) {
                    currentPointer=p.id;swipe.onTouchCancel(p.id);up(p)
                }
                currentPointer=ids[index]
                pointerMap.add(ids[index],index)?.let {p ->swipe.onTouchDown(p.id,p.index,event.getX(p.index),event.getY(p.index),density);down(p)}
            }
            2 -> for((i,id) in ids.withIndex()) pointerMap.findById(id)?.let {p ->
                currentPointer=id;p.index=i
                val always=p.hasTriggeredGestureMove && (p.initialKey?.computedData?.code==KeyCode.DELETE &&
                    prefs.gestures.deleteKeySwipeLeft.get() in listOf(SwipeAction.DELETE_CHARACTERS_PRECISELY,SwipeAction.SELECT_CHARACTERS_PRECISELY) ||
                    p.initialKey?.computedData?.code in listOf(KeyCode.SPACE,KeyCode.CJK_SPACE))
                if(swipe.onTouchMove(p.id,p.index,event.getX(i),event.getY(i),density,32,always) || p.hasTriggeredGestureMove) {
                    p.hasTriggeredGestureMove=true;p.activeKey?.let {inputEventDispatcher.sendCancel(it.computedDataOnDown)}
                } else moved(p)
            }
            6 -> pointerMap.findById(ids[index])?.let {p ->
                p.index=index
                if(swipedUp(p) || p.hasTriggeredGestureMove) {
                    if(p.hasTriggeredGestureMove && p.initialKey?.computedData?.code==KeyCode.DELETE) deleteSwipedSelection()
                    cancel(p)
                } else up(p)
                pointerMap.removeById(p.id)
            }
            1 -> {
                for(p in pointerMap) {currentPointer=p.id
                    if(p.id==ids[index]) {p.index=index
                        if(swipedUp(p) || p.hasTriggeredGestureMove) {
                            if(p.hasTriggeredGestureMove && p.initialKey?.computedData?.code==KeyCode.DELETE &&
                                prefs.gestures.deleteKeySwipeLeft.get() !in listOf(SwipeAction.SELECT_CHARACTERS_PRECISELY,SwipeAction.SELECT_WORDS_PRECISELY)) deleteSwipedSelection()
                            cancel(p)
                        } else up(p)
                    } else {swipe.onTouchCancel(p.id);cancel(p)}
                }
                pointerMap.clear()
            }
            3 -> {for(p in pointerMap) {currentPointer=p.id;swipe.onTouchCancel(p.id);cancel(p)};pointerMap.clear()}
        }
        return output.toIntArray()
    }
    private fun cancel(p: RefPointer) {
        currentPointer=p.id
        if(p.hasTriggeredMassSelection) {p.hasTriggeredMassSelection=false;editorInstance.massSelection.end()}
        p.activeKey?.let {inputEventDispatcher.sendCancel(it.computedDataOnDown);popupUiController.hide()}
        p.activeKey=null;p.hasTriggeredGestureMove=false
    }
}
