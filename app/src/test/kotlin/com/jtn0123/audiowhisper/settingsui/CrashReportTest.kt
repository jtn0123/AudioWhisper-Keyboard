package com.jtn0123.audiowhisper.settingsui

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Test

class CrashReportTest {
    @Test fun nativeLoadReportOmitsEveryUnstructuredPayload() {
        val secret = "private field clipboard transcript learned word"
        val failure = UnsatisfiedLinkError(secret).apply {
            initCause(IllegalArgumentException(secret))
            addSuppressed(IllegalStateException(secret))
            stackTrace = arrayOf(
                StackTraceElement("com.jtn0123.audiowhisper.rust.NativeRuntime", "load", secret, 17),
                StackTraceElement(secret, secret, secret, 18),
            )
        }
        val report = CrashReport.nativeLoad(failure)
        assertEquals("AudioWhisper native engine unavailable\nlibrary=awnative\ncategory=UnsatisfiedLinkError\ncom.jtn0123.audiowhisper.rust.NativeRuntime.load:17", report.details)
        assertFalse(report.toString().contains(secret))
        assertEquals(dev.patrickgold.florisboard.R.string.crash__native_load_summary, report.summary)
    }
}
