package com.jtn0123.audiowhisper.rust

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals

/** Frozen arithmetic from PhraseListener, driven by deterministic VAD replies. */
private class ReferenceSegmentation {
    private val ring = FloatArray(400_000)
    var end = 0L
    private var speechFrom: Long? = null
    private var nextPreviewAt = 0L
    private var lastPhraseEnd = 0L
    fun add(audio: FloatArray) { for (sample in audio) { ring[(end % ring.size).toInt()] = sample; end++ } }
    fun slice(from: Long, until: Long): FloatArray {
        val first = maxOf(from, end - ring.size, 0L); val last = minOf(until, end)
        if (first >= last) return FloatArray(0)
        return FloatArray((last-first).toInt()) { ring[((first+it)%ring.size).toInt()] }
    }
    fun phrase(start: Long, samples: FloatArray): FloatArray {
        val result = slice(start-4800, start)+samples
        lastPhraseEnd = start+samples.size; speechFrom = null; return result
    }
    fun preview(detected: Boolean): FloatArray {
        if (!detected) { speechFrom = null; return FloatArray(0) }
        val now = end
        val from = speechFrom ?: maxOf(lastPhraseEnd, now-4512-4800).also { speechFrom=it;nextPreviewAt=now+16000/2 }
        if (now<nextPreviewAt) return FloatArray(0)
        nextPreviewAt=now+16000
        return slice(maxOf(from,now-160000),now)
    }
}
internal object SegmentationNative {
    init { RustLibrary.load() }
    @JvmStatic external fun create(): Long
    @JvmStatic external fun free(handle: Long)
    @JvmStatic external fun call(handle: Long, op: Int, audio: FloatArray, from: Long, until: Long, detected: Boolean): FloatArray
}
class DictationSegmentationMatchTest {
    @Test fun recordedModelVadTraces() {
        val lines=checkNotNull(javaClass.getResourceAsStream("/dictation-vad-traces.tsv")).bufferedReader().use { it.readLines() }.drop(1)
        var fixtures=0;var chunks=0;var previews=0;var phrases=0
        for ((fixture, rows) in lines.groupBy { it.substringBefore('\t').toInt() }) {
            val rng=Random(7000+fixture);val reference=ReferenceSegmentation();val handle=SegmentationNative.create()
            fun segments(encoded: String) {
                if (encoded=="-") return
                for (segment in encoded.split(',')) {
                    val (start,length)=segment.split(':').map { it.toLong() }
                    val samples=reference.slice(start,start+length)
                    assertEquals(length.toInt(),samples.size)
                    assertContentEquals(reference.phrase(start,samples),SegmentationNative.call(handle,1,samples,start,0,false),"recorded segment $fixture")
                    phrases++
                }
            }
            try {
                for (row in rows) {
                    val fields=row.split('\t');val end=fields[1].toLong();val detected=fields[2]=="1"
                    val audio=FloatArray((end-reference.end).toInt()) { rng.nextFloat()*2-1 }
                    reference.add(audio);SegmentationNative.call(handle,0,audio,0,0,false)
                    segments(fields[4])
                    val preview=reference.preview(detected)
                    assertContentEquals(preview,SegmentationNative.call(handle,2,FloatArray(0),0,0,detected),"recorded preview $fixture")
                    assertEquals(fields[3].toInt(),preview.size)
                    if (preview.isNotEmpty()) previews++
                    segments(fields[5]);chunks++
                }
            } finally { SegmentationNative.free(handle) }
            fixtures++
        }
        assertEquals(7,fixtures);assertEquals(315,chunks);assertEquals(14,previews);assertEquals(5,phrases)
    }
    @Test fun seededAudioVadSegmentsAndPreviewTimeouts() {
        repeat(200) { seed ->
            val rng=Random(seed);val reference=ReferenceSegmentation();val handle=SegmentationNative.create()
            try {
                repeat(250) { step ->
                    val audio=FloatArray(rng.nextInt(1,3201)) { rng.nextFloat()*2-1 }
                    reference.add(audio);SegmentationNative.call(handle,0,audio,0,0,false)
                    if (step%17==0) {
                        val start=maxOf(0,reference.end-rng.nextInt(1,12000));val samples=reference.slice(start,reference.end)
                        assertContentEquals(reference.phrase(start,samples),SegmentationNative.call(handle,1,samples,start,0,false),"segment $seed/$step")
                    }
                    val detected=step%29!=0
                    assertContentEquals(reference.preview(detected),SegmentationNative.call(handle,2,FloatArray(0),0,0,detected),"preview $seed/$step")
                    val from=reference.end-rng.nextInt(0,500000);val until=reference.end+rng.nextInt(-100,100)
                    assertContentEquals(reference.slice(from,until),SegmentationNative.call(handle,3,FloatArray(0),from,until,false),"history $seed/$step")
                }
            } finally { SegmentationNative.free(handle) }
        }
    }
}
