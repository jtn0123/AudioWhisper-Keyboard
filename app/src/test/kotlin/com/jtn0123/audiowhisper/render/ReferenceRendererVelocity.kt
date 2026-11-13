package com.jtn0123.audiowhisper.render

/** Test-only Android 16 LSQ2 reference; matches VelocityTracker's timestamps/history policy. */
internal class ReferenceRendererVelocity {
    private val samples = mutableMapOf<Int,MutableList<Triple<Long,Float,Float>>>()
    private var last = 0L
    fun add(action: Int, index: Int, ids: IntArray, times: LongArray, xy: FloatArray) {
        if (action == 0 || action == 3) samples.clear()
        if (action == 5) samples.remove(ids[index])
        if (action == 1 || action == 6) {
            if (times.last()-last > 40_000_000) samples.clear()
            return
        }
        if (action !in listOf(0,2,5)) return
        for (h in 1 until times.size) for ((p,id) in ids.withIndex()) {
            val now=times[h]
            if (id in samples && now-last > 40_000_000) samples.clear()
            last=now
            val trace=samples.getOrPut(id) { mutableListOf() }
            if (trace.lastOrNull()?.first == now) trace.removeAt(trace.lastIndex)
            trace.add(Triple(now,xy[((h-1)*ids.size+p)*2],xy[((h-1)*ids.size+p)*2+1]))
            while (trace.size>20 || now-trace.first().first>100_000_000) trace.removeAt(0)
        }
    }
    fun get(id: Int): Pair<Float,Float> {
        val p=samples[id] ?: return 0f to 0f
        fun axis(y: Boolean): Float {
            if (p.size<2) return 0f
            fun value(i: Int)=if(y) p[i].third else p[i].second
            if (p.size==2) {
                val x=floatArrayOf(0f,-(p[1].first-p[0].first).toFloat()*0.000000001f)
                val q0=1f/kotlin.math.sqrt(2f)
                val dot=x[0]*q0+x[1]*q0
                val q=floatArrayOf(x[0]-dot*q0,x[1]-dot*q0)
                val norm=kotlin.math.sqrt(q[0]*q[0]+q[1]*q[1])
                if(norm<.000001f) return 0f
                val inv=1f/norm;q[0]*=inv;q[1]*=inv
                return (q[0]*value(1)+q[1]*value(0))/(q[0]*x[0]+q[1]*x[1])
            }
            val t=p.map { -(p.last().first-it.first).toFloat()*0.000000001f }
            val v=p.indices.map(::value)
            var sx=0f;var sx2=0f;var sx3=0f;var sx4=0f;var sy=0f;var sxy=0f;var sx2y=0f
            for (i in t.indices) {val x=t[i];val x2=x*x;val x3=x2*x;val x4=x3*x
                sx+=x;sx2+=x2;sxy+=x*v[i];sx2y+=x2*v[i];sy+=v[i];sx3+=x3;sx4+=x4}
            val n=p.size.toFloat();val xx=sx2-sx*sx/n;val xy=sxy-sx*sy/n
            val xx2=sx3-sx*sx2/n;val x2y=sx2y-sx2*sy/n;val x2x2=sx4-sx2*sx2/n
            val d=xx*x2x2-xx2*xx2
            return if(d==0f) 0f else (xy*x2x2-x2y*xx2)/d
        }
        return (axis(false)*1000f/1000f) to (axis(true)*1000f/1000f)
    }
}
