package com.jtn0123.audiowhisper.gif

import java.io.ByteArrayOutputStream
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream

/** A download went past its cap, [maxBytes]. */
class TooLargeException(val maxBytes: Int) : IOException("Response is larger than $maxBytes bytes")

/** Copies [input] to [output], throwing [TooLargeException] once more than [maxBytes] have arrived. */
internal fun copyLimited(input: InputStream, output: OutputStream, maxBytes: Int) {
    val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
    var total = 0
    while (true) {
        val read = input.read(buffer)
        if (read < 0) return
        total += read
        if (total > maxBytes) throw TooLargeException(maxBytes)
        output.write(buffer, 0, read)
    }
}

/** All of [input], or [TooLargeException] if it is longer than [maxBytes]. */
internal fun readLimited(input: InputStream, maxBytes: Int): ByteArray =
    ByteArrayOutputStream().also { copyLimited(input, it, maxBytes) }.toByteArray()
