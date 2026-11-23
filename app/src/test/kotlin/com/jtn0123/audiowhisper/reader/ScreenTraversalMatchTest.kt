package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.ReaderTree
import com.jtn0123.audiowhisper.rust.ReaderWalk
import kotlin.random.Random
import kotlin.test.Test

/** Seeded trees walked by the native walk, checked against the frozen Kotlin walk's answers (golden/screen-traversal-*.txt). */
class ScreenTraversalMatchTest {
    private class Tree(val fields: List<IntArray>, val children: List<List<Int>>, val texts: List<String>, val classes: List<String>) : ReaderTree {
        override fun facts(id: Int) = fields[id]
        override fun className(id: Int) = classes[id]
        override fun text(id: Int) = texts[id]
        override fun child(id: Int, index: Int) = children[id][index]
    }
    @Test fun seededTreesMatchSelectionAndReadingOrder() = GoldenTape.check("screen-traversal-seeded") { tape ->
        val visible = intArrayOf(0, 0, 1080, 1600)
        repeat(200) { seed ->
            val rng = Random(seed)
            repeat(50) {
                val count = rng.nextInt(1, 200)
                val children = List(count) { arrayListOf<Int>() }
                for (i in 1 until count) children[rng.nextInt(i)] += i
                val tree = Tree(List(count) { i ->
                    val top = rng.nextInt(-400, 1800)
                    intArrayOf(0, top, 1080, top + rng.nextInt(0, 1500),
                        if (rng.nextInt(20) == 0) 1 else 0, if (rng.nextInt(20) == 0) 1 else 0,
                        rng.nextInt(2), rng.nextInt(2), rng.nextInt(4), rng.nextInt(4), rng.nextInt(2), rng.nextInt(2), children[i].size)
                }, children, List(count) { listOf("", " ", "\u00a0", "A synthetic block.", "42 ✓")[rng.nextInt(5)] },
                    List(count) { listOf("android.webkit.WebView", "ViewPager", "TextView", "HorizontalScrollView")[rng.nextInt(4)] })
                tape.record(ReaderWalk.walk(tree, visible).joinToString(","))
            }
        }
    }
    @Test fun deepAndWideTreesPreserveTheOriginalLimits() = GoldenTape.check("screen-traversal-limits") { tape ->
        for ((count, chain) in listOf(300 to true, 20_100 to false)) {
            val children = List(count) { i -> if (chain && i + 1 < count) listOf(i + 1) else if (!chain && i == 0) (1 until count).toList() else emptyList() }
            val tree = Tree(List(count) { i -> intArrayOf(0, 0, 1080, 1600, 0, 0, 1, 0, 0, 0, 0, 0, children[i].size) },
                children, List(count) { "Synthetic block." }, List(count) { "TextView" })
            tape.record(ReaderWalk.walk(tree, intArrayOf(0, 0, 1080, 1600)).joinToString(","))
        }
    }
}
