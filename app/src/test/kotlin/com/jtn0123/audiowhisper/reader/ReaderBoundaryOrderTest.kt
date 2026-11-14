package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.ReaderTree
import com.jtn0123.audiowhisper.rust.ReaderWalk
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertSame

/** Freeze the actual pre-consolidation bridge's reads, including reads the older Kotlin walk skipped. */
class ReaderBoundaryOrderTest {
    private open class Tree : ReaderTree {
        val reads = mutableListOf<String>()
        override fun facts(id: Int): IntArray {
            reads += "facts$id"
            return intArrayOf(0, 0, 100, 100, if (id == 1) 1 else 0, 0, 1, 0, 0, 0, 0, 0, if (id == 0) 2 else 0)
        }
        override fun className(id: Int): String { reads += "class$id"; return "TextView" }
        override fun text(id: Int): String { reads += "text$id"; return if (id == 0) "synthetic root" else "" }
        override fun child(id: Int, index: Int): Int { reads += "child$id:$index"; return index + 1 }
    }
    private val visible = intArrayOf(0, 0, 100, 100)

    @Test fun passwordPruningAndParentFallbackPreserveEveryRead() {
        val tree = Tree()
        assertContentEquals(intArrayOf(-1, 3, 0, 0, 0, 100, 0), ReaderWalk.walk(tree, visible))
        assertEquals(listOf("facts0", "class0", "child0:0", "facts1", "class1", "child0:1",
            "facts2", "class2", "text2", "text0"), tree.reads)
    }

    @Test fun platformFailuresStopAtTheSameReadAndKeepTheOriginalException() {
        for (failClass in listOf(false, true)) {
            val marker = IllegalStateException("synthetic platform failure")
            val tree = object : Tree() {
                override fun facts(id: Int): IntArray = super.facts(id).also { if (id == 1 && !failClass) throw marker }
                override fun className(id: Int): String = super.className(id).also { if (id == 1 && failClass) throw marker }
            }
            assertSame(marker, assertFailsWith<IllegalStateException> { ReaderWalk.walk(tree, visible) })
            assertEquals(listOf("facts0", "class0", "child0:0", "facts1") + if (failClass) listOf("class1") else emptyList(), tree.reads)
        }
    }

    @Test fun invalidFactsFailBeforeAskingForClassOrText() {
        val tree = object : Tree() {
            override fun facts(id: Int): IntArray = super.facts(id).let { if (id == 1) it.copyOf(12) else it }
        }
        assertEquals("Node facts length", assertFailsWith<IllegalArgumentException> { ReaderWalk.walk(tree, visible) }.message)
        assertEquals(listOf("facts0", "class0", "child0:0", "facts1"), tree.reads)
    }
}
