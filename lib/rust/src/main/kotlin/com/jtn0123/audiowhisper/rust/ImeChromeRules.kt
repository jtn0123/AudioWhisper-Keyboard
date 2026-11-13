package com.jtn0123.audiowhisper.rust

object ImeChromeRules {
    init { RustLibrary.load() }
    external fun oneHanded(config: IntArray): FloatArray
    external fun extract(config: FloatArray): FloatArray
    external fun sideButtons(config: FloatArray): FloatArray
}
