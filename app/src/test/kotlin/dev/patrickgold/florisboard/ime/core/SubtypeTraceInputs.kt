package dev.patrickgold.florisboard.ime.core

import com.jtn0123.audiowhisper.render.KeyGraphAssets
import dev.patrickgold.florisboard.ime.keyboard.CurrencySet
import dev.patrickgold.florisboard.ime.keyboard.LayoutType
import dev.patrickgold.florisboard.ime.keyboard.extCoreCurrencySet
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject
import kotlin.coroutines.CoroutineContext
import kotlin.random.Random

/** A dispatcher that runs nothing until the trace says so, one task at a time, in order. */
internal class QueueDispatcher : CoroutineDispatcher() {
    private val queue = ArrayDeque<Runnable>()
    override fun dispatch(context: CoroutineContext, block: Runnable) { queue.addLast(block) }

    /** Runs at most [tasks] tasks, or until none is left; how many it ran. */
    fun run(tasks: Int = Int.MAX_VALUE): Int {
        var ran = 0
        while (ran < tasks && queue.isNotEmpty()) { queue.removeFirst().run(); ran++ }
        return ran
    }

    fun clear() = queue.clear()
}

/** The subtype settings in memory, with every write the keyboard made, in order. */
internal class TraceStore(raw: String, private var id: Long) : SubtypeStore {
    override val lists = MutableStateFlow(raw)
    val writes = mutableListOf<String>()
    override fun activeId() = id
    fun put(raw: String) { lists.value = raw }
    fun putId(value: Long) { id = value }
    override suspend fun setActiveId(id: Long) { writes += "active $id"; this.id = id }
    override suspend fun setList(raw: String) { writes += "list ${raw.hashCode()}/${raw.length}"; lists.value = raw }
}

/**
 * The stored lists, edits and currency sets the subtype trace feeds both keyboards: subtypes as the
 * settings screens store them, with locale tags Android names differently, layouts, composers,
 * popup mappings and providers the keyboard no longer has, add-on components, missing fields,
 * duplicate ids, and blank or broken lists.
 */
internal class SubtypeTraceInputs(private val random: Random) {
    private var time = 1_700_000_000_000L

    /** A new subtype with [id]. */
    fun subtype(id: Long): JsonObject = buildJsonObject {
        put("id", id)
        put("primaryLocale", TAGS.random(random))
        putJsonArray("secondaryLocales") { repeat(if (random.nextInt(4) == 0) random.nextInt(1, 3) else 0) { add(JsonPrimitive(TAGS.random(random))) } }
        if (random.nextInt(8) != 0) putJsonObject("nlpProviders") {
            put("spelling", PROVIDERS.random(random)); put("suggestion", PROVIDERS.random(random))
        }
        put("composer", COMPOSERS.random(random))
        put("currencySet", rare(CURRENCIES, "broken"))
        if (random.nextInt(6) != 0) put("punctuationRule", PUNCTUATION.random(random))
        put("popupMapping", POPUPS.random(random))
        putJsonObject("layoutMap") {
            for ((type, key) in MAPPED) if (random.nextInt(4) != 0) put(key, layouts(type).random(random))
        }
    }

    /** [list] after an edit the settings screens make, and the edit's name. */
    fun edit(list: List<JsonObject>): Pair<List<JsonObject>, String> {
        val out = list.toMutableList()
        val name = when (if (out.isEmpty()) 0 else random.nextInt(12)) {
            0, 1, 2 -> "add".also { out.add(random.nextInt(out.size + 1), subtype(time++)) }
            3 -> "add a duplicate id".also { out += subtype(out.random(random).id()) }
            4 -> "add a copy".also { out += JsonObject(out.random(random) + ("id" to JsonPrimitive(time++))) }
            5, 6 -> "remove".also { out.removeAt(random.nextInt(out.size)) }
            7, 8 -> "modify".also { val at = random.nextInt(out.size); out[at] = subtype(out[at].id()) }
            9 -> "reorder".also { out.shuffle(random) }
            10 -> "move".also { out.add(random.nextInt(out.size), out.removeAt(random.nextInt(out.size))) }
            else -> "rewrite"
        }
        if (random.nextInt(40) == 0) out.clear()
        return out to name
    }

