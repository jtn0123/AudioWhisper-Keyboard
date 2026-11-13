package com.jtn0123.audiowhisper.rust

object MediaRules {
    init { RustLibrary.load() }
    external fun scrollChips(config: FloatArray): FloatArray
    external fun grid(config: FloatArray): FloatArray
    external fun trace(config: IntArray, operations: LongArray): LongArray
}
