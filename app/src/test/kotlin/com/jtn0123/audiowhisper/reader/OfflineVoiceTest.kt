package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.ReaderRules.VoiceFacts
import java.util.Locale
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertSame

/**
 * Read-aloud's fallback voice stays on the phone: the choice crosses into Rust and back with each
 * voice's facts in the right places.
 */
class OfflineVoiceTest {
    private fun voice(locale: String, needsNetwork: Boolean = false, installed: Boolean = true, quality: Int = 300, latency: Int = 300) =
        VoiceFacts(locale.substringBefore('_'), locale.substringAfter('_', ""), needsNetwork, installed, quality, latency)

    private fun choose(voices: List<VoiceFacts>, default: VoiceFacts?, fallback: Locale = Locale.US) =
        OfflineVoice.choose(voices, default, fallback) { it }

    @Test
    fun theDefaultVoiceIsKeptWhenItWorksOffline() {
        val default = voice("en_US")
        val voices = listOf(voice("en_US", quality = 500), default)
        assertSame(default, choose(voices, default))
    }

    @Test
    fun aDefaultThatNeedsTheNetworkGivesWayToAnOfflineVoiceForItsLanguage() {
        val online = voice("en_GB", needsNetwork = true, quality = 500)
        val offline = voice("en_GB", quality = 200)
        val voices = listOf(online, voice("en_US", quality = 400), offline, voice("fr_FR"))
        assertSame(offline, choose(voices, online))
    }

    @Test
    fun aVoiceWhoseDataIsNotInstalledNeverReads() {
        val missing = voice("en_US", installed = false, quality = 500)
        val there = voice("en_US", quality = 100)
        assertSame(there, choose(listOf(missing, there), missing))
        assertEquals(null, choose(listOf(missing), missing))
    }

    @Test
    fun withNoOfflineVoiceForTheLanguageNothingReads() {
        val online = voice("en_US", needsNetwork = true)
        assertEquals(null, choose(listOf(online, voice("de_DE")), online))
        assertEquals(null, choose(listOf(online), default = null))
        assertEquals(null, choose(emptyList(), default = null))
    }

    @Test
    fun withoutADefaultThePhonesLanguageIsRead() {
        val british = voice("en_GB")
        val german = voice("de_DE", quality = 500)
        assertSame(british, choose(listOf(german, british), default = null, fallback = Locale.UK))
        assertSame(german, choose(listOf(german, british), default = null, fallback = Locale.GERMANY))
    }

    @Test
    fun qualityThenLatencyBreakATieWithinTheCountry() {
        val slow = voice("en_US", quality = 400, latency = 400)
        val fast = voice("en_US", quality = 400, latency = 200)
        val worse = voice("en_US", quality = 300, latency = 100)
        val online = voice("en_US", needsNetwork = true)
        assertSame(fast, choose(listOf(worse, slow, fast), online))
    }
}
