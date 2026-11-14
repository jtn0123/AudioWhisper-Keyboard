package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.ReaderRules

/** Retained original helper for the unchanged reference comparisons; production uses the platform host. */
internal fun FollowAlong.Companion.fits(span: IntRange, top: Int, bottom: Int): Boolean =
    ReaderRules.fits(span.first, span.last, top, bottom)
