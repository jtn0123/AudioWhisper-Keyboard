package dev.patrickgold.florisboard.ime.text.gestures

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The swipe and glide detectors in Rust ([SwipeGesture.Detector] and [GlideTypingGesture.Detector],
 * which call lib/rust/crates/gestures) do exactly what the Kotlin they replaced did
 * ([ReferenceSwipeDetector] and [ReferenceGlideDetector]): random runs of touches with several
 * fingers in any order, at the thresholds' edges, on the edges between directions, at odd densities,
 * velocities and preferences, and across a clock that wraps.
 */
class GesturesMatchTest {
    private fun assertSeen(seen: Map<String, Int>, outcomes: List<String>) {
        for (outcome in outcomes) {
            assertTrue((seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${seen[outcome]} times, of $seen")
        }
    }

    @Test
    fun swipesMatch() {
        val comparison = SwipeGestureComparison()
        comparison.swipes(Random(SEED), runs = 3_000, steps = 60)
        assertEquals(emptyList(), comparison.differences)
        val directions = SwipeGesture.Direction.entries
        assertSeen(comparison.seen, listOf("move none", "up none") + directions.flatMap { listOf("move $it", "up $it") })
    }

    @Test
    fun edgesBetweenDirectionsMatch() {
        val comparison = SwipeGestureComparison()
        comparison.edges()
        assertEquals(emptyList(), comparison.differences)
        // Each of the eight reflections, at four densities and four scales.
        assertEquals(8 * 16, comparison.seen.values.sum())
    }

    @Test
    fun glidesMatch() {
        val comparison = GlideGestureComparison()
        comparison.glides(Random(SEED), runs = 3_000, steps = 40)
        assertEquals(emptyList(), comparison.differences)
        assertSeen(comparison.seen, GLIDE_OUTCOMES)
    }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        val GLIDE_OUTCOMES = listOf(
            "down: first",
            "down: followed",
            "down: ignored",
            "move: not followed",
            "move: a glide",
            "move: too slow",
            "move: undecided",
            "move: undecided from a swipe key",
            "move: decided true",
            "move: decided false",
            "up: not followed",
            "up: glide complete",
            "up: no glide",
            "cancel: glide",
            "cancel: no glide",
            "other",
        )
    }
}
