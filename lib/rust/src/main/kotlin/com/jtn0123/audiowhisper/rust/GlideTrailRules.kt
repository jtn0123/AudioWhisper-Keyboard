package com.jtn0123.audiowhisper.rust

/** Font-independent geometry comparison against Compose's test-only circle drawing reference. */
object GlideTrailRules {
    init { RustLibrary.load() }
    external fun circles(times: LongArray, points: FloatArray, finish: Long, now: Long, duration: Long, strip: Float): FloatArray
}