    /** [list] as the setting's text: compact, spaced, or now and then blank or broken. */
    fun raw(list: List<JsonObject>): String = when (random.nextInt(400)) {
        0 -> listOf("", "  ", " \t", " ").random(random)
        1 -> listOf("[{", "{}", "[1]", "[{\"id\":1}]", "nope").random(random)
        in 2..30 -> PRETTY.encodeToString(JsonArray.serializer(), JsonArray(list))
        else -> JsonArray(list).toString()
    }

    /** The stored list [raw] as the trace edits it: empty when it is blank or doesn't parse. */
    fun list(raw: String): List<JsonObject> = runCatching {
        Json.parseToJsonElement(raw).jsonArray.map { it.jsonObject }
    }.getOrDefault(emptyList())

    /** The currency sets Android loaded: some of the bundled names, or none loaded yet. */
    fun currencySets(): Map<ExtensionComponentName, CurrencySet>? = if (random.nextInt(10) == 0) null else
        SETS.filter { random.nextInt(4) != 0 }.associate { name ->
            val component = ExtensionComponentName.from(name)
            component to CurrencySet(component.componentId, name, List(random.nextInt(7)) { TextKeyData(code = 36 + it) })
        }

    /** An id the list may have. */
    fun id(list: List<JsonObject>): Long = if (list.isNotEmpty() && random.nextInt(4) != 0) list.random(random).id()
        else listOf(-1L, 0L, time + 7).random(random)

    private fun layouts(type: LayoutType): List<String> =
        KeyGraphAssets.names[type].orEmpty().map { "$it" } + if (type == LayoutType.CHARACTERS) LOST_LAYOUTS else emptyList()

    private fun rare(pool: List<String>, odd: String) = if (random.nextInt(3_000) == 0) odd else pool.random(random)

    private fun JsonObject.id() = (getValue("id") as JsonPrimitive).content.toLong()

    companion object {
        private val PRETTY = Json { prettyPrint = true }
        val TAGS = listOf("en-US", "en_US", "en-us", "EN-US", "en", "fr-FR", "fr_FR", "de-DE", "de-CH-1996", "ar",
            "ar-EG", "fa-IR", "he-IL", "iw", "ru-RU", "zh-Hant-TW", "zh_TW", "ja-JP", "ko-KR", "sr-Latn-RS", "es-419",
            "pt_BR", "nb", "no", "in", "id", "tlh", "und", "", "x-private", "en-US-POSIX", "zh-min-nan")
        val PROVIDERS = listOf("org.florisboard.nlp.providers.latin", "org.florisboard.nlp.providers.han.shape", "my.addon.nlp")
        val COMPOSERS = listOf("appender", "telex", "hangul-unicode", "kana-unicode").map { "org.florisboard.composers:$it" } +
            "my.addon:kana-unicode"
        val SETS = listOf("dollar", "euro", "pound", "yen", "rupee", "won").map { "${extCoreCurrencySet(it)}" } + "my.addon:coins"
        val CURRENCIES = SETS + "org.florisboard.currencysets:lira"
        val PUNCTUATION = listOf("org.florisboard.localization:default", "my.addon:rules")
        val POPUPS = KeyGraphAssets.mappings.keys.map { "$it" } +
            listOf("ar", "bg", "cjk").map { "org.florisboard.localization:$it" } + "my.addon:cjk"
        val LOST_LAYOUTS = listOf("arabic", "jcuken_russian", "not_indexed").map { "org.florisboard.layouts:$it" } +
            "my.addon:arabic"
        val MAPPED = listOf(LayoutType.CHARACTERS to "characters", LayoutType.SYMBOLS to "symbols",
            LayoutType.SYMBOLS2 to "symbols2", LayoutType.NUMERIC to "numeric",
            LayoutType.NUMERIC_ADVANCED to "numericAdvanced", LayoutType.NUMERIC_ROW to "numericRow",
            LayoutType.PHONE to "phone", LayoutType.PHONE2 to "phone2")
    }
}
