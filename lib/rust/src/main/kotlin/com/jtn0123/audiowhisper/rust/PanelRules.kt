package com.jtn0123.audiowhisper.rust

/** Opaque value ids keep equality and preference serialization at the Android storage edge. */
object PanelRules {
    init { RustLibrary.load() }
    @JvmStatic external fun overflowAndHistoryMoves(config: IntArray): IntArray
    @JvmStatic external fun scroll(config: FloatArray): FloatArray
    @JvmStatic external fun selection(config: IntArray): IntArray
    @JvmStatic external fun editing(codes: IntArray, hold: Long, repeat: Long, words: Long): IntArray
    @JvmStatic external fun cells(config: FloatArray): FloatArray
    @JvmStatic external fun emojiChoices(tones: IntArray, tone: Int): IntArray
    fun history(pinned: IntArray, recent: IntArray, config: IntArray): Array<IntArray> =
        PanelPolicyNative.history(pinned, recent, config).map { it as IntArray }.toTypedArray()
    fun arrange(config: IntArray, dynamic: IntArray, hidden: IntArray): Array<IntArray> =
        PanelPolicyNative.arrange(config, dynamic, hidden).map { it as IntArray }.toTypedArray()
}

internal object PanelPolicyNative {
    @JvmStatic external fun history(pinned: IntArray, recent: IntArray, config: IntArray): Array<Any>
    @JvmStatic external fun arrange(config: IntArray, dynamic: IntArray, hidden: IntArray): Array<Any>
}
