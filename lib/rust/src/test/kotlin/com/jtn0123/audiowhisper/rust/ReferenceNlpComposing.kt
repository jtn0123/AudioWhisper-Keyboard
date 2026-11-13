package com.jtn0123.audiowhisper.rust

/** The removed default provider rule; Android ICU still supplies status/start/end. */
internal object ReferenceNlpComposing {
    fun range(status: Int, start: Int, end: Int, text: String): List<String> = if (status != 0) {
        val adjusted = (start - 1).takeIf { text.getOrNull(it) == ':' } ?: start
        listOf(adjusted.toString(), end.toString())
    } else listOf("-1", "-1")
}
