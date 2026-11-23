package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.ScreenWalkRules
import kotlin.random.Random

/** A rectangle on the screen, in pixels, as Android's `Rect` has it. */
internal data class Box(val left: Int, val top: Int, val right: Int, val bottom: Int) {
    /** How much of this shows in [visible], in square pixels. */
    fun areaIn(visible: Box): Long = ScreenWalkRules.visibleArea(sides(visible))

    /** The part of this that shows in [visible], or all of [visible] if none does. */
    fun partIn(visible: Box): Box = ScreenWalkRules.visiblePart(sides(visible)).let { Box(it[0], it[1], it[2], it[3]) }

    private fun sides(visible: Box) =
        intArrayOf(left, top, right, bottom, visible.left, visible.top, visible.right, visible.bottom)
}

/**
 * Asks the screen walk's rules in Rust (lib/rust/crates/reader/src/screen.rs, behind
 * [ScreenWalkRules]) the questions the Kotlin they replaced answered, and records the answers on
 * [tape], whose golden file holds that Kotlin's answers (docs/phase6-kotlin-references.md): random
 * scrollable views and their class names, boxes on and off the screen and at Int's ends, trees of
 * scrollers holding pieces of text, and pieces to bring into view.
 */
class ScreenComparison(private val tape: GoldenTape) {
    /** How often each outcome worth reaching was reached, so a run that tests nothing shows. */
    val seen = sortedMapOf<String, Int>()

    private fun check(what: String, answer: Any?) = tape.record("$what: $answer")

    private fun see(what: String, happened: Boolean = true) {
        if (happened) seen[what] = (seen[what] ?: 0) + 1
    }

    /** The constants the walk shares with the Rust. */
    fun constants() = check("NONE", ScreenWalkRules.NONE)

    /** Random scrollable views: whether each goes sideways. */
    fun sideways(random: Random, count: Int) = repeat(count) {
        val rows = SIZES.random(random)
        val columns = SIZES.random(random)
        val upDown = random.nextBoolean()
        val across = random.nextBoolean()
        val name = NAMES.random(random) + if (random.nextInt(4) == 0) NAMES.random(random) else ""
        val rust = ScreenWalkRules.isSideways(rows, columns, upDown, across, name)
        check("isSideways", rust)
        see("sideways by its collection", rows == 1 && columns > 1)
        see("sideways by its scrolling", rows != 1 && !upDown && across)
        see("sideways by its name", rust && rows != 1 && !upDown && !across)
        see("not sideways", !rust)
    }

    /** Random boxes and the part of the screen in sight: how much of each shows, and which part. */
    fun boxes(random: Random, count: Int) = repeat(count) {
        val bounds = box(random)
        val visible = box(random)
        val area = bounds.areaIn(visible)
        check("visibleArea", area)
        check("visiblePart", bounds.partIn(visible))
        val overlap = overlap(bounds, visible)
        see("overlapping", overlap != null && area > 0)
        see("overlapping with no area", overlap != null && area == 0L)
        see("apart", overlap == null)
        see("an area that wraps", area < 0)
        see("a width that wraps", overlap != null && overlap.right.toLong() - overlap.left > Int.MAX_VALUE)
    }

    /** Random trees of scrollers holding pieces of text: which is the list, and what is in it. */
    fun lists(random: Random, count: Int) = repeat(count) {
        val scrollers = random.nextInt(MAX_SCROLLERS + 1)
        val parents = IntArray(scrollers) { random.nextInt(-1, it) }
        val areas = LongArray(scrollers) { AREAS.random(random) }
        val pieces = IntArray(random.nextInt(MAX_PIECES + 1)) { random.nextInt(-1, scrollers) }
        val main = ScreenWalkRules.mainScroller(pieces, areas)
        check("mainScroller", main)
        val inList = ScreenWalkRules.inList(pieces, parents, main).toList()
        check("inList", inList)
        see("no list", main == ScreenWalkRules.NONE)
        see("not the biggest", main != ScreenWalkRules.NONE && areas.any { it > areas[main] })
        see("a piece in the list", pieces.indices.any { inList[it] && pieces[it] == main })
        see("a piece in a list inside it", pieces.indices.any { inList[it] && pieces[it] != main })
        see("a piece outside it", main != ScreenWalkRules.NONE && false in inList)
    }

    /** Random pieces to bring into view, web pages' and apps' own. */
    fun moves(random: Random, count: Int) = repeat(count) {
        val (first, last, top, bottom) = List(4) { number(random) }
        val canTurn = random.nextBoolean()
        val move = ScreenWalkRules.move(first, last, top, bottom, canTurn)
        check("move", move.name)
        see("move ${move.name}")
        val after = ScreenWalkRules.afterTurn(first, last, top, bottom)
        check("afterTurn", after.name)
        see("after a turn ${after.name}")
        val inside = ScreenWalkRules.isInside(first, last, top, bottom)
        check("isInside", inside)
        see(if (inside) "well inside" else "not well inside")
    }

    /** Where [a] and [b] overlap, as `Rect.intersect` finds it, to say which case a box was. */
    private fun overlap(a: Box, b: Box): Box? =
        if (a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom) {
            Box(maxOf(a.left, b.left), maxOf(a.top, b.top), minOf(a.right, b.right), minOf(a.bottom, b.bottom))
        } else {
            null
        }

    /** A number for a piece or a view: mostly near the screen, sometimes at Int's ends. */
    private fun number(random: Random): Int =
        if (random.nextInt(8) == 0) EDGES.random(random) else random.nextInt(-200, 2_600)

    private fun box(random: Random): Box {
        val top = number(random)
        if (random.nextInt(WIDE_ONE_IN) == 0) {
            // As wide as Int goes, so where two overlap their width wraps.
            return Box(Int.MIN_VALUE + random.nextInt(3), top, Int.MAX_VALUE - random.nextInt(3), top + random.nextInt(1, 2_000))
        }
        val left = number(random)
        if (random.nextInt(6) == 0) return Box(left, top, number(random), number(random))
        val width = if (random.nextInt(10) == 0) 0 else random.nextInt(1, 1_200)
        val height = if (random.nextInt(10) == 0) 0 else random.nextInt(1, 2_000)
        return Box(left, top, left + width, top + height)
    }

    private companion object {
        const val MAX_SCROLLERS = 6
        const val MAX_PIECES = 12

        /** One box in this many spans all of Int's width. */
        const val WIDE_ONE_IN = 12

        val SIZES = listOf(0, 0, 0, 1, 1, 1, 2, 5, -1, Int.MIN_VALUE, Int.MAX_VALUE)
        val EDGES = listOf(0, 1, -1, Int.MAX_VALUE, Int.MIN_VALUE, Int.MAX_VALUE - 1, Int.MIN_VALUE + 1)
        val AREAS = listOf(0L, 0L, -1L, 1L, 500L, 800L, 800L, 1_000L, 2_000_000L, Long.MAX_VALUE, Long.MIN_VALUE)
        val NAMES = listOf(
            "", "RecyclerView", "android.widget.ScrollView", "android.widget.HorizontalScrollView",
            "HorizontalScrollView", "HorizontalScrollViewX", "horizontalscrollview", "ViewPager",
            "androidx.viewpager2.widget.ViewPager2", "viewpager", "View Pager", "Pager", "ScrollView",
            "😀ViewPager", "İ", "\uD800", "android.webkit.WebView",
        )
    }
}
