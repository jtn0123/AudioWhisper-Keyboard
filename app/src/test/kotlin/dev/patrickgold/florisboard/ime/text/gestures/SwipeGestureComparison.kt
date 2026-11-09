package dev.patrickgold.florisboard.ime.text.gestures

import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin
import kotlin.random.Random

/**
 * Puts the swipe detector in Rust ([SwipeGesture.Detector]) up against the Kotlin it replaced,
 * [ReferenceSwipeDetector], wherever this runs: the JVM's tests and, copied in for a check, the
 * phone. It records each difference and how often each outcome was seen, so a caller can tell the
 * runs reached them all.
 */
class SwipeGestureComparison {
    val differences = mutableListOf<String>()
    val seen = mutableMapOf<String, Int>()

    private fun check(what: String, input: String, kotlin: Any?, rust: Any?) {
        if (kotlin != rust && differences.size < MAX_DIFFERENCES) differences += "$what($input): kotlin $kotlin, rust $rust"
    }

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    /** Records each swipe, and answers as the keyboard might: true for two in three. */
    private class Recorder : SwipeGesture.Listener {
        val events = mutableListOf<SwipeGesture.Event>()

        override fun onSwipe(event: SwipeGesture.Event): Boolean {
            events += event
            return events.size % 3 != 0
        }
    }

    /** The settings a run swipes with: the screen's density and the two preferences. */
    private class Settings(random: Random) {
        val density = DENSITIES.random(random)
        val distance = DISTANCES.random(random)
        val velocity = VELOCITIES.random(random)

        override fun toString() = "density $density, distance $distance, velocity $velocity"
    }

    /**
     * Random runs of fingers going down, moving, lifting and cancelled, several at once and out of
     * order, with moves around the swipe distance and on the edges between directions.
     */
    fun swipes(random: Random, runs: Int, steps: Int) = repeat(runs) { run ->
        val kotlinListener = Recorder()
        val rustListener = Recorder()
        val kotlin = ReferenceSwipeDetector(kotlinListener)
        val rust = SwipeGesture.Detector(rustListener)
        var settings = Settings(random)
        val at = mutableMapOf<Int, Pair<Float, Float>>()
        repeat(steps) { step ->
            val input = "run $run step $step, $settings"
            val id = ID.random(random)
            when (random.nextInt(20)) {
                0 -> {
                    kotlin.onTouchEvent(ACTION_DOWN)
                    rust.reset()
                    at.clear()
                }
                1 -> settings = Settings(random)
                in 2..5 -> {
                    val start = start(random)
                    at[id] = start
                    kotlin.onTouchDown(id, 0, start.first, start.second, settings.density)
                    rust.down(id, start.first, start.second, settings.density)
                }
                in 6..14 -> {
                    val (x, y) = moved(random, at[id] ?: start(random), settings).also { at[id] = it }
                    val always = random.nextInt(4) == 0
                    val before = kotlinListener.events.size
                    val kotlinAnswer = kotlin.onTouchMove(id, 0, x, y, settings.density, settings.distance, always)
                    val rustAnswer = rust.move(id, x, y, settings.density, settings.distance, always)
                    check("move", "$input, id $id to ($x, $y), always $always", kotlinAnswer, rustAnswer)
                    see(outcome("move", kotlinListener, before))
                }
                in 15..17 -> {
                    val (x, y) = moved(random, at[id] ?: start(random), settings)
                    val (velocityX, velocityY) = velocity(random, settings)
                    at.remove(id)
                    val before = kotlinListener.events.size
                    val kotlinAnswer = kotlin.onTouchUp(
                        id, x, y, velocityX, velocityY, settings.density, settings.distance, settings.velocity,
                    )
                    val rustAnswer = rust.up(
                        id, x, y, velocityX, velocityY, settings.density, settings.distance, settings.velocity,
                    )
                    check("up", "$input, id $id at ($x, $y), velocity ($velocityX, $velocityY)", kotlinAnswer, rustAnswer)
                    see(outcome("up", kotlinListener, before))
                }
                else -> {
                    at.remove(id)
                    kotlin.onTouchCancel(id)
                    rust.cancel(id)
                }
            }
            check("swipes", input, kotlinListener.events.size, rustListener.events.size)
            check("last swipe", input, kotlinListener.events.lastOrNull(), rustListener.events.lastOrNull())
        }
    }

