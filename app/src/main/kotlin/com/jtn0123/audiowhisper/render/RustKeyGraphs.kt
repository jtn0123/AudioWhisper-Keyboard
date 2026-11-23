package com.jtn0123.audiowhisper.render

import android.content.Context
import android.icu.lang.UCharacter
import com.jtn0123.audiowhisper.dictation.DictationFields
import com.jtn0123.audiowhisper.rust.KeyGraph
import com.jtn0123.audiowhisper.rust.KeyGraphLocale
import com.jtn0123.audiowhisper.rust.LayoutCache
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.ime.core.DisplayLanguageNamesIn
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.core.SubtypeFacts
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.KEY_LABEL_RESOURCES
import dev.patrickgold.florisboard.ime.keyboard.KeyboardManager
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.computeLanguageDisplayName
import dev.patrickgold.florisboard.ime.keyboard.keyLabelResource
import dev.patrickgold.florisboard.lib.Unicode
import dev.patrickgold.florisboard.lib.io.ZipUtils
import dev.patrickgold.florisboard.subtypeManager
import org.florisboard.lib.android.AndroidKeyguardManager
import org.florisboard.lib.android.systemService

/**
 * The Rust key graphs, one for each keyboard mode and subtype, as Kotlin's keyboard cache held
 * them. Kotlin reads the layout files and reports the facts; Rust builds and computes the keys,
 * asking Kotlin only for locale casing and ICU's non-spacing-mark predicate.
 */
internal class RustKeyGraphs(private val context: Context, private val manager: KeyboardManager) {
    private val prefs by FlorisPreferenceStore
    private val extensionManager by context.extensionManager()
    private val clipboardManager by context.clipboardManager()
    private val subtypeManager by context.subtypeManager()
    private val keyguard by lazy { context.systemService(AndroidKeyguardManager::class) }
    private val cache = LayoutCache<RustKeyboard>()
    /** Each file read, by its extension and path; null for one that didn't read. Never cleared, as main's weren't. */
    private val texts = HashMap<String, String?>()
    private var defaultLanguage: Pair<DisplayLanguageNamesIn, String>? = null

    fun clear() = cache.clear()
    fun clear(mode: KeyboardMode) = cache.clearMode(mode.value)

    /** The keyboard for [mode] and the subtype Rust says names [facts], built the first time it is asked for. */
    fun keyboard(mode: KeyboardMode, facts: SubtypeFacts): RustKeyboard = cache.get(mode.value, facts.key)
        ?: RustKeyboard(KeyGraph.build(*build(mode, facts))).also { cache.set(mode.value, facts.key, it) }

    /** Computes [keyboard]'s keys for [evaluator]: its state, editor and subtype, and the facts around them. */
    fun compute(keyboard: RustKeyboard, evaluator: ComputingEvaluator) {
        val subtype = evaluator.subtype.primaryLocale.base
        keyboard.compute(facts(evaluator).row()) { ask, locale, value ->
            val asked = if (locale == KeyGraphLocale.SUBTYPE) subtype else Subtype.DEFAULT.primaryLocale.base
            keyGraphAnswer(ask, value, asked, UCharacter::toString, Unicode::isNonSpacingMark)
        }
    }

    private fun facts(evaluator: ComputingEvaluator): KeyGraphFactsInput {
        val info = evaluator.editorInfo
        val names = prefs.localization.displayLanguageNamesIn.get()
        return KeyGraphFactsInput(evaluator.state.rawValue, info.inputAttributes.raw, info.imeOptions.raw,
            prefs.keyboard.utilityKeyEnabled.get(), prefs.keyboard.utilityKeyAction.get().ordinal,
            manager.shouldShowLanguageSwitch(), info.isRichInputEditor,
            keyguard.isDeviceLocked || keyguard.isKeyguardLocked,
            clipboardManager.canBePasted(clipboardManager.primaryClip), prefs.suggestion.incognitoMode.get().ordinal,
            info.imeOptions.flagNoPersonalizedLearning, subtypeManager.subtypes.size,
            DictationFields.allows(info.inputAttributes),
            computeLanguageDisplayName(evaluator.subtype.primaryLocale, names), defaultLanguage(names),
            prefs.keyboard.keyHintConfiguration(), subtypeManager.getCurrencySet(evaluator.subtype).slots,
            KEY_LABEL_RESOURCES.map { keyLabelResource(it) { context } })
    }

    /** `Subtype.DEFAULT`'s language name, kept apart from the subtype's one-entry cache. */
    private fun defaultLanguage(names: DisplayLanguageNamesIn): String =
        defaultLanguage?.takeIf { it.first == names }?.second ?: Subtype.DEFAULT.primaryLocale
            .let { if (names == DisplayLanguageNamesIn.NATIVE_LOCALE) it.displayName(it) else it.displayName() }
            .also { defaultLanguage = names to it }

    private fun build(mode: KeyboardMode, facts: SubtypeFacts): Array<String> {
        val layouts = manager.resources.layouts.value
        val mappings = manager.resources.popupMappings.value
        val files = subtypeFiles(
            facts,
            { (type, name) -> layouts?.get(type)?.get(name)?.modifier },
            { (type, name) -> layouts?.get(type)?.get(name)?.let { read(name.extensionId, it.arrangementFile(type)) } },
            { name -> mappings?.get(name)?.let { read(name.extensionId, it.mappingFile()) } },
        )
        return keyGraphBuild(mode, prefs.keyboard.numberRow.get(), prefs.keyboard.hintedNumberRowEnabled.get(), files)
    }

    /**
     * A file's text, kept once read (or once it failed to), as Kotlin's layout cache kept it; a
     * missing extension is asked again.
     */
    private fun read(extension: String, path: String): String? {
        val source = extensionManager.getExtensionById(extension)?.sourceRef ?: return null
        val read = { ZipUtils.readFileFromArchive(context, source, path).getOrNull() }
        return synchronized(texts) { texts.getOrPut("$extension/$path", read) }
    }
}
