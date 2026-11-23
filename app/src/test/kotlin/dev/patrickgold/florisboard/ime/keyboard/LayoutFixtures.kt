package dev.patrickgold.florisboard.ime.keyboard

import com.jtn0123.audiowhisper.rust.KeyModels
import dev.patrickgold.florisboard.ime.core.SubtypeLayoutMap
import dev.patrickgold.florisboard.ime.popup.PopupMapping
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.text.keyboard.AutoTextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.lib.io.DefaultJsonConfig
import kotlinx.serialization.json.*
import java.io.File
import java.lang.reflect.Proxy
import java.util.IdentityHashMap
import java.util.Locale

internal class LayoutFixtures {
    data class Layout(val type: LayoutType, val name: ExtensionComponentName, val meta: LayoutArrangementComponent,
                      val rows: LayoutArrangement)
    data class Preset(val locale: Locale, val layouts: SubtypeLayoutMap, val popups: ExtensionComponentName,
                      val currency: String)
    private val root = File("src/main/assets/ime/keyboard")
    private val layoutRoot = File(root, "org.florisboard.layouts")
    private val index = DefaultJsonConfig.parseToJsonElement(File(layoutRoot, "extension.json").readText()).jsonObject
    val layouts = index.getValue("layouts").jsonObject.flatMap { (type, values) ->
        val kind = LayoutType.entries.single { it.id == type }
        values.jsonArray.map { value ->
            val meta = DefaultJsonConfig.decodeFromJsonElement<LayoutArrangementComponent>(value)
            Layout(kind, extCoreLayout(meta.id), meta,
                DefaultJsonConfig.decodeFromString<LayoutArrangement>(File(layoutRoot, meta.arrangementFile(kind)).readText()))
        }
    }
    val bundledFiles = File(layoutRoot, "layouts").walkTopDown().filter { it.isFile && it.extension == "json" }.toSet()
    val indexedFiles = layouts.map { File(layoutRoot, it.meta.arrangementFile(it.type)) }.toSet()
    private val localization = File(root, "org.florisboard.localization")
    val presets = DefaultJsonConfig.parseToJsonElement(File(localization, "extension.json").readText()).jsonObject
        .getValue("subtypePresets").jsonArray.map { value ->
            val p = value.jsonObject
            Preset(Locale.forLanguageTag(p.getValue("languageTag").jsonPrimitive.content),
                DefaultJsonConfig.decodeFromJsonElement<SubtypeLayoutMap>(p.getValue("preferred")),
                ExtensionComponentName.from(p["popupMapping"]?.jsonPrimitive?.content ?: "org.florisboard.localization:default"),
                p.getValue("currencySet").jsonPrimitive.content.substringAfter(':'))
        }
    val popupMappings = File(localization, "popupMappings").listFiles()!!.associate { file ->
        file.nameWithoutExtension to DefaultJsonConfig.decodeFromString<PopupMapping>(file.readText())
    }
    val currencySets = DefaultJsonConfig.parseToJsonElement(
        File(root, "org.florisboard.currencysets/extension.json").readText()).jsonObject.getValue("currencySets").jsonArray
        .associate { value -> value.jsonObject.let { it.getValue("id").jsonPrimitive.content to
            DefaultJsonConfig.decodeFromJsonElement<List<TextKeyData>>(it.getValue("slots")) } }

    fun load(request: Pair<LayoutType, ExtensionComponentName>?, candidate: Layout?): Layout? {
        request ?: return null
        if (candidate?.type == request.first) return candidate
        return layouts.firstOrNull { it.type == request.first && it.name == request.second }
    }

    /** No Android ICU in JVM tests: feed both sides identical locale-casing snapshots.
     * The Android casing/cache was not changed; the emulator exercises that path. */
    class Prepared(private val locale: Locale) {
        private val cache = IdentityHashMap<AbstractKeyData, AbstractKeyData>()
        fun rows(rows: LayoutArrangement?) = rows?.map { row -> row.map(::key) }
        fun popup(popup: PopupSet<AbstractKeyData>?) = popup?.let { PopupSet(it.main?.let(::key), it.relevant.map(::key)) }
        fun key(data: AbstractKeyData): AbstractKeyData = cache.getOrPut(data) {
            when (data) {
                is AutoTextKeyData -> CaseSelector(
                    TextKeyData(data.type, data.code, data.label.lowercase(locale), data.groupId, popup(data.popup)),
                    TextKeyData(data.type, data.code, data.label.uppercase(locale), data.groupId, popup(data.popup)))
                is TextKeyData -> data.copy(popup = popup(data.popup))
                is CaseSelector -> CaseSelector(key(data.lower), key(data.upper))
                is ShiftStateSelector -> ShiftStateSelector(data.unshifted?.let(::key), data.shifted?.let(::key),
                    data.shiftedManual?.let(::key), data.shiftedAutomatic?.let(::key), data.capsLock?.let(::key), data.default?.let(::key))
                is VariationSelector -> VariationSelector(data.default?.let(::key), data.email?.let(::key), data.uri?.let(::key),
                    data.normal?.let(::key), data.password?.let(::key))
                is LayoutDirectionSelector -> LayoutDirectionSelector(key(data.ltr), key(data.rtl))
                is CharWidthSelector -> CharWidthSelector(data.full?.let(::key), data.half?.let(::key))
                else -> data
            }
        }
    }

    fun evaluator(raw: Long, currency: String? = null): ComputingEvaluator {
        val state = KeyboardState.new(raw.toULong())
        return Proxy.newProxyInstance(ComputingEvaluator::class.java.classLoader, arrayOf(ComputingEvaluator::class.java)) { _, method, args ->
            when (method.name) {
                "getState" -> state
                "isSlot" -> currency != null && KeyModels.isCurrency((args!![0] as KeyData).code)
                "slotData" -> currencySets[currency]?.getOrNull(KeyModels.currencySlot((args!![0] as KeyData).code))
                else -> error("Unexpected evaluator access: ${method.name}")
            }
        } as ComputingEvaluator
    }
}
