package dev.patrickgold.florisboard.lib.crashutility

import com.jtn0123.audiowhisper.rust.CrashAfter
import com.jtn0123.audiowhisper.rust.CrashRules

/** What a crash does to the app's files and notifications: [CrashUtility]'s handler's, or a test's. */
internal interface CrashHost {
    /** When the last crash was, 0 for none. */
    fun lastCrash(): Long

    fun setLastCrash(timestamp: Long)

    /** The report of the crash at [timestamp], as read back; empty if there is none. */
    fun report(timestamp: Long): String

    /** Drops the report of the crash at [timestamp]. */
    fun forget(timestamp: Long)

    fun notifyOnce()

    /** Says crashes keep coming, and switches away from the keyboard. */
    fun notifyLoop()
}

/** Rust's crash rules, or a test's stand-in for them. */
internal interface CrashSteps {
    fun after(last: Long, now: Long): Int

    fun same(last: String, current: String): Boolean
}

private object RustCrashSteps : CrashSteps {
    override fun after(last: Long, now: Long) = CrashRules.after(last, now)

    override fun same(last: String, current: String) = CrashRules.same(last, current)
}

/**
 * After a crash at [timestamp] whose report, [stacktrace], is written: the last crash's report
 * dropped if it is this one, the time kept, and a notification, of a loop if they come too fast.
 * Those choices are Rust's (utilities::crash). The handler is only installed once the library has
 * loaded, but a crash may be the library failing; if it can't be called, every report is kept and
 * the notification is of one crash, which is always safe.
 */
internal fun afterCrash(host: CrashHost, stacktrace: String, timestamp: Long, rules: CrashSteps = RustCrashSteps) {
    val last = host.lastCrash()
    val steps = try {
        rules.after(last, timestamp)
    } catch (_: LinkageError) {
        0
    }
    if (steps and CrashAfter.CHECK_LAST != 0 && rules.same(host.report(last), stacktrace)) host.forget(last)
    host.setLastCrash(timestamp)
    if (steps and CrashAfter.LOOP != 0) host.notifyLoop() else host.notifyOnce()
}
