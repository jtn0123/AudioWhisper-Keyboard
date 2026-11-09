package dev.patrickgold.florisboard.ime.text.composing

/**
 * The Kotlin `WithRules` composer replaced by lib/rust/crates/composing/src/rules.rs, kept to
 * test against. Edited only to drop its serialization.
 */
internal class ReferenceWithRules(
    override val id: String,
    override val label: String,
    val rules: Map<String, String>,
) : Composer {
    override val toRead = (rules.keys.maxOf { it.length } - 1).coerceAtLeast(0)

    val ruleOrder = rules.keys.toList().sortedBy { it.length }.reversed()

    override fun getActions(precedingText: String, toInsert: String): Pair<Int, String> {
        val str = precedingText + toInsert
        for (key in ruleOrder) {
            if (str.lowercase().endsWith(key)) {
                val value = rules.getValue(key)
                val firstOfKey = str.takeLast(key.length).take(1)
                return (key.length - 1) to (if (firstOfKey.uppercase() == firstOfKey) value.uppercase() else value)
            }
        }
        return 0 to toInsert
    }
}
