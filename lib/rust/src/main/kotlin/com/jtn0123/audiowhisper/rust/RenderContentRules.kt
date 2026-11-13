package com.jtn0123.audiowhisper.rust

/** Font-independent geometry oracle; drawing calls the same functions inside Rust. */
object RenderContentRules {
    init { RustLibrary.load() }
    external fun phoneLabel(mode: String, kind: String, boundsAndMetrics: FloatArray, hint: Boolean): FloatArray
}