    /**
     * Swipes whose angle lands on the edge between two directions, or one unit in the last place
     * from it, at densities and scales that keep it there: the reflections of the one pair of
     * floats a search of 10^10 found. A sector takes its lower edge.
     */
    fun edges() {
        val (a, b) = EDGE
        val reflections = listOf(a to b, b to a).flatMap { (x, y) -> listOf(x to y, -x to y, x to -y, -x to -y) }
        for ((x, y) in reflections) for (density in listOf(1f, 2f, 4f, 0.5f)) for (scale in listOf(1f, 0.5f, 0.25f, 2f)) {
            val kotlinListener = Recorder()
            val rustListener = Recorder()
            val kotlin = ReferenceSwipeDetector(kotlinListener)
            val rust = SwipeGesture.Detector(rustListener)
            val (px, py) = x * scale * density to y * scale * density
            kotlin.onTouchDown(0, 0, 0f, 0f, density)
            rust.down(0, 0f, 0f, density)
            val input = "($px, $py) at density $density"
            check("edge move", input, kotlin.onTouchMove(0, 0, px, py, density, 32, true), rust.move(0, px, py, density, 32, true))
            check("edge up", input, kotlin.onTouchUp(0, px, py, 1e4f, 0f, density, 32, 1900), rust.up(0, px, py, 1e4f, 0f, density, 32, 1900))
            check("edge swipes", input, kotlinListener.events, rustListener.events)
            see("edge ${kotlinListener.events.first().direction}")
        }
    }

    /** What a step did: the direction of the swipe it reported, or none. */
    private fun outcome(type: String, listener: Recorder, before: Int) =
        "$type ${if (listener.events.size > before) listener.events.last().direction else "none"}"

    private fun start(random: Random): Pair<Float, Float> = when (random.nextInt(10)) {
        0 -> 0f to 0f
        1 -> random.nextFloat() * 1e6f to -random.nextFloat() * 1e6f
        else -> random.nextFloat() * 1_500f to random.nextFloat() * 700f
    }

    /** A move from [from] (pixels): any way, as far as a few swipe distances or exactly half one. */
    private fun moved(random: Random, from: Pair<Float, Float>, settings: Settings): Pair<Float, Float> {
        val half = settings.distance.toFloat() / 2f
        val (dx, dy) = when (random.nextInt(16)) {
            // Exactly half the swipe distance along an axis, and a few units of the last place either side.
            0, 1 -> {
                val length = nudge(random, half) * sign(random)
                if (random.nextBoolean()) length to 0f else 0f to length
            }
            // On an edge between two directions, a hair either side.
            2, 3, 4 -> {
                val angle = (2 * random.nextInt(8) + 1) * PI / 8
                val length = settings.distance * (0.5 + 3 * random.nextDouble())
                (length * cos(angle)).toFloat() to nudge(random, (length * sin(angle)).toFloat())
            }
            // Along a diagonal or an axis.
            5 -> {
                val length = settings.distance * (0.5 + 3 * random.nextDouble()).toFloat()
                (random.nextInt(3) - 1) * length to (random.nextInt(3) - 1) * length
            }
            6 -> ODD.random(random) to ODD.random(random)
            7 -> 0f to 0f
            else -> {
                val angle = random.nextDouble() * 2 * PI
                val length = settings.distance * 2.5 * random.nextDouble()
                (length * cos(angle)).toFloat() to (length * sin(angle)).toFloat()
            }
        }
        return when (random.nextInt(4)) {
            // Straight from where the finger is, in dp at its density.
            0 -> from.first + dx * settings.density to from.second + dy * settings.density
            else -> (from.first / settings.density + dx) * settings.density to (from.second / settings.density + dy) * settings.density
        }
    }

    /** A velocity (pixels a second): around the swipe velocity, still, either way, or odd. */
    private fun velocity(random: Random, settings: Settings): Pair<Float, Float> {
        fun one(): Float = when (random.nextInt(8)) {
            0 -> 0f
            1, 2 -> nudge(random, settings.velocity.toFloat()) * settings.density * sign(random)
            3 -> ODD.random(random)
            else -> (random.nextFloat() - 0.5f) * 3f * settings.velocity * settings.density
        }
        return one() to one()
    }

    private fun sign(random: Random) = if (random.nextBoolean()) 1f else -1f

    /** [value], or a few units of the last place either side of it. */
    private fun nudge(random: Random, value: Float): Float {
        var nudged = value
        repeat(random.nextInt(4)) { nudged = if (random.nextBoolean()) Math.nextUp(nudged) else Math.nextDown(nudged) }
        return nudged
    }

    private companion object {
        const val MAX_DIFFERENCES = 20
        const val ACTION_DOWN = 0
        val EDGE = 1952.4448f to 808.7291f
        val ID = listOf(0, 0, 0, 1, 1, 2, 3, 4, 5, -1, -2, 9)
        val DENSITIES = listOf(1f, 1f, 2f, 2f, 4f, 0.75f, 1.5f, 2.625f, 2.75f, 3f, 3.5f, 0.1f, 0f, -2f, Float.NaN)
        val DISTANCES = listOf(32, 32, 32, 12, 20, 48, 72, 1, 0, -32, 16_777_219, Int.MAX_VALUE)
        val VELOCITIES = listOf(1900, 1900, 400, 1000, 4000, 0, -1, Int.MAX_VALUE, Int.MIN_VALUE)
        val ODD = listOf(
            Float.NaN, Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY, Float.MAX_VALUE, -Float.MAX_VALUE,
            Float.MIN_VALUE, -0f, 1e30f, -1e30f, 3e9f,
        )
    }
}
