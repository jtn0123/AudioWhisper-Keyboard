package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.DictationAudio
import com.jtn0123.audiowhisper.rust.MicRules
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * Seeded traces holding the microphone's Rust rules ([MicRules], dictation::audio) to main's
 * Kotlin, whose answers are golden digests (docs/phase6-kotlin-references.md, generated at
 * e00807e9d): recordings, effect by effect; WAV files of every shape; and the buffer and read rules.
 *
 * One intended difference, in the digest: a WAV file main never finished reading (its walk came
 * back to a chunk it had read) or crashed on (a chunk before the file's start, a negative sample
 * count) is refused as holding no audio, the IllegalArgumentException main threw for other bad
 * files. At the generator, the 60,000 files reached each of those kinds at least 20 times.
 */
class MicTraceTest {
    /** One read: what it returns, and the samples it leaves in the buffer. */
    private class Read(val count: Int, val samples: FloatArray)

    /** Audio playing a script of reads, then blocking until stopped and returning [afterStop]. */
    private class ScriptedAudio(private val script: List<Read>, private val afterStop: Read, val effects: MutableList<String>) : MicAudio {
        private var next = 0
        private val stopped = CountDownLatch(1)
        val idle = CountDownLatch(1)

        override fun read(buffer: FloatArray): Int {
            val read = if (next < script.size) script[next++] else afterStop.also { idle.countDown(); stopped.await() }
            read.samples.copyInto(buffer)
            return read.count
        }

        override fun stop() = stopped.countDown()

        override fun close() {
            synchronized(effects) { effects += "close" }
            idle.countDown()
        }
    }

    @Test
    fun recordingsMatchMain() {
        val tape = GoldenTape("mic-recordings")
        for (seed in SEEDS) {
            val random = Random(seed)
            val seen = HashMap<String, Int>()
            var reads = 0
            var effects = 0
            while (reads < READS) {
                val script = List(random.nextInt(1, 120)) { read(random, seen) }
                val afterStop = read(random, seen, stopping = true)
                reads += script.size + 1
                val new = record(script, afterStop)
                tape.record(new)
                effects += new.size
            }
            println("Mic trace seed $seed: $reads reads, $effects effects, 0 differ, seen $seen")
            for (kind in listOf("full", "partial", "nothing", "error", "error after stop", "after stop")) {
                assertTrue((seen[kind] ?: 0) >= MIN_SEEN, "seed $seed saw $kind ${seen[kind]} times")
            }
        }
        tape.finish()
    }

    private fun record(script: List<Read>, afterStop: Read): List<String> {
        val effects = mutableListOf<String>()
        val audio = ScriptedAudio(script, afterStop, effects)
        val onLevel = { level: Float -> synchronized(effects) { effects += "level ${bits(level)}" } }
        val onError = { code: Int -> synchronized(effects) { effects += "error $code" } }
        val onAudio = { samples: FloatArray -> synchronized(effects) { effects += "audio ${samples.size} ${hash(samples)}" } }
        val recorder = MicRecorder(onLevel, onError, onAudio)
        recorder.start { audio }
        assertTrue(audio.idle.await(10, TimeUnit.SECONDS), "the recording neither ended nor ran out of reads")
        recorder.stop()
        return synchronized(effects) { effects.toList() }
    }

    private fun read(random: Random, seen: HashMap<String, Int>, stopping: Boolean = false): Read {
        val size = DictationAudio.chunkSamples
        val roll = random.nextInt(100)
        val (kind, count) = when {
            stopping && roll < 40 -> "nothing" to 0
            stopping && roll < 70 -> "error after stop" to -random.nextInt(1, 7)
            stopping -> "after stop" to random.nextInt(1, size + 1)
            roll < 75 -> "full" to size
            roll < 88 -> "partial" to random.nextInt(1, size)
            roll < 96 -> "nothing" to 0
            else -> "error" to -random.nextInt(1, 7)
        }
        seen[kind] = (seen[kind] ?: 0) + 1
        return Read(count, samples(random, size))
    }

    /** Speech, silence, clipping, and now and then a value no mic gives, all at once in places. */
    private fun samples(random: Random, size: Int): FloatArray {
        val loudness = listOf(0f, 0.001f, 0.05f, 0.3f, 1f, 3f)[random.nextInt(6)]
        return FloatArray(size) {
            when (random.nextInt(2000)) {
                0 -> Float.NaN
                1 -> if (random.nextBoolean()) Float.MAX_VALUE else -Float.MIN_VALUE
                else -> (random.nextFloat() * 2 - 1) * loudness
            }
        }
    }

