package com.jtn0123.audiowhisper.settingsui

import android.app.Activity
import android.content.ComponentName
import android.content.res.Configuration
import android.database.ContentObserver
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.drawable.Drawable
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import android.widget.Toast
import androidx.core.content.res.ResourcesCompat
import androidx.core.graphics.createBitmap
import com.jtn0123.audiowhisper.dictation.ModelFiles
import com.jtn0123.audiowhisper.dictation.ModelState
import com.jtn0123.audiowhisper.reader.VoiceEngine
import dev.patrickgold.florisboard.BuildConfig
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.lib.util.launchActivity
import dev.patrickgold.florisboard.lib.util.launchUrl
import dev.patrickgold.florisboard.nlpManager
import org.florisboard.lib.android.AndroidSettings
import kotlin.concurrent.thread

/**
 * The device facts the native screens read (settings op 20) and the platform effects their rows
 * return. Rust decides what each fact means and what each effect says; this only reads Android
 * and carries the plans out.
 */
internal class SettingsFacts(private val activity: Activity) {
    private var spellIconFor: String? = null
    private var appIconSent = false
    private val observer = object : ContentObserver(Handler(Looper.getMainLooper())) {
        override fun onChange(selfChange: Boolean) { listener?.invoke() }
    }
    private var listener: (() -> Unit)? = null
    private var models: List<String> = emptyList()
    private val main = Handler(Looper.getMainLooper())

    /** SpellCheckerServiceSelector observes these two while the screen is in the foreground. */
    fun observe(onChange: () -> Unit) {
        listener = onChange
        for (key in SPELL_KEYS) AndroidSettings.Secure.getUriFor(key)?.let {
            activity.contentResolver.registerContentObserver(it, false, observer)
        }
    }
    fun release() { listener = null; activity.contentResolver.unregisterContentObserver(observer) }

    /** Key/value pairs; a null setting is left out, so Rust can tell null from "null". */
    fun pairs(): Array<String> {
        val pm = activity.packageManager
        return factPairs(
            BuildConfig.VERSION_NAME, BuildConfig.VERSION_CODE, activity.packageName,
            activity.resources.configuration.keyboard != Configuration.KEYBOARD_NOKEYS,
            AndroidSettings.Secure.getString(activity, "spell_checker_enabled"),
            AndroidSettings.Secure.getString(activity, "selected_spell_checker"),
        ) { pkg -> runCatching { pm.getApplicationLabel(pm.getApplicationInfo(pkg, 0)).toString() }.getOrNull() } +
            models.toTypedArray()
    }

    /** ModelStatus: the names and sizes at once, then the SHA-256 check off the main thread. */
    fun checkModels(onChange: () -> Unit) {
        val context = activity.applicationContext
        models = modelPairs(ModelFiles.quickState(context), VoiceEngine.quickState(context), checked = false)
        onChange()
        thread(name = "settings-model-check") {
            val full = modelPairs(ModelFiles.fullState(context), VoiceEngine.fullState(context), checked = true)
            main.post { models = full; onChange() }
        }
    }

    /** The pictures the screens draw: the app icon once, the spell checker's when it changes. */
    fun images(send: (String, Int, Int, IntArray) -> Unit) {
        if (!appIconSent) {
            ResourcesCompat.getDrawable(activity.resources, R.mipmap.floris_app_icon, null)?.let { picture("app_icon", it, send) }
            appIconSent = true
        }
        val pkg = spellPackage(AndroidSettings.Secure.getString(activity, "selected_spell_checker"))
        if (pkg == spellIconFor) return
        spellIconFor = pkg
        val icon = runCatching { activity.packageManager.getApplicationIcon(activity.packageManager.getApplicationInfo(pkg, 0)) }.getOrNull()
        if (icon == null) send("spell_icon", 0, 0, IntArray(0)) else picture("spell_icon", icon, send)
    }

