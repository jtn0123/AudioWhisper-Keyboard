package com.jtn0123.audiowhisper.rust

/** Copied UTF-16 fields, including empty strings and lone surrogates, in one JNI string. */
internal class Utf16Batch(texts: List<String>) {
    val lengths = IntArray(texts.size) { texts[it].length }
    val text = texts.joinToString("")
}
