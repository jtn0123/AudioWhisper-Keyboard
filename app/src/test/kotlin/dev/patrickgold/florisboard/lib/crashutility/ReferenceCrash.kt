package dev.patrickgold.florisboard.lib.crashutility

/**
 * Main's crash handling (b1cf45f18): UncaughtExceptionHandler.uncaughtException's choices after
 * the report is written, and CrashDialogActivity's report, verbatim apart from the calls to
 * Android:
 *
 * - `getLastCrashTimestamp(application)`, `setLastCrashTimestamp(application, t)`:
 *   `host.lastCrash()`, `host.setLastCrash(t)`;
 * - `readFile(application.getUstFile(t))` and that file's `delete()`: `host.report(t)`,
 *   `host.forget(t)`;
 * - `pushCrashMultipleNotification` then the switch away from the keyboard: `host.notifyLoop()`,
 *   and `pushCrashOnceNotification`: `host.notifyOnce()`;
 * - the report's `BuildConfig`, string resources and `Devtools` values: parameters, and its
 *   `flogWarning` for no stack traces is left out (logs are not the report).
 */
internal object ReferenceCrash {
    fun afterCrash(host: CrashHost, stacktrace: String, timestamp: Long) {
        val lastTimestamp = host.lastCrash()
        if (lastTimestamp > 0) {
            val lastStacktrace = host.report(lastTimestamp)
            if (lastStacktrace == stacktrace) {
                // Delete last stacktrace if it matches previous unhandled one
                host.forget(lastTimestamp)
            }
        }
        host.setLastCrash(timestamp)
        if (timestamp - lastTimestamp < 5000) {
            host.notifyLoop()
        } else {
            host.notifyOnce()
        }
    }

    class Facts(
        val versionName: String,
        val versionCode: Int,
        val debug: Boolean,
        val commitUrl: String,
        val changelogUrl: String,
        val device: String,
        val android: String,
        val debugLog: String,
    )

    fun report(facts: Facts, stacktraces: List<CrashUtility.Stacktrace>): String {
        val errorReport = StringBuilder()
        val versionName = buildString {
            append("[")
            append(facts.versionName)
            append("](")
            if (facts.debug) {
                append(facts.commitUrl)
            } else {
                append(facts.changelogUrl)
            }
            append(")")
        }
        errorReport.apply {
            appendLine("#### Environment information")
            appendLine("- FlorisBoard $versionName (${facts.versionCode})")
            appendLine("- Device: ${facts.device}")
            appendLine("- Android: ${facts.android}")
            appendLine()
            appendLine("#### Attached logs and stacktrace files")
            appendCollapsibleSection(
                summary = "Detailed info (Debug log header)",
                details = facts.debugLog,
            )
            appendLine()
            if (stacktraces.isNotEmpty()) {
                stacktraces.forEach {
                    appendCollapsibleSection(it.name, it.details)
                    appendLine()
                }
            }
        }
        return errorReport.toString()
    }

    private fun StringBuilder.appendCollapsibleSection(summary: String, details: String) {
        this.appendLine("<details>")
        this.append("<summary>").append(summary).appendLine("</summary>")
        this.appendLine()
        this.appendLine("```")
        this.appendLine(details)
        this.appendLine("```")
        this.appendLine("</details>")
    }
}
