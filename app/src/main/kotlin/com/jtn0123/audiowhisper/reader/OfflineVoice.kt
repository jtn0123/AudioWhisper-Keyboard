package com.jtn0123.audiowhisper.reader

import android.speech.tts.TextToSpeech
import com.jtn0123.audiowhisper.rust.ReaderRules
import java.util.Locale
import android.speech.tts.Voice as TtsVoice

/**
 * Which of the phone's text-to-speech voices read-aloud may use: one that works without the
 * network, since nothing it reads may leave the phone. The engine's default is kept when it is
 * offline; otherwise the best offline voice for its language; null if there is none, and then
 * nothing is read. The rule is Rust's (lib/rust/crates/reader/src/voices.rs).
 */
internal object OfflineVoice {
    /**
     * A speech parameter that asks an engine from before API 21, which picks its own voice, not
     * to synthesise on a server: `TextToSpeech.Engine.KEY_FEATURE_NETWORK_SYNTHESIS`, whose
     * value this is, given as "false". The platform marks the constant deprecated in favour of
     * choosing the voice, which [choose] does; this is the second guard.
     */
    const val NETWORK_SYNTHESIS = "networkTts"

    /** The voice for [default]'s language (or [fallback]'s, without a default) among [voices]. */
    fun choose(voices: Collection<TtsVoice>, default: TtsVoice?, fallback: Locale): TtsVoice? =
        choose(voices.toList(), default, fallback, ::facts)

    /** [choose] for anything [facts] describes. */
    fun <V> choose(voices: List<V>, default: V?, fallback: Locale, facts: (V) -> ReaderRules.VoiceFacts): V? {
        val wanted = default?.let(facts)
        val language = wanted?.language ?: fallback.language
        val country = wanted?.country ?: fallback.country
        val index = ReaderRules.offlineVoice(voices.map(facts), language, country, voices.indexOf(default))
        return voices.getOrNull(index)
    }

    private fun facts(voice: TtsVoice) = ReaderRules.VoiceFacts(
        language = voice.locale.language,
        country = voice.locale.country,
        needsNetwork = voice.isNetworkConnectionRequired,
        installed = TextToSpeech.Engine.KEY_FEATURE_NOT_INSTALLED !in voice.features.orEmpty(),
        quality = voice.quality,
        latency = voice.latency,
    )
}
