package com.jtn0123.audiowhisper.rust

object ImeChromeRules {
    init { RustLibrary.load() }
    external fun fontMultiplier(config: IntArray): Float
    external fun rowHeights(base: Float, config: IntArray): FloatArray
    external fun minimumTarget(config: FloatArray): FloatArray
    external fun oneHanded(config: IntArray): FloatArray
    external fun watermark(flags: IntArray): Boolean
    external fun previewSize(config: FloatArray): IntArray
    external fun startEllipsis(labels: Array<String>, config: FloatArray): Array<String>
    external fun extract(config: FloatArray): FloatArray
    external fun sideButtons(config: FloatArray): FloatArray
}
