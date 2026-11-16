package com.jtn0123.audiowhisper.rust

import java.io.File

/**
 * The dictionaries the match tests run on: the one the app ships, and AOSP's English dictionary
 * that it shipped before the corpus rebuild, kept unchanged as a fixture.
 */
object TestDictionaries {
    val shipped = File("../../app/src/main/assets/dicts/main_en-US.dict")
    val aosp = File("crates/latin/tests/fixtures/aosp_main_en-US.dict")
    val both get() = listOf(aosp, shipped)
}
