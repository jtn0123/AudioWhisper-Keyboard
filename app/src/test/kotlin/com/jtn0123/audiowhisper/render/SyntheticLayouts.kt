package com.jtn0123.audiowhisper.render

import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import kotlinx.serialization.json.JsonPrimitive
import java.lang.reflect.Modifier
import kotlin.random.Random

/**
 * Seeded layout and popup-mapping files of the kind an extension could ship, for what the bundled
 * ones never use: multi-text keys, every selector nested in another, keys and popups of every
 * type, group and special code, empty rows, and the leniencies the JSON config allows (a quoted
 * code, a missing or unknown class name, a key type in any case).
 */
internal class SyntheticLayouts(private val random: Random) {
    private val labels = mutableListOf<String>()

    /** A layout file of a few rows, sometimes with an empty one partway down. */
    fun layout(): String {
        labels.clear()
        val rows = List(1 + random.nextInt(5)) { row ->
            if (row > 0 && random.nextInt(8) == 0) "[]" else array(List(1 + random.nextInt(11)) { key(2) })
        }
        return array(rows)
    }

    /** A modifier layout file, whose first row takes the main layout's last row where a code-0 text key stands. */
    fun modifier(): String {
        val rows = List(1 + random.nextInt(3)) { row ->
            val keys = MutableList(random.nextInt(6)) { key(1) }
            if (row == 0) repeat(1 + random.nextInt(2)) { keys.add(random.nextInt(keys.size + 1), PLACEHOLDER) }
            array(keys)
        }
        return array(rows)
    }

    /** A popup-mapping file whose labels are mostly ones the last [layout] used. */
    fun mapping(): String {
        val variations = VARIATIONS.filter { it == "all" || random.nextInt(3) == 0 }
        return obj(variations.map { variation ->
            variation to obj(List(random.nextInt(12)) { mappingLabel() }.distinct().map { it to popup(1) })
        })
    }

    private fun mappingLabel(): String = when (random.nextInt(4)) {
        0 -> GROUP_LABELS.random(random)
        1 -> LABELS.random(random)
        else -> labels.randomOrNull(random)?.lowercase() ?: "a"
    }

    private fun key(depth: Int): String {
        val pick = random.nextInt(if (depth > 0) 16 else 10)
        return when (pick) {
            in 0..4 -> leaf("auto_text_key", depth)
            in 5..7 -> leaf("text_key", depth)
            8 -> multi(depth)
            9 -> leaf(if (random.nextBoolean()) null else "unknown_key", depth)
            10 -> obj(listOf("$" to str("case_selector"), "lower" to key(depth - 1), "upper" to key(depth - 1)))
            11 -> selector("shift_state_selector", listOf("unshifted", "shifted", "shiftedManual", "shiftedAutomatic", "capsLock", "default"), depth)
            12 -> selector("variation_selector", listOf("default", "email", "uri", "normal", "password"), depth)
            13 -> obj(listOf("$" to str("layout_direction_selector"), "ltr" to key(depth - 1), "rtl" to key(depth - 1)))
            14 -> obj(listOf("$" to str("char_width_selector"), "full" to maybeKey(depth - 1), "half" to maybeKey(depth - 1)))
            else -> leaf("auto_text_key", depth)
        }
    }

    private fun maybeKey(depth: Int) = if (random.nextInt(4) == 0) "null" else key(depth)

    private fun selector(name: String, children: List<String>, depth: Int) = obj(listOf("$" to str(name)) +
        children.filter { random.nextInt(3) != 0 }.map { it to maybeKey(depth - 1) })

    private fun leaf(name: String?, depth: Int): String {
        val code = CODES.random(random)
        val label = when (random.nextInt(4)) {
            0 -> ""
            1 -> LABELS.random(random)
            else -> if (code > 0) String(Character.toChars(code)) else LABELS.random(random)
        }
        labels += label
        val fields = mutableListOf<Pair<String, String>>()
        name?.let { fields += "$" to str(it) }
        if (random.nextInt(6) != 0) fields += "type" to str(keyType())
        fields += "code" to if (random.nextInt(20) == 0) str("$code") else "$code"
        if (random.nextInt(8) != 0) fields += "label" to str(label)
        if (random.nextInt(4) == 0) fields += "groupId" to "${GROUPS.random(random)}"
        if (depth > 0 && random.nextInt(4) == 0) fields += "popup" to popup(depth - 1)
        return obj(fields)
    }

    private fun multi(depth: Int): String {
        val label = LABELS.random(random)
        labels += label
        val fields = mutableListOf("$" to str("multi_text_key"), "codePoints" to array(List(random.nextInt(4)) { "${CODES.random(random)}" }),
            "label" to str(label))
        if (random.nextBoolean()) fields += "type" to str(keyType())
        if (random.nextInt(4) == 0) fields += "groupId" to "${GROUPS.random(random)}"
        if (depth > 0 && random.nextInt(4) == 0) fields += "popup" to popup(depth - 1)
        return obj(fields)
    }

    private fun popup(depth: Int): String {
        val fields = mutableListOf<Pair<String, String>>()
        if (random.nextBoolean()) fields += "main" to key(depth)
        if (random.nextInt(4) != 0) fields += "relevant" to array(List(random.nextInt(5)) { key(depth) })
        return obj(fields)
    }

    private fun keyType(): String {
        val type = if (random.nextInt(3) == 0) KeyType.entries.random(random) else KeyType.CHARACTER
        return when (random.nextInt(3)) {
            0 -> type.name
            1 -> type.name.lowercase()
            else -> type.name.lowercase().replaceFirstChar(Char::uppercaseChar)
        }
    }

    private fun str(text: String) = JsonPrimitive(text).toString()
    private fun array(items: List<String>) = items.joinToString(",", "[", "]")
    private fun obj(fields: List<Pair<String, String>>) = fields.joinToString(",", "{", "}") { (key, value) -> "${str(key)}:$value" }

    private companion object {
        const val PLACEHOLDER = "{\"$\":\"text_key\",\"code\":0}"
        val VARIATIONS = listOf("all", "email", "uri", "normal", "password")
        val GROUP_LABELS = listOf("~enter", "~left", "~right")
        val GROUPS = listOf(0, 1, 2, 3, 7, -1)

        /** Every `KeyCode`, letters with casing of their own, marks, and characters past the BMP. */
        val CODES: List<Int> = KeyCode::class.java.declaredFields
            .filter { Modifier.isStatic(it.modifiers) && Modifier.isPublic(it.modifiers) && it.type == Int::class.javaPrimitiveType }
            .map { it.getInt(null) } +
            listOf(97, 98, 122, 65, 90, 105, 73, 0x131, 0x130, 0xDF, 0x1C5, 0xFB01, 0x3A3, 0x3C2, 0x301, 0x93F, 0x627, 0x5D0,
                0x1F600, 0x10400, 48, 49, 57, 0x660, 0x6F1, 0x966, 36, 8364, 0x3042, 0xFF41, Int.MIN_VALUE)

        val LABELS = listOf("a", "A", "ß", "İ", "ı", "ﬁ", "Σ", "ς", "ǅ", "😀", "́", "~enter", "a b", "123", "?", "ABC", "١")
    }
}
