package com.jtn0123.audiowhisper.rust

/** Calls the exact native labelling rules used by the semantic scene; absent from the APK. */
internal object SemanticRules {
    init { System.loadLibrary("awnative") }
    external fun labels(codes: IntArray, labels: Array<String>, shift: String, bar: Int = -1): Array<String>
}
