package com.jtn0123.audiowhisper.dictation

/**
 * The last [capacity] samples heard, addressed by their position since listening began.
 *
 * Silero VAD notices speech a little after it starts, so a phrase it hands over can be
 * missing its first word ("I need…" heard as "need…"). Keeping recent audio lets the
 * listener give each phrase back the moment before the VAD's start. Memory stays fixed
 * however long the dictation runs.
 */
class ReferenceAudioHistory(private val capacity: Int) {
    private val ring = FloatArray(capacity)

    /** Samples added since the start; the next one added is at this position. */
    var end = 0L
        private set

    fun add(audio: FloatArray) {
        for (sample in audio) {
            ring[(end % capacity).toInt()] = sample
            end++
        }
    }

    /** The samples at positions [from] until [until], less any no longer held or not yet heard. */
    fun slice(from: Long, until: Long): FloatArray {
        val first = maxOf(from, end - capacity, 0L)
        val last = minOf(until, end)
        if (first >= last) return FloatArray(0)
        return FloatArray((last - first).toInt()) { ring[((first + it) % capacity).toInt()] }
    }
}
