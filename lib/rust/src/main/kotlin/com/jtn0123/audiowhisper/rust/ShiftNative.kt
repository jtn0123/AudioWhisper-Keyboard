package com.jtn0123.audiowhisper.rust

/**
 * The shift key's entry points in libawnative: lib/rust/crates/awnative/src/shift.rs, which
 * documents each. [ShiftRules] wraps them.
 */
internal object ShiftNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun next(state: Int, event: Int, behavior: Int, capsMode: Int, facts: Int): Int

    @JvmStatic external fun isConsecutive(lastCode: Int, lastTime: Long, code: Int, now: Long, timeout: Long): Boolean

    @JvmStatic external fun isDoubleSpacePeriod(text: String): Boolean
}
