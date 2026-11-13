package com.jtn0123.audiowhisper.rust

/** Stateless UTF-16 transport for the Java-free snygg crate. The Snygg API owns object allocation. */
object SnyggGrammar {
    init { RustLibrary.load() }

    @JvmStatic external fun call(operation: Int, input: Array<String>): Array<String>
}
