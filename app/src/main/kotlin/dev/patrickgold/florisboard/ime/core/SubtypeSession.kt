package dev.patrickgold.florisboard.ime.core

import com.jtn0123.audiowhisper.rust.ImeSubtypeLoaded
import com.jtn0123.audiowhisper.rust.ImeSubtypeOp
import com.jtn0123.audiowhisper.rust.ImeSubtypeRow
import com.jtn0123.audiowhisper.rust.ImeSubtypeSwitched
import com.jtn0123.audiowhisper.rust.ImeSubtypes
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.ime.keyboard.CurrencySet
import dev.patrickgold.florisboard.ime.keyboard.LayoutType
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName

/**
 * What Rust says a subtype names for the key graph: its cache [key], the same for equal subtypes,
 * its layout for each `LayoutType` and its [popupMapping].
 */
internal class SubtypeFacts(val key: Int, private val layouts: List<String>, val popupMapping: ExtensionComponentName) {
    /** The layout the subtype names for [type], if any. */
    fun layout(type: LayoutType): ExtensionComponentName? =
        layouts[type.ordinal].takeIf { it.isNotEmpty() }?.let(ExtensionComponentName::from)
}

/** A subtype as Kotlin's readers take it, and what Rust says it names. */
internal class SubtypeEntry(val subtype: Subtype, val facts: SubtypeFacts)

/**
 * The keyboard's subtypes as Rust holds them (crates/imesubtype): the list, the active subtype and
 * what each names. Android names locale tags with [name], as `FlorisLocale` names them, and
 * reports the currency sets it loaded.
 */
internal class SubtypeSession(
    private val name: (String) -> String = { FlorisLocale.fromTag(it).languageTag() },
    private val native: ImeSubtypes = ImeSubtypes(),
) {
    /** The list a load read: each subtype, and the repaired list's text to store when it was repaired. */
    class Loaded(val list: List<SubtypeEntry>, val repaired: String?)

    /** A switch's active subtype, and whether Android stores its id. */
    class Switched(val active: SubtypeEntry, val write: Boolean)

    /** The currency sets last reported to Rust: the map Android loaded them in, and each set in its order. */
    private var reportedMap: Map<ExtensionComponentName, CurrencySet>? = null
    private var reportedSets = listOf<CurrencySet>()

    /** Loads the stored list [raw]; blank is empty. Throws when it doesn't parse. */
    @Synchronized
    fun load(raw: String): Loaded {
        var rows = native.call(ImeSubtypeOp.LOAD, raw)
        val asks = rows[ImeSubtypeLoaded.ASKS].toInt()
        if (asks > 0) {
            val tags = rows.copyOfRange(ImeSubtypeLoaded.SIZE, ImeSubtypeLoaded.SIZE + asks)
            native.call(ImeSubtypeOp.LOCALES, *tags.map(name).toTypedArray())
            rows = native.call(ImeSubtypeOp.LOAD, raw)
            check(rows[ImeSubtypeLoaded.ASKS] == "0") { "a load asked again" }
        }
        var at = ImeSubtypeLoaded.SIZE
        val list = List(rows[ImeSubtypeLoaded.COUNT].toInt()) { entry(rows, at).also { at = it.second }.first }
        val repaired = rows[ImeSubtypeLoaded.RAW].takeIf { WireFlag.decode(rows[ImeSubtypeLoaded.CHANGED]) }
        return Loaded(list, repaired)
    }

    /** The active subtype once the list changed, given the id the settings store. */
    fun evaluate(stored: Long): Switched = checkNotNull(switched(ImeSubtypeOp.EVALUATE, "$stored"))

    /** The language key's next subtype. */
    fun next(): Switched = checkNotNull(switched(ImeSubtypeOp.NEXT))

    /** The language key's previous subtype. */
    fun previous(): Switched = checkNotNull(switched(ImeSubtypeOp.PREVIOUS))

    /** The active subtype, unchanged. */
    fun active(): SubtypeEntry = checkNotNull(switched(ImeSubtypeOp.ACTIVE)).active

    /** The subtype with [id], made active, or null when the list has none. */
    fun select(id: Long): Switched? = switched(ImeSubtypeOp.SELECT, "$id")

    /**
     * The set [currencySet] names among [sets], Android's loaded currency sets, or null for none.
     * Android reports the sets to Rust whenever it loaded a new map of them.
     */
    @Synchronized
    fun currencySet(sets: Map<ExtensionComponentName, CurrencySet>?, currencySet: ExtensionComponentName): CurrencySet? {
        if (sets !== reportedMap) {
            val entries = sets.orEmpty().entries.toList()
            native.call(ImeSubtypeOp.SETS, *entries.map { "${it.key}" }.toTypedArray())
            reportedMap = sets
            reportedSets = entries.map { it.value }
        }
        return reportedSets.getOrNull(native.call(ImeSubtypeOp.CURRENCY, "$currencySet")[0].toInt())
    }

    @Synchronized
    private fun switched(operation: ImeSubtypeOp, vararg fields: String): Switched? {
        val rows = native.call(operation, *fields)
        if (!WireFlag.decode(rows[ImeSubtypeSwitched.ACTIVE])) return null
        val (active, _) = entry(rows, ImeSubtypeSwitched.SIZE)
        return Switched(active, WireFlag.decode(rows[ImeSubtypeSwitched.WRITE]))
    }
}

/** The subtype row at [at] in [rows], and where the next row starts. */
private fun entry(rows: Array<String>, at: Int): Pair<SubtypeEntry, Int> = with(ImeSubtypeRow) {
    fun field(field: Int) = rows[at + field]
    fun component(field: Int) = ExtensionComponentName.from(field(field))
    val secondary = rows.copyOfRange(at + SIZE, at + SIZE + field(SECONDARY).toInt())
    val subtype = Subtype(
        field(ID).toLong(), FlorisLocale.fromTag(field(LOCALE)), secondary.map(FlorisLocale::fromTag),
        SubtypeNlpProviderMap(field(SPELLING), field(SUGGESTION)), component(COMPOSER), component(CURRENCY),
        component(PUNCTUATION), component(POPUP),
        SubtypeLayoutMap(component(CHARACTERS), component(SYMBOLS), component(SYMBOLS2), component(NUMERIC),
            component(NUMERIC_ADVANCED), component(NUMERIC_ROW), component(PHONE), component(PHONE2)),
    )
    val layouts = LayoutType.entries.map { field(CHARACTERS + it.ordinal) }
    SubtypeEntry(subtype, SubtypeFacts(field(KEY).toInt(), layouts, subtype.popupMapping)) to at + SIZE + secondary.size
}
