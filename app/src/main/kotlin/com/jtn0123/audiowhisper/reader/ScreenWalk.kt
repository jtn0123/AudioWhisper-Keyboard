package com.jtn0123.audiowhisper.reader

import android.graphics.Rect
import android.view.accessibility.AccessibilityNodeInfo
import android.view.accessibility.AccessibilityNodeInfo.AccessibilityAction

/**
 * What a walk of one window found: the app it belongs to, its text in reading order, the list that
 * scrolls (null if nothing does), the part of the screen the app shows above the keyboard, how
 * many nodes the walk looked at, and whether there is a web page in it.
 */
internal class Screen(
    val app: String?,
    val blocks: List<TextBlock>,
    val scroller: AccessibilityNodeInfo?,
    val visibleTop: Int,
    val visibleBottom: Int,
    val nodes: Int,
    val web: Boolean,
) {
    /** Every block is in sight: a short page, or a web page whose tree is still being built. */
    val allShown: Boolean get() = PageText.allShown(blocks, visibleTop, visibleBottom)
}

/**
 * Walks a window's accessibility tree for its text. Fields being typed in and passwords are left
 * out, and so are labels only an icon carries ("Back", "More options"), which aren't page text.
 * A node with text and children is read through its children: a web page puts its title on its
 * root, and a link's text both on the link and on the text inside it.
 *
 * The list that scrolls is the one going up and down that holds the most of the text; a carousel
 * or a pager going sideways is never it, and its pages out of sight are skipped.
 */
internal class ScreenWalk(private val visible: Rect) {
    private class AndroidTree(root: AccessibilityNodeInfo) : com.jtn0123.audiowhisper.rust.ReaderTree {
        val nodes = arrayListOf(root)
        val texts = mutableMapOf<Int, String>()
        override fun facts(id: Int): IntArray {
            val node = nodes[id]
            val bounds = Rect().also(node::getBoundsInScreen)
            val collection = node.collectionInfo
            val actions = node.actionList.map { it.id }
            fun Boolean.bit() = if (this) 1 else 0
            return intArrayOf(bounds.left, bounds.top, bounds.right, bounds.bottom,
                node.isPassword.bit(), node.isEditable.bit(), node.isVisibleToUser.bit(), node.isScrollable.bit(),
                collection?.rowCount ?: 0, collection?.columnCount ?: 0,
                actions.any { it in UP_DOWN }.bit(), actions.any { it in ACROSS }.bit(), node.childCount)
        }
        override fun className(id: Int): String = nodes[id].className?.toString().orEmpty()
        override fun text(id: Int): String = nodes[id].text?.toString().orEmpty().also { texts[id] = it }
        override fun child(id: Int, index: Int): Int {
            val child = nodes[id].getChild(index) ?: return -1
            nodes += child
            return nodes.lastIndex
        }
    }
    fun walk(root: AccessibilityNodeInfo): Screen {
        val tree = AndroidTree(root)
        val packet = com.jtn0123.audiowhisper.rust.ReaderWalk.walk(tree,
            intArrayOf(visible.left, visible.top, visible.right, visible.bottom))
        val list = tree.nodes.getOrNull(packet[0])
        val part = list?.let { ScreenRules.visiblePart(Rect().also(it::getBoundsInScreen).box(), visible.box()) }
        val follow = FollowAlong(list, part?.let { Rect(it.left, it.top, it.right, it.bottom) } ?: visible)
        val pieces = packet.drop(3).chunked(4).map { (id, top, bottom, inList) ->
            TextBlock(checkNotNull(tree.texts[id]), top, bottom, inList != 0) {
                if (inList != 0) follow.show(tree.nodes[id])
            }
        }
        return Screen(root.packageName?.toString(), PageText.blocks(pieces), list,
            visible.top, visible.bottom, packet[1], packet[2] != 0)
    }
    private fun Rect.box() = Box(left, top, right, bottom)
    private companion object {
        val UP_DOWN = setOf(AccessibilityAction.ACTION_SCROLL_UP.id, AccessibilityAction.ACTION_SCROLL_DOWN.id,
            AccessibilityAction.ACTION_PAGE_UP.id, AccessibilityAction.ACTION_PAGE_DOWN.id)
        val ACROSS = setOf(AccessibilityAction.ACTION_SCROLL_LEFT.id, AccessibilityAction.ACTION_SCROLL_RIGHT.id,
            AccessibilityAction.ACTION_PAGE_LEFT.id, AccessibilityAction.ACTION_PAGE_RIGHT.id)
    }
}
