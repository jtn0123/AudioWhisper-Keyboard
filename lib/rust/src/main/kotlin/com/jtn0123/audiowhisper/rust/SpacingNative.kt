package com.jtn0123.audiowhisper.rust

/**
 * The spacing rules' entry points in libawnative: lib/rust/crates/awnative/src/spacing.rs, which
 * documents each and how inputs and answers are packed. [SpacingRules] packs and unpacks them.
 */
internal object SpacingNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun commitChar(text: String, typing: IntArray, lookedUp: String?, lengths: IntArray?): Int

    @JvmStatic external fun phantomSpaceBefore(text: String, typing: IntArray, lookedUp: String?, lengths: IntArray?): Int

    @JvmStatic external fun supportsAutoSpace(language: String): Boolean

    @JvmStatic external fun fallbackRule(): Array<String>

    @JvmStatic external fun inactive(): Int

    @JvmStatic external fun autoSpaceActive(stayActiveNextUpdate: Boolean): Int

    @JvmStatic external fun phantomSpaceActive(showComposingRegion: Boolean, stayActiveNextUpdate: Boolean): Int

    @JvmStatic external fun isActive(state: Int): Boolean

    @JvmStatic external fun showsComposingRegion(state: Int): Boolean

    @JvmStatic external fun afterUpdate(state: Int): Int

    @JvmStatic external fun keepsCandidate(state: Int): Boolean
}
