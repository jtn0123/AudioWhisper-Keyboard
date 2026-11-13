package com.jtn0123.audiowhisper.rust

/** Stateless UTF-16 and f32-bit transport for Snygg values and ordered property decoding. */
object SnyggValues {
    init { RustLibrary.load() }
    @JvmStatic external fun call(operation: Int, input: Array<String>): Array<String>
}
