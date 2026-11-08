package dev.patrickgold.florisboard.lib.util

import java.util.Locale
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals

class UnitUtilsTest {
    private val before = Locale.getDefault()

    @AfterTest
    fun restoreLocale() = Locale.setDefault(before)

    @Test
    fun aSizeUnderOneKibIsInBytes() {
        assertEquals("512 bytes", UnitUtils.formatMemorySize(512))
    }

    @Test
    fun largerSizesTakeTheirUnit() {
        assertEquals("1.50 KiB", UnitUtils.formatMemorySize(1_536))
        assertEquals("2.00 MiB", UnitUtils.formatMemorySize(2L * 1024 * 1024))
        assertEquals("3.00 GiB", UnitUtils.formatMemorySize(3L * 1024 * 1024 * 1024))
    }

    @Test
    fun theDecimalPointDoesNotFollowThePhonesLanguage() {
        Locale.setDefault(Locale.GERMANY)
        assertEquals("1.50 KiB", UnitUtils.formatMemorySize(1_536))
    }
}
