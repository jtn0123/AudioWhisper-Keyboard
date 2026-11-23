package dev.patrickgold.florisboard.lib.crashutility

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.CrashFact
import com.jtn0123.audiowhisper.rust.CrashRules
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Seeded traces holding what a crash leaves behind, now Rust's ([afterCrash] over
 * utilities::crash), to main's ([ReferenceCrash]): crashes at moving times (a few a second, some
 * minutes apart, the clock set back now and then), the crash screen reading and deleting the
 * reports between them, and the app's data cleared, every crash a process that died. Every call to
 * Android is an effect, in order. Also the crash screen's report, against main's, and the handler
 * with the library unavailable. Texts are synthetic and compared, never printed.
 */
class CrashTraceTest {
    /** The app's crash files and notifications: reports are read back line by line, as readFile does. */
    private class FakeHost : CrashHost {
        val effects = mutableListOf<String>()
        val files = HashMap<Long, String>()
        var last = 0L

        override fun lastCrash() = last.also { effects += "last" }
        override fun setLastCrash(timestamp: Long) {
            effects += "set $timestamp"
            last = timestamp
        }

        override fun report(timestamp: Long): String {
            effects += "read $timestamp"
            return files[timestamp]?.let(::readBack).orEmpty()
        }

        override fun forget(timestamp: Long) {
            effects += "forget $timestamp ${files.remove(timestamp) != null}"
        }

        override fun notifyOnce() {
            effects += "once"
        }

        override fun notifyLoop() {
            effects += "loop"
        }

        /** readFile: each line, as BufferedReader splits them, then a line separator. */
        private fun readBack(text: String): String {
            if (text.isEmpty()) return ""
            val lines = text.split(Regex("\r\n|\n|\r"))
            val ended = text.endsWith("\n") || text.endsWith("\r")
            return (if (ended) lines.dropLast(1) else lines).joinToString("") { it + "\n" }
        }
    }

    private fun trace(seed: Long, reference: Boolean): List<String> {
        val random = Random(seed)
        val host = FakeHost()
        var now = 1_700_000_000_000L
        repeat(EVENTS) {
            when (random.nextInt(20)) {
                in 0..13 -> {
                    now += when (random.nextInt(10)) {
                        0 -> -random.nextLong(0, 60_000)
                        1 -> random.nextLong(5_000, 600_000)
                        else -> random.nextLong(0, 6_000)
                    }
                    val stacktrace = TRACES[random.nextInt(TRACES.size)]
                    // The handler writes the report first, as uncaughtException does.
                    host.files[now] = stacktrace
                    host.effects += "crash"
                    if (reference) ReferenceCrash.afterCrash(host, stacktrace, now) else afterCrash(host, stacktrace, now)
                }
                in 14..17 -> host.effects += "screen read ${host.files.size}".also { host.files.clear() }
                18 -> if (random.nextInt(10) == 0) {
                    host.effects += "data cleared"
                    host.files.clear()
                    host.last = 0
                }
                else -> host.effects += "quiet"
            }
        }
        return host.effects
    }

    @Test
    fun crashesMatchMain() {
        // The reference's effects also go on a golden tape (docs/phase6-kotlin-references.md).
        val tape = GoldenTape.ofReference("crash-traces")
        for (seed in SEEDS) {
            val main = trace(seed, reference = true)
            main.forEach(tape::record)
            val new = trace(seed, reference = false)
            val first = main.indices.firstOrNull { it >= new.size || main[it] != new[it] }
            assertEquals(null, first, "seed $seed: first difference")
            assertEquals(main.size, new.size, "seed $seed: lengths")
            val seen = mapOf(
                "crashes" to main.count { it == "crash" },
                "last checked" to main.count { it.startsWith("read ") },
                "repeat dropped" to main.count { it.startsWith("forget ") && it.endsWith("true") },
                "repeat already read" to main.count { it.startsWith("forget ") && it.endsWith("false") },
                "loops" to main.count { it == "loop" },
                "once" to main.count { it == "once" },
                "clock set back" to main.filter { it.startsWith("set ") }.map { it.substring(4).toLong() }
                    .zipWithNext().count { (a, b) -> b < a },
                "data cleared" to main.count { it == "data cleared" },
            )
            println("Crash trace seed $seed: $EVENTS events, ${main.size} effects, 0 differ, seen $seen")
            for ((kind, count) in seen) assertTrue(count >= MIN_SEEN, "seed $seed saw $kind $count times")
        }
        tape.finish()
    }

