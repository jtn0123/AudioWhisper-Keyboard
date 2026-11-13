package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Full JNI packets versus the frozen Compose controller, swipe policy and Kotlin detectors. */
class RendererTouchMatchTest {
    private val codes=intArrayOf(97,101,KeyCode.SPACE,KeyCode.DELETE,KeyCode.SHIFT,
        KeyCode.LANGUAGE_SWITCH,KeyCode.ARROW_LEFT,KeyCode.ENTER,KeyCode.CJK_SPACE)
    private val defaults=listOf(SwipeAction.SHIFT,SwipeAction.HIDE_KEYBOARD,SwipeAction.SWITCH_TO_NEXT_SUBTYPE,
        SwipeAction.SWITCH_TO_PREV_SUBTYPE,SwipeAction.NO_ACTION,SwipeAction.MOVE_CURSOR_LEFT,
        SwipeAction.MOVE_CURSOR_RIGHT,SwipeAction.SHOW_INPUT_METHOD_PICKER,SwipeAction.DELETE_CHARACTERS_PRECISELY)
    private fun renderer()=KeyboardRenderer(emptyArray()).also {r ->
        r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",
            emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
        val g=codes.flatMapIndexed {i,code -> listOf(i*100f,0f,(i+1)*100f,100f,i*100f,0f,(i+1)*100f,100f,1f,code.toFloat())}.toFloatArray()
        r.update(floatArrayOf(900f,120f,20f),arrayOf("characters","unshifted")+Array(codes.size) {"character"},
            intArrayOf(),g,Array(codes.size*3) {""},emptyArray())
    }
    private class Trace(private val r: KeyboardRenderer,private val ref: ReferenceRendererTouch,
        private val actions: List<SwipeAction>) {
        var label="scripted"
        var time=1_000_000_000L
        var step=0
        fun event(action: Int,ids: IntArray,xy: FloatArray,index: Int=0,dt: Long=16_000_000,history: Boolean=false) {
            time+=dt
            val times=if(history) longArrayOf(0,time-5_000_000,time) else longArrayOf(0,time)
            val points=if(history) xy+xy else xy
            val expected=ref.event(action,index,ids,times,points)
            val actual=r.touch(intArrayOf(action,index)+ids,times,points)
            compare(expected,actual,"step ${step++}, action $action")
        }
        fun hold(id: Int,popup: Boolean=false,uninterrupted: Boolean=true) {
            compare(ref.longPress(id,popup,uninterrupted),r.longPress(id,popup,uninterrupted),"hold step ${step++}")
        }
        private fun compare(expected: IntArray,actual: IntArray,where: String) {
            fun normalized(a: IntArray,native: Boolean)=a.toList().chunked(3).flatMap {
                if(it[0]==SWIPE_ACTION) listOf(it[0],if(native) actions[it[1]].ordinal else it[1],0) else it
            }.toIntArray()
            assertContentEquals(normalized(expected,false),normalized(actual,true),"$label $where")
        }
    }
    private fun configured(r: KeyboardRenderer,actions: List<SwipeAction>,glide: Boolean,density: Float=1f,raw: Boolean=false,gif: Boolean=false): Trace {
        r.touch(intArrayOf(3,0),longArrayOf(),floatArrayOf())
        r.gestures(floatArrayOf(density,32f,if(glide) 1f else 0f,if(raw) 1f else 0f,if(gif) 1f else 0f,1f,200f,32f,1900f),actions.map {it.name}.toTypedArray())
        return Trace(r,ReferenceRendererTouch(codes,actions,glide,density,raw,gif),actions)
    }
    @Test fun scriptedGestureRegressionsAndEveryConfiguredActionMatch(): Unit = renderer().use {r ->
        // 300-press/e-hold event shape retained from the stage 3 benchmark, scaled to this fixture.
        configured(r,defaults,true).apply {
            repeat(300) {event(0,intArrayOf(0),floatArrayOf(50f,70f),dt=85_000_000);event(1,intArrayOf(0),floatArrayOf(50f,70f),dt=35_000_000)}
            repeat(8) {event(0,intArrayOf(0),floatArrayOf(150f,70f));hold(0,true);event(1,intArrayOf(0),floatArrayOf(150f,70f),dt=500_000_000)}
        }
        for(glide in listOf(false,true)) for(start in codes.indices) configured(r,defaults,glide).apply {
            val x=start*100f+50f
            event(0,intArrayOf(0),floatArrayOf(x,70f))
            event(2,intArrayOf(0),floatArrayOf((x-100).coerceAtLeast(5f),70f),history=true)
            event(2,intArrayOf(0),floatArrayOf((x-180).coerceAtLeast(5f),70f))
            event(1,intArrayOf(0),floatArrayOf((x-180).coerceAtLeast(5f),70f))
        }
        for(action in SwipeAction.entries) for(slot in defaults.indices) configured(r,defaults.toMutableList().also {it[slot]=action},false).apply {
            val x=when(slot) {in 4..7->250f;8->350f;else->150f}
            event(0,intArrayOf(0),floatArrayOf(x,70f))
            if(slot==7) hold(0) else {
                val end=when(slot) {0,4->floatArrayOf(x,21f);1->floatArrayOf(x,119f);3,6->floatArrayOf(x+150,70f);else->floatArrayOf(x-140,70f)}
                event(2,intArrayOf(0),end,dt=5_000_000);event(2,intArrayOf(0),end,dt=5_000_000)
                event(1,intArrayOf(0),end,dt=5_000_000)
            }
            event(3,intArrayOf(0),floatArrayOf(x,70f))
        }
        // Shift held while typing; rolling characters; pointer index reordering and slot reuse.
        configured(r,defaults,true).apply {
            event(0,intArrayOf(0),floatArrayOf(450f,70f));event(5,intArrayOf(0,7),floatArrayOf(450f,70f,50f,70f),1)
            event(6,intArrayOf(7,0),floatArrayOf(50f,70f,450f,70f));event(1,intArrayOf(0),floatArrayOf(450f,70f))
            event(0,intArrayOf(0),floatArrayOf(50f,70f));event(5,intArrayOf(0,7),floatArrayOf(50f,70f,150f,70f),1)
            event(6,intArrayOf(0,7),floatArrayOf(50f,70f,150f,70f));event(5,intArrayOf(7,14),floatArrayOf(150f,70f,50f,70f),1)
            event(3,intArrayOf(7,14),floatArrayOf(150f,70f,50f,70f))
        }
    }
    @Test fun seededMultiPointerSequencesMatchCompose()=renderer().use {r ->
        val random=Random(0x3b0302)
        repeat(1800) {run ->
            val actions=if(run%3==0) defaults else List(9) {SwipeAction.entries[random.nextInt(SwipeAction.entries.size)]}
            val trace=configured(r,actions,run%2==0,listOf(.75f,1f,2.625f)[run%3],run%11==0,run%13==0)
            trace.label="seeded run $run"
            val ids=mutableListOf(0);val positions=mutableMapOf(0 to (random.nextInt(9)*100f+50f to 70f))
            fun event(action: Int,index: Int=0,history: Boolean=false) {
                trace.event(action,ids.toIntArray(),ids.flatMap {positions.getValue(it).let {p->listOf(p.first,p.second)}}.toFloatArray(),index,
                    dt=listOf(1L,8L,16L,41L,125L,501L)[random.nextInt(6)]*1_000_000,history=history)
            }
            event(0)
            repeat(24) {
                when(random.nextInt(7)) {
                    0 -> if(ids.size<4) {
                        val id=listOf(1,7,14,31).first {it !in ids};ids.add(id);positions[id]=random.nextInt(9)*100f+50f to 70f
                        event(5,ids.lastIndex)
                    }
                    1 -> if(ids.size>1) {val index=random.nextInt(ids.size);event(6,index);positions.remove(ids.removeAt(index))}
                    2 -> trace.hold(ids.random(random),random.nextBoolean(),random.nextBoolean())
                    else -> {
                        for(id in ids) {val p=positions.getValue(id);positions[id]=(p.first+random.nextInt(-170,171)).coerceIn(1f,899f) to random.nextInt(25,116).toFloat()}
                        if(random.nextBoolean()) ids.shuffle(random)
                        event(2,history=random.nextBoolean())
                    }
                }
            }
            event(if(run%5==0) 3 else 1,random.nextInt(ids.size))
        }
    }
}
