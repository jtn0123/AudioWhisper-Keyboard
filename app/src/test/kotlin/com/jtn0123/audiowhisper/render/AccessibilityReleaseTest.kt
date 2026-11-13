package com.jtn0123.audiowhisper.render

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class AccessibilityReleaseTest {
    @Test fun heldSourceReleaseSelectsTheDefaultPopupInsteadOfLeavingItOpen() {
        assertEquals(100, accessibilityReleaseTarget(3, 3, true, true, false, 100))
    }

    @Test fun dragToAnAccentSelectsThatChoice() {
        assertEquals(101, accessibilityReleaseTarget(3, 101, true, true, true, 100))
    }

    @Test fun leavingTheKeyboardCancelsAHold() {
        assertNull(accessibilityReleaseTarget(3, -1, true, true, false, 100))
        assertNull(accessibilityReleaseTarget(3, 9, true, true, false, 100))
    }

    @Test fun shiftLongClickMustNotAlsoClickOnRelease() {
        assertNull(accessibilityReleaseTarget(3, 3, true, false, false, null))
    }

    @Test fun ordinaryAccessibleTapCommitsExactlyOnce() {
        assertEquals(3, accessibilityReleaseTarget(3, 3, false, false, false, null))
        assertNull(accessibilityReleaseTarget(-1, -1, false, false, false, null))
    }
    @Test fun popupDragCanSelectAnAccentAboveTheHostRectangle() {
        val targets = listOf(
            AccessibilityTarget(3, intArrayOf(0, 0, 100, 100), false),
            AccessibilityTarget(101, intArrayOf(0, -60, 50, 20), true),
        )
        assertEquals(101, accessibilityHitTarget(25f, -20f, 100, 100, targets))
        assertEquals(101, accessibilityHitTarget(25f, 10f, 100, 100, targets))
        assertNull(accessibilityHitTarget(75f, -20f, 100, 100, targets))
        assertNull(accessibilityHitTarget(25f, -61f, 100, 100, targets))
    }

    @Test fun nonPopupTargetsNeverExtendTheAccessibleViewport() {
        val targets = listOf(AccessibilityTarget(3, intArrayOf(-30, -30, 130, 130), false))
        assertNull(accessibilityHitTarget(25f, -20f, 100, 100, targets))
        assertEquals(3, accessibilityHitTarget(25f, 20f, 100, 100, targets))
    }
}