    @Test
    fun reportsMatchMain() {
        val tape = GoldenTape.ofReference("crash-reports")
        for (seed in SEEDS) {
            val random = Random(seed)
            var debug = 0
            var traces = 0
            repeat(REPORTS) { n ->
                val facts = ReferenceCrash.Facts(
                    text(random), random.nextInt(), random.nextBoolean(), text(random), text(random),
                    text(random), text(random), text(random),
                )
                val stacktraces = List(random.nextInt(0, 4)) { CrashUtility.Stacktrace(text(random), text(random)) }
                val main = ReferenceCrash.report(facts, stacktraces)
                tape.record(main)
                val new = CrashRules.report(row(facts), facts.debug, stacktraces.map { it.name }.toTypedArray(),
                    stacktraces.map { it.details }.toTypedArray())
                assertEquals(main.length, new.length, "seed $seed report $n: lengths")
                assertTrue(main == new, "seed $seed report $n: same text")
                if (facts.debug) debug++
                traces += stacktraces.size
            }
            println("Crash report trace seed $seed: $REPORTS reports, $debug debug, $traces stack traces, 0 differ")
            assertTrue(debug >= MIN_SEEN && REPORTS - debug >= MIN_SEEN && traces >= MIN_SEEN)
        }
        tape.finish()
    }

    /** A report carries its inputs and fixed headings, nothing else: no text of the user's can join it. */
    @Test
    fun aReportAddsNothingButItsHeadings() {
        val facts = ReferenceCrash.Facts("V", 1, false, "C", "L", "D", "A", "G")
        val empty = CrashRules.report(row(facts), false, emptyArray(), emptyArray())
        val withTrace = CrashRules.report(row(facts), false, arrayOf("N"), arrayOf("T"))
        val skeleton = "#### Environment information\n- FlorisBoard [V](L) (1)\n- Device: D\n- Android: A\n\n" +
            "#### Attached logs and stacktrace files\n<details>\n<summary>Detailed info (Debug log header)</summary>\n\n" +
            "```\nG\n```\n</details>\n\n"
        assertEquals(skeleton, empty)
        assertEquals(skeleton + "<details>\n<summary>N</summary>\n\n```\nT\n```\n</details>\n\n", withTrace)
    }

    /**
     * The bootstrap contract: with the library unavailable, the handler still keeps the time and
     * notifies, keeping every report and never calling the loop. Each way the library fails to
     * load reaches the handler as a LinkageError.
     */
    @Test
    fun withoutTheLibraryEveryReportIsKeptAndOneCrashSaid() {
        val failures = listOf(UnsatisfiedLinkError("no awnative"), NoClassDefFoundError("CrashNative"),
            ExceptionInInitializerError())
        for (failure in failures) {
            val host = FakeHost().apply { last = 1_000; files[1_000] = "same\n"; files[2_000] = "same\n" }
            val unavailable = object : CrashSteps {
                override fun after(last: Long, now: Long): Int = throw failure
                override fun same(last: String, current: String): Boolean = throw failure
            }
            val thrown = runCatching { afterCrash(host, "same\n", 2_000, unavailable) }.exceptionOrNull()
            assertEquals(null, thrown, "${failure::class.simpleName} reached the handler")
            assertEquals(listOf("last", "set 2000", "once"), host.effects, failure::class.simpleName)
            assertEquals(setOf(1_000L, 2_000L), host.files.keys)
        }
    }

    private fun row(facts: ReferenceCrash.Facts): Array<String> {
        val row = Array(CrashFact.SIZE) { "" }
        row[CrashFact.VERSION_NAME] = facts.versionName
        row[CrashFact.VERSION_CODE] = facts.versionCode.toString()
        row[CrashFact.COMMIT_URL] = facts.commitUrl
        row[CrashFact.CHANGELOG_URL] = facts.changelogUrl
        row[CrashFact.DEVICE] = facts.device
        row[CrashFact.ANDROID] = facts.android
        row[CrashFact.DEBUG_LOG] = facts.debugLog
        return row
    }

    /** Synthetic text: words, line breaks, markdown, an emoji's surrogate pair, and a lone surrogate. */
    private fun text(random: Random): String = buildString {
        repeat(random.nextInt(0, 6)) { append(PIECES[random.nextInt(PIECES.size)]) }
    }

    private companion object {
        val SEEDS = longArrayOf(20261122, 7, 4_000_000_007)
        const val EVENTS = 50_000
        const val REPORTS = 20_000
        const val MIN_SEEN = 20
        val TRACES = listOf("a\n\tat b\n", "a\n\tat c\n", "x\r\n\tat y\r\n", "z", "", "a\n\tat b")
        val PIECES = listOf("alpha", " ", "\n", "```", "<summary>", "😀", "\uD800", "é", "](", "\r\n", "0")
    }
}
