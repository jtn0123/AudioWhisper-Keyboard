package com.jtn0123.audiowhisper.rust

/** Index plans only; timestamps are Long, preference duration arithmetic intentionally remains Int. */
object ClipboardRules {
    init { RustLibrary.load() }
    /** Records: timestamp/type/pinned/sensitive/hasText. Config: now/enabled/limitEnabled/limit/old/minutes/sensitive/seconds/newHasText. */
    fun plan(records: LongArray, texts: Array<String>, config: LongArray): Array<IntArray> =
        ClipboardNative.plan(records, texts, config).map { it as IntArray }.toTypedArray()
    external fun mimeMatches(concrete: String, desired: String): Boolean
    external fun eligible(types: Array<String>, editor: Array<String>): Boolean
}

internal object ClipboardNative {
    external fun plan(records: LongArray, texts: Array<String>, config: LongArray): Array<Any>
}
