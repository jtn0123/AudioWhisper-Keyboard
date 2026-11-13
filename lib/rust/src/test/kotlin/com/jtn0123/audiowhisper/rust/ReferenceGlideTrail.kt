package com.jtn0123.audiowhisper.rust

import kotlin.math.sqrt

/** Test-only TextKeyboardLayoutController.drawGlideTrail, with drawCircle collecting floats.
 * ValueAnimator/AccelerateInterpolator's clock is supplied explicitly in nanoseconds.
 */
internal object ReferenceGlideTrail {
    fun circles(times: LongArray, points: FloatArray, finish: Long, now: Long, duration: Long, strip: Float): FloatArray {
        if(duration<=0) return floatArrayOf(0f)
        val fraction=((now-finish).coerceAtLeast(0).toFloat()/duration.toFloat()).coerceIn(0f,1f)
        if(finish>=0 && fraction>=1f) return floatArrayOf(0f)
        val initialRadius=if(finish>=0) 20f + fraction*fraction * -20f else 20f
        var radius=initialRadius
        var prevX=points.lastOrNull() ?: 0f
        var prevY=points.lastOrNull() ?: 0f
        if(times.isNotEmpty()) {prevX=points[points.size-2];prevY=points.last()}
        val out=mutableListOf<Float>()
        for(i in times.lastIndex downTo 1) {
            if(now-times[i-1]>duration) break
            val dx=prevX-points[(i-1)*2];val dy=prevY-points[(i-1)*2+1]
            val dist=sqrt(dx*dx+dy*dy)
            val numPoints=(dist/3f).toInt()
            for(j in 0 until numPoints) {
                radius*=.99f
                val intermediateX=points[i*2]*(1-j.toFloat()/numPoints)+points[(i-1)*2]*(j.toFloat()/numPoints)
                val intermediateY=points[i*2+1]*(1-j.toFloat()/numPoints)+points[(i-1)*2+1]*(j.toFloat()/numPoints)
                out.addAll(listOf(intermediateX,intermediateY+strip,radius))
                prevX=intermediateX;prevY=intermediateY
            }
        }
        return (listOf(if(out.isEmpty()) 0f else 1f)+out).toFloatArray()
    }
}
