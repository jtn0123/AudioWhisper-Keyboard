package com.jtn0123.audiowhisper.render

import android.content.Context
import com.jtn0123.audiowhisper.rust.ClipFlag
import com.jtn0123.audiowhisper.rust.StoredClip
import dev.patrickgold.florisboard.FlorisApplication
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardStoreFile
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.io.FlorisRef
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import java.io.File
import java.security.MessageDigest
import org.florisboard.lib.kotlin.mimeTypeFilterOf
import org.json.JSONArray
import org.json.JSONObject

/** Test APK only. Preference values and synthetic utility strings leave only hashes. */
object AcceptanceProbe {
    @JvmStatic fun changeBackupSettings() = kotlinx.coroutines.runBlocking {
        val prefs by FlorisPreferenceStore
        prefs.keyboard.heightFactorPortrait.set(113).getOrThrow()
        prefs.correction.autoCapitalization.set(false).getOrThrow()
        prefs.theme.dayThemeId.set(dev.patrickgold.florisboard.lib.ext.ExtensionComponentName(
            "org.florisboard.themes", "floris_night")).getOrThrow()
    }

    @JvmStatic fun learnedFixture(context: Context) {
        val file = File(context.filesDir, "learned-words.txt")
        val words = com.jtn0123.audiowhisper.suggest.LearnedWords(file)
        words.load()
        val before = words.knownSpellings().size
        repeat(4) { words.add("awbackupfixture", null, false) }
        check(words.save())
        val report = JSONObject().put("before_known", before).put("after_known", words.knownSpellings().size)
            .put("sha256", hash(file.readText()))
        File(context.filesDir, "learned-fixture.json").writeText(report.toString())
    }

    private fun hash(value: String) = MessageDigest.getInstance("SHA-256")
        .digest(value.toByteArray(Charsets.UTF_8)).joinToString("") { "%02x".format(it) }

    private fun <T : Any> value(pref: PreferenceData<T>): JSONObject {
        val raw = pref.serializer.serialize(pref.get())
        return JSONObject().put("is_null", raw == null).put("sha256", hash(raw.orEmpty()))
            .put("utf16_units", raw?.length ?: 0)
    }

    @JvmStatic fun clipboard(context: Context) {
        val rows = JSONArray()
        // The order the Room query gave: newest first, equal times newest row first.
        val clips = ClipboardStoreFile.get(context).items().sortedWith(compareByDescending<StoredClip> { it.created }
            .thenByDescending { it.id })
        for (clip in clips) {
            check(clip.text in (1..4).map { "aw-test-$it" })
            rows.put(JSONObject().put("sha256", hash(clip.text)).put("type", clip.kind)
                .put("pinned", clip.flags and ClipFlag.PINNED.code != 0).put("mime", clip.mimeTypes))
        }
        val pending = File(context.filesDir, "clipboard-probe.json.tmp")
        pending.writeText(rows.toString())
        check(pending.renameTo(File(context.filesDir, "clipboard-probe.json")))
    }

    @JvmStatic fun snapshot(context: Context) {
        val app = context.applicationContext as FlorisApplication
        check(app.preferenceStoreLoaded.value)
        val prefs by FlorisPreferenceStore
        val values = JSONObject()
        for ((key, pref) in prefs.declaredPreferenceEntries.entries.sortedBy { it.key.key }) {
            values.put(key.key, value(pref))
        }
        val mime = JSONArray()
        for (pattern in listOf("*/*", "text/*", "application/zip", "application/font-*", "text/pl.in")) {
            val filter = mimeTypeFilterOf(pattern)
            for (input in listOf(null, "", "text/plain", "image/png", "application/zip", "text/", "/plain")) {
                mime.put(filter.matches(input))
            }
            mime.put(filter.matchesAny(listOf("image/png", "text/plain")))
            mime.put(filter.matchesAll(listOf("text/plain", "text/html")))
            mime.put(filter.matchesOne(listOf("text/plain", "image/png")))
        }
        val locales = JSONArray()
        for (tag in listOf("en-US", "fr_FR", "tr-TR", "zh-Hant-TW", "", "en_US_POSIX")) {
            val locale = FlorisLocale.fromTag(tag)
            locales.put(hash(listOf(locale.languageTag(), locale.localeTag(),
                locale.displayName(FlorisLocale.ENGLISH)).joinToString("\n")))
        }
        val refs = JSONArray()
        for (input in listOf("assets:/ime/test.json", "internal:/synthetic/%20?q#r",
            "cache:fixture", "florisboard://assets/path", "content://synthetic/item", "https://example.invalid/a%20b")) {
            val ref = FlorisRef.from(input)
            refs.put(hash(listOf(ref.uri.toString(), ref.scheme, ref.authority, ref.relativePath,
                ref.isValid, ref.isExternal, ref.isAssets, ref.isInternal, ref.isCache).joinToString("\n")))
        }
        val report = JSONObject().put("preferences", values).put("preference_count", values.length())
            .put("active_subtype_id", app.subtypeManager.value.activeSubtype.id)
            .put("subtype_count", app.subtypeManager.value.subtypes.size)
            .put("mime", mime).put("locales", locales).put("refs", refs)
        val pending = File(context.filesDir, "acceptance-probe.json.tmp")
        pending.writeText(report.toString(2))
        check(pending.renameTo(File(context.filesDir, "acceptance-probe.json")))
    }
}
