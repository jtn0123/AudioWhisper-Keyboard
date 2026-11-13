package com.jtn0123.audiowhisper.rust

/** Key/domain policy in crates/keys. Integers are the existing layout wire values.
 * State fields: mode, variation, shift, UI mode, RTL, selection, manual selection/start/end,
 * incognito, actions overflow/editor, subtype dialog, composing, half width.
 * Selector kinds: case, shift, variation, direction, width; children retain Kotlin order.
 */
object KeyModels {
    init { RustLibrary.load() }

    @JvmStatic external fun codes(): IntArray
    @JvmStatic external fun read(raw: Long, field: Int): Int
    @JvmStatic external fun write(raw: Long, field: Int, value: Int): Long
    @JvmStatic external fun select(kind: Int, raw: Long, present: Int): Int
    @JvmStatic external fun isCurrency(code: Int): Boolean
    @JvmStatic external fun currencySlot(code: Int): Int
    @JvmStatic external fun currencyDefaults(): IntArray
    @JvmStatic external fun isSpace(type: Int, code: Int): Boolean
    @JvmStatic external fun predefined(): Array<String>
    @JvmStatic external fun codeKind(code: Int): Int
    @JvmStatic external fun text(code: Int, label: String, display: Boolean, nonSpacing: Boolean): String
    @JvmStatic external fun multiText(codes: IntArray, label: String, display: Boolean): String
    @JvmStatic external fun popupMode(mode: Int): Boolean
    @JvmStatic external fun popupGroup(group: Int): String?
    @JvmStatic external fun popupSource(variation: Int, present: Int): Int
    @JvmStatic external fun enabled(code: Int, a: Int, b: Int): Boolean
    @JvmStatic external fun visible(code: Int, utility: Boolean, action: Int, language: Boolean): Boolean
    @JvmStatic external fun phoneHint(code: Int): String?
}
