package com.jtn0123.audiowhisper.rust

/** Test-only specification for decoded-frame admission and visibility clocks. No Android types. */
internal fun referenceMedia(bounds: IntArray, ops: LongArray): LongArray {
    data class Frame(val bytes: Int,val delay: Long)
    data class Animation(val frames: MutableList<Frame> = mutableListOf(),var frame: Int=0,var deadline: Long?=null,var touched: Long=0,var repeats: Int=-1,var plays: Int=0,var finished: Boolean=false)
    val entries=sortedMapOf<Long,Animation>()
    var visible=emptySet<Long>()
    var clock=0L
    fun bytes()=entries.values.sumOf { e->e.frames.sumOf { it.bytes } }
    var budget=bounds[0];var perItem=bounds[1]
    val output=mutableListOf<Long>()
    for(o in ops.toList().chunked(8)) {
        var accepted=false
        val id=o[1]
        when(o[0].toInt()) {
            0 -> {
                val index=o[2].toInt();val size=(o[4]*o[5]*8).toInt()
                if(index<64 && size<=perItem && size<=budget) {
                    if(index==0) entries.remove(id)
                    val previous=entries[id]
                    if(index==(previous?.frames?.size ?: 0) && (previous?.frames?.sumOf { it.bytes } ?: 0)+size<=perItem) {
                        while(bytes()+size>budget) {
                            val victim=entries.filter { it.key!=id && it.key !in visible }.minByOrNull { it.value.touched }?.key ?: break
                            entries.remove(victim)
                        }
                        if(bytes()+size<=budget) {
                            clock++
                            val entry=entries.getOrPut(id){Animation(touched=clock)}
                            entry.frames.add(Frame(size,o[3].coerceAtLeast(10_000_000)));entry.touched=clock;accepted=true
                        }
                    }
                }
            }
            1 -> {
                visible=if(id==0L) emptySet() else setOf(id)
                for((key,e)in entries) {
                    if(key in visible) {
                        clock++;e.touched=clock
                        if(e.frames.size>1 && e.deadline==null && !e.finished) e.deadline=o[7]+e.frames[e.frame].delay
                    }else e.deadline=null
                }
            }
            2 -> for(e in entries.values) if(e.deadline?.let { o[7]>=it }==true) {
                val next=(e.frame+1)%e.frames.size
                if(next==0)e.plays++
                if(next==0 && e.repeats>=0 && e.plays>e.repeats){e.finished=true;e.deadline=null}
                else{e.frame=next;e.deadline=o[7]+e.frames[e.frame].delay;accepted=true}
            }
            3 -> {entries.clear();visible=emptySet();clock=0;budget=24*1024*1024;perItem=4*1024*1024}
            4 -> entries[id]?.let { it.repeats=o[2].toInt().coerceAtLeast(-1) }
        }
        output.addAll(listOf(entries[id]?.frame?.toLong() ?: -1,entries.values.mapNotNull { it.deadline }.minOrNull() ?: -1,
            bytes().toLong(),entries.size.toLong(),if(accepted)1 else 0))
    }
    return output.toLongArray()
}
internal fun referenceGifGrid(c: FloatArray): FloatArray {
    val cell=(c[0]-8*c[1])/3
    val lanes=FloatArray(3)
    return c.drop(2).flatMap { aspect ->
        val lane=lanes.indices.minBy { lanes[it] }
        val x=lane*(cell+4*c[1]);val y=lanes[lane];val h=cell/aspect.coerceIn(.5f,2.5f)
        lanes[lane]+=h+4*c[1]
        listOf(x,y,x+cell,y+h)
    }.toFloatArray()
}