    @Test
    fun wavFilesMatchMain() {
        val tape = GoldenTape("mic-wav-files")
        for (seed in SEEDS) {
            val random = Random(seed)
            val seen = HashMap<String, Int>()
            repeat(WAVS) {
                val answer = answer(runCatching { MicRules.pcm16(wav(random)) })
                tape.record(answer)
                val kind = when {
                    answer.startsWith("audio") -> "audio"
                    answer == "refused Not a WAV file" -> "not a wav"
                    answer == "refused No audio in the WAV file" -> "no audio"
                    else -> answer
                }
                seen[kind] = (seen[kind] ?: 0) + 1
            }
            println("WAV trace seed $seed: $WAVS files, seen $seen")
            for (kind in listOf("audio", "not a wav", "no audio")) {
                assertTrue((seen[kind] ?: 0) >= MIN_SEEN, "seed $seed saw $kind ${seen[kind]} times")
            }
        }
        tape.finish()
    }

    /** A parse's answer as text: every sample's bits, or the refusal and its message. */
    private fun answer(result: Result<FloatArray>): String = result.fold(
        { samples -> "audio " + samples.joinToString(",") { bits(it) } },
        { if (it is IllegalArgumentException) "refused ${it.message}" else "threw ${it.javaClass.simpleName}" },
    )

    /** A WAV file: chunks of every kind, sizes true and false, cut short or whole. */
    private fun wav(random: Random): ByteArray {
        val out = java.io.ByteArrayOutputStream()
        out.write(if (random.nextInt(25) == 0) "RIFX".toByteArray() else "RIFF".toByteArray())
        out.write(int(random.nextInt()))
        out.write(if (random.nextInt(25) == 0) "WAVF".toByteArray() else "WAVE".toByteArray())
        repeat(random.nextInt(0, 5)) {
            val id = listOf("fmt ", "LIST", "junk", "data", "fact")[random.nextInt(5)].toByteArray()
            val body = ByteArray(random.nextInt(0, 41)).also(random::nextBytes)
            if (random.nextInt(8) == 0 && body.size >= 12) "data".toByteArray().copyInto(body, random.nextInt(body.size - 11))
            out.write(id)
            out.write(int(size(random, body.size, out.size())))
            out.write(body)
            if (body.size % 2 == 1) out.write(0)
        }
        val bytes = out.toByteArray()
        return if (random.nextInt(6) == 0) bytes.copyOf(random.nextInt(bytes.size + 1)) else bytes
    }

    /** A chunk's size field: its true size mostly, else a lie forward, back, or out of range. */
    private fun size(random: Random, size: Int, at: Int): Int = when (random.nextInt(20)) {
        in 0..12 -> size
        13 -> size + random.nextInt(1, 60)
        14 -> -random.nextInt(1, 9)
        15 -> -random.nextInt(9, at + 40)
        16 -> listOf(Int.MAX_VALUE, Int.MIN_VALUE, Int.MAX_VALUE - 20)[random.nextInt(3)]
        17 -> -(at - 4) - random.nextInt(0, 3)
        else -> random.nextInt(0, size + 1)
    }

    @Test
    fun bufferAndReadRulesMatchMain() {
        val random = Random(SEEDS[0])
        val minimums = listOf(Int.MIN_VALUE, -2, -1, 0, 31_999, 32_000, 32_001, Int.MAX_VALUE) + List(10_000) { random.nextInt() }
        GoldenTape.check("mic-buffer-bytes") { tape -> for (minimum in minimums) tape.record(MicRules.bufferBytes(minimum).toString()) }
        println("Mic buffer trace: ${minimums.size} minimums")
    }

    private fun bits(value: Float) = if (value.isNaN()) "NaN" else value.toRawBits().toString()

    private fun hash(samples: FloatArray) = samples.fold(17) { h, s -> 31 * h + bits(s).hashCode() }

    private fun int(value: Int) = ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN).putInt(value).array()

    private companion object {
        val SEEDS = longArrayOf(20261122, 7, 4_000_000_007)
        const val READS = 50_000
        const val WAVS = 20_000
        const val MIN_SEEN = 20
    }
}
