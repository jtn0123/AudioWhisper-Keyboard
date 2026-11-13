package com.jtn0123.audiowhisper.rust

object SubtypeRules {
    init { RustLibrary.load() }
    external fun geometry(config: FloatArray): FloatArray
    external fun rows(selected: Long, ids: LongArray, labels: Array<String>): Array<String>
}
