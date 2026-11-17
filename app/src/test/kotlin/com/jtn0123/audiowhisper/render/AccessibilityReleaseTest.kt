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
            AccessibilityTarget(3, intArrayOf(0, 0, 100, 100), false, 0),
            AccessibilityTarget(101, intArrayOf(0, -60, 50, 20), true, 1),
        )
        assertEquals(101, accessibilityHitTarget(25f, -20f, 100, 100, targets))
        assertEquals(101, accessibilityHitTarget(25f, 10f, 100, 100, targets))
        assertNull(accessibilityHitTarget(75f, -20f, 100, 100, targets))
        assertNull(accessibilityHitTarget(25f, -61f, 100, 100, targets))
    }

    @Test fun touchFindsTheTargetDrawnOnTopNotTheLastOneRead() {
        // Reading order puts a key's popup right after the key, before the keys it covers.
        val key = AccessibilityTarget(3, intArrayOf(0, 50, 50, 100), false, 0)
        val popup = AccessibilityTarget(101, intArrayOf(0, 0, 100, 60), true, 4)
        val covered = AccessibilityTarget(5, intArrayOf(50, 0, 100, 50), false, 2)
        val targets = listOf(key, popup, covered)
        assertEquals(101, accessibilityHitTarget(75f, 25f, 100, 100, targets))
        assertEquals(101, accessibilityHitTarget(25f, 55f, 100, 100, targets))
        assertEquals(3, accessibilityHitTarget(25f, 75f, 100, 100, targets))
        // Equal layers keep the old rule: the later target wins.
        val tied = listOf(covered, covered.copy(id = 6, layer = 2))
        assertEquals(6, accessibilityHitTarget(75f, 25f, 100, 100, tied))
    }

    @Test fun nonPopupTargetsNeverExtendTheAccessibleViewport() {
        val targets = listOf(AccessibilityTarget(3, intArrayOf(-30, -30, 130, 130), false, 0))
        assertNull(accessibilityHitTarget(25f, -20f, 100, 100, targets))
        assertEquals(3, accessibilityHitTarget(25f, 20f, 100, 100, targets))
    }
}
