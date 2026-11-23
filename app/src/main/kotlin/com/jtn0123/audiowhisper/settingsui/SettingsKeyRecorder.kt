package com.jtn0123.audiowhisper.settingsui

import android.content.Context
import android.icu.lang.UCharacter
import androidx.activity.ComponentActivity
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.ime.input.InputKeyEventReceiver
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.DefaultComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.Key
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.keyboard.Keyboard
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.Unicode
import kotlin.reflect.KMutableProperty0

/** One key-up taken from this app's keyboard for the code editor's *record a key*, as EditCodeValueDialog's receiver
 * took it: [start] puts this in [slot], the InputEventDispatcher's `keyEventReceiver`, in place of the receiver there;
 * the next key-up gives that receiver back and goes to [recorded]; a key down, repeat or cancel is dropped. [stop] gives
 * the receiver back while this still holds the slot, and leaves alone one put in after it. */
internal class KeyRecording(private val slot: KMutableProperty0<InputKeyEventReceiver?>, private val recorded: (Int) -> Unit) :
    InputKeyEventReceiver {
    private var previous: InputKeyEventReceiver? = null
    val listening get() = slot.get() === this
    fun start() {
        if (listening) return
        previous = slot.get()
        slot.set(this)
    }
    /** Whether this was listening. */
    fun stop(): Boolean {
        val was = listening
        if (was) slot.set(previous)
        previous = null
        return was
    }
    override fun onInputKeyUp(data: KeyData) { if (stop()) recorded(data.code) }
    override fun onInputKeyDown(data: KeyData) = Unit
    override fun onInputKeyRepeat(data: KeyData) = Unit
    override fun onInputKeyCancel(data: KeyData) = Unit
}

/** A `THEME_CODE` report of [kind] about [code], with nothing described. */
internal fun themeCodeRow(kind: String, code: String = "") = SettingsUiThemeCode.row(SettingsUiThemeCode.KIND to kind,
    SettingsUiThemeCode.CODE to code, SettingsUiThemeCode.NAME to "", SettingsUiThemeCode.NON_SPACING to WireFlag.encode(false),
    SettingsUiThemeCode.ICON to "")

/** The `THEME_CODE` described report for [code], as TextKeyDataPreviewBox worked it out: for a character its Unicode
 * [name] and whether it is a [nonSpacing] mark, and the keyboard's [icon] token for the key the code names, which Rust
 * draws; 0 for none. */
internal fun describedCode(code: Int, name: (Int) -> String?, nonSpacing: (Int) -> Boolean, icon: (KeyData) -> Int):
    Array<String> {
    val data = TextKeyData.getCodeInfoAsTextKeyData(code) ?: TextKeyData.UNSPECIFIED
    val character = data.code > 0
    return SettingsUiThemeCode.row(SettingsUiThemeCode.KIND to SettingsUiThemeCodeReport.DESCRIBED,
        SettingsUiThemeCode.CODE to code.toString(), SettingsUiThemeCode.NAME to (if (character) name(code) else null).orEmpty(),
        SettingsUiThemeCode.NON_SPACING to WireFlag.encode(character && nonSpacing(code)),
        SettingsUiThemeCode.ICON to icon(data).toString())
}

/** The icon token the keyboard draws for [data] (`InputRules.visual`), as `computeImageVector` picks its vector. */
private fun ComputingEvaluator.iconToken(data: KeyData): Int = InputRules.visual(longArrayOf(data.code.toLong(),
    state.rawValue.toLong(), editorInfo.inputAttributes.raw.toLong(), editorInfo.imeOptions.raw.toLong(), keyboard.mode.toInt().toLong()))

/** TextKeyDataPreviewBox's evaluator: the keyboard's defaults in the numeric advanced mode, with [context] for the icons
 * it takes from resources. */
internal fun previewEvaluator(context: Context?): ComputingEvaluator = object : ComputingEvaluator by DefaultComputingEvaluator {
    override val keyboard = object : Keyboard() {
        override val mode = KeyboardMode.NUMERIC_ADVANCED
        override fun getKeyForPos(pointerX: Float, pointerY: Float): Key? = error("not implemented")
        override fun keys(): Iterator<Key> = error("not implemented")
        override fun layout(keyboardWidth: Float, keyboardHeight: Float, desiredKey: Key, extendTouchBoundariesDownwards: Boolean) =
            error("not implemented")
    }
    override fun context() = context
}

/** The code editor's Kotlin side, for what only Kotlin reaches: this app's keyboard's key events, which live in the IME's
 * InputEventDispatcher, and ICU's names and the keyboard's icon vectors for the key data preview. A `themeCodeRecord`
 * effect takes the next key-up and answers with a key report; `themeCodeStop` and the activity stopping give the keyboard
 * its receiver back, the latter answering with a stopped report, so a key pressed in another app is never taken; and
 * `themeCodeDescribe` answers for the one code the preview shows. Codes go nowhere but [report], and are never logged. */
internal class SettingsKeyRecorder(private val activity: ComponentActivity, private val report: (Array<String>) -> Unit) {
    private val keyboardManager by activity.keyboardManager()
    private val recording by lazy {
        KeyRecording(keyboardManager.inputEventDispatcher::keyEventReceiver) { code ->
            activity.runOnUiThread { report(themeCodeRow(SettingsUiThemeCodeReport.KEY, code.toString())) }
        }
    }
    private val evaluator = previewEvaluator(activity)
    private val stopped = LifecycleEventObserver { _, event ->
        if (event == Lifecycle.Event.ON_STOP && recording.stop()) report(themeCodeRow(SettingsUiThemeCodeReport.STOPPED))
    }
    init { activity.lifecycle.addObserver(stopped) }

    /** A `themeCodeRecord`, `themeCodeStop` or `themeCodeDescribe` effect. */
    fun execute(p: List<String>) {
        when (p[0]) {
            SettingsUiThemeCodeEffect.RECORD -> recording.start()
            SettingsUiThemeCodeEffect.STOP -> recording.stop()
            SettingsUiThemeCodeEffect.DESCRIBE -> p.getOrNull(1)?.toIntOrNull()?.let { code ->
                report(describedCode(code, { UCharacter.getName(it) ?: UCharacter.getExtendedName(it) }, Unicode::isNonSpacingMark) {
                    evaluator.iconToken(it)
                })
            }
        }
    }
    /** The settings view is gone: the keyboard gets its receiver back. */
    fun release() {
        activity.lifecycle.removeObserver(stopped)
        recording.stop()
    }
}
