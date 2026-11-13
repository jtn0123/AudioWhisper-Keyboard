package com.jtn0123.audiowhisper.rust

/** UTF-16/object-token transport to the Java-free compiler and editor. */
object SnyggCompiler {
    init { RustLibrary.load() }
    @JvmStatic external fun call(operation: Int, input: Array<String>): Array<String>
}
