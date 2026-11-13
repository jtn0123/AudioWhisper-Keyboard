package org.florisboard.lib.snygg

import org.florisboard.lib.snygg.value.SnyggValue
import java.util.ConcurrentModificationException

/** Managed object storage. Rust owns insertion/replacement/unset slot selection. */
internal class SnyggEditorProperties : AbstractMutableMap<String, SnyggValue>() {
    private class Entry(override val key: String, initial: SnyggValue) : MutableMap.MutableEntry<String, SnyggValue> {
        private var current = initial
        override val value get() = current
        override fun setValue(newValue: SnyggValue): SnyggValue = current.also { current = newValue }
        override fun equals(other: Any?) = other is Map.Entry<*, *> && key == other.key && value == other.value
        override fun hashCode() = key.hashCode() xor value.hashCode()
    }
    private val storage = mutableListOf<Entry>()
    private var modifications = 0
    override fun put(key: String, value: SnyggValue): SnyggValue? {
        val i = SnyggCompilerAdapter.slot(storage.map { it.key }, key)
        if (i >= 0) return storage[i].setValue(value)
        storage.add(Entry(key, value)); modifications++
        return null
    }
    override fun get(key: String): SnyggValue? {
        val i = SnyggCompilerAdapter.slot(storage.map { it.key }, key)
        return storage.getOrNull(i)?.value
    }
    override fun remove(key: String): SnyggValue? {
        val i = SnyggCompilerAdapter.slot(storage.map { it.key }, key)
        if (i < 0) return null
        modifications++
        return storage.removeAt(i).value
    }
    override fun clear() { storage.clear(); modifications++ }
    override val entries: MutableSet<MutableMap.MutableEntry<String, SnyggValue>>
        get() = object : AbstractMutableSet<MutableMap.MutableEntry<String, SnyggValue>>() {
            override val size get() = storage.size
            override fun add(element: MutableMap.MutableEntry<String, SnyggValue>): Boolean =
                throw UnsupportedOperationException()
            override fun iterator(): MutableIterator<MutableMap.MutableEntry<String, SnyggValue>> =
                object : MutableIterator<MutableMap.MutableEntry<String, SnyggValue>> {
                    private var at = 0
                    private var last: String? = null
                    private var expected = modifications
                    private fun checkModification() {
                        if (expected != modifications) throw ConcurrentModificationException()
                    }
                    override fun hasNext() = at < storage.size
                    override fun next(): MutableMap.MutableEntry<String, SnyggValue> {
                        checkModification()
                        if (!hasNext()) throw NoSuchElementException()
                        return storage[at++].also { last = it.key }
                    }
                    override fun remove() {
                        checkModification()
                        val key = checkNotNull(last)
                        this@SnyggEditorProperties.remove(key); at--; last = null; expected = modifications
                    }
                }
        }
}
