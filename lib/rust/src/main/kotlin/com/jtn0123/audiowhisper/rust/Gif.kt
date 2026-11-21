package com.jtn0123.audiowhisper.rust

/** Whether [place] in recents' order is a GIF sent rather than one from before. */
fun GifRules.isSent(place: Int): Boolean = place < 0

/** The index [place] in recents' order names, among those before or those sent. */
fun GifRules.index(place: Int): Int = if (place < 0) -1 - place else place
