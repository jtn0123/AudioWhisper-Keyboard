package com.jtn0123.audiowhisper.suggest

import java.io.File

/**
 * The dictionaries the JVM tests open. A test that compares two implementations runs on [both];
 * a test that pins what the decoder answers runs on [aosp], the dictionary the app shipped before
 * the corpus rebuild, kept unchanged as a fixture so those answers stay pinned.
 */
object TestDictionaries {
    /** The dictionary the app ships. */
    val shipped = File("src/main/assets/dicts/main_en-US.dict")

    /** AOSP's English dictionary, byte for byte as the app shipped it before. */
    val aosp = File("../lib/rust/crates/latin/tests/fixtures/aosp_main_en-US.dict")

    val both get() = listOf(aosp, shipped)
}