    /** FlorisCanvasIcon's bitmap: the drawable at its intrinsic size. */
    private fun picture(name: String, drawable: Drawable, send: (String, Int, Int, IntArray) -> Unit) {
        val bitmap: Bitmap = createBitmap(drawable.intrinsicWidth, drawable.intrinsicHeight).also { target ->
            drawable.setBounds(0, 0, target.width, target.height)
            drawable.draw(Canvas(target))
        }
        val pixels = IntArray(bitmap.width * bitmap.height)
        bitmap.getPixels(pixels, 0, bitmap.width, 0, 0, bitmap.width, bitmap.height)
        send(name, bitmap.width, bitmap.height, pixels)
        bitmap.recycle()
    }

    /** Executes one of the effects group 2 rows return; false for any other effect. */
    fun execute(p: List<String>): Boolean {
        when (p[0]) {
            "url" -> activity.launchUrl(p[1])
            "intent" -> when (p[1]) {
                "hardKeyboardSettings" -> activity.launchActivity { it.action = Settings.ACTION_HARD_KEYBOARD_SETTINGS }
                "spellCheckerSettings" -> activity.launchActivity {
                    it.addCategory(android.content.Intent.CATEGORY_DEFAULT)
                    it.component = ComponentName("com.android.settings", "com.android.settings.Settings\$SpellCheckersSettingsActivity")
                }
                else -> error("unknown settings intent")
            }
            // AboutScreen's version row: the text is the version, never anything the user typed.
            "copy" -> try {
                activity.clipboardManager().value.addNewPlaintext(p[1])
                Toast.makeText(activity, p[2], Toast.LENGTH_SHORT).show()
            } catch (e: Throwable) {
                Toast.makeText(activity, p[3].replace("{error_message}", e.message.toString()), Toast.LENGTH_SHORT).show()
            }
            // ForgetLearnedWordsDialog's confirm: erase, then the toast Rust worded.
            "forgetLearnedWords" -> {
                activity.nlpManager().value.forgetLearnedWords()
                Toast.makeText(activity, p[1], Toast.LENGTH_SHORT).show()
            }
            else -> return false
        }
        return true
    }

    companion object {
        private val SPELL_KEYS = listOf("spell_checker_enabled", "selected_spell_checker")

        /**
         * SpellCheckerServiceSelector's `ComponentName.unflattenFromString(id)!!.packageName`, or
         * "null" when that throws: the text before the first '/', which must not be the last
         * character. Spelled out so the JVM tests run the same parse the device does.
         */
        fun spellPackage(id: String?): String {
            val sep = id?.indexOf('/') ?: -1
            return if (id == null || sep < 0 || sep + 1 >= id.length) "null" else id.substring(0, sep)
        }

        /** ModelStatus's two states as facts; the file names only for a damaged model, as it shows. */
        fun modelPairs(speech: ModelState, voice: ModelState, checked: Boolean): List<String> =
            listOf("speech" to speech, "voice" to voice).flatMap { (name, state) ->
                when (state) {
                    ModelState.Ready -> listOf("${name}_state", "ready")
                    is ModelState.Missing -> listOf("${name}_state", "missing")
                    is ModelState.Damaged -> listOf("${name}_state", "damaged", "${name}_files", state.files.joinToString(", "))
                }
            } + listOf("models_checked", checked.toString())

        /** [pairs] from the values Android reports; `label` looks an app's label up, null when it fails. */
        fun factPairs(
            versionName: String, versionCode: Int, packageName: String, keyboardAttached: Boolean,
            spellEnabled: String?, spellId: String?, label: (String) -> String?,
        ): Array<String> {
            val out = mutableListOf("version_name", versionName, "version_code", versionCode.toString(),
                "package", packageName, "physical_keyboard", keyboardAttached.toString())
            if (spellEnabled != null) out += listOf("spell_enabled", spellEnabled)
            if (spellId != null) out += listOf("spell_id", spellId)
            val pkg = spellPackage(spellId)
            out += listOf("spell_package", pkg)
            label(pkg)?.let { out += listOf("spell_label", it) }
            return out.toTypedArray()
        }
    }
}
