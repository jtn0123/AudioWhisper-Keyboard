package com.jtn0123.audiowhisper.dictation

import android.content.Context
import androidx.core.content.edit
import com.jtn0123.audiowhisper.rust.ModelCheckRules
import com.jtn0123.audiowhisper.rust.ModelCheckState
import java.io.File
import java.io.IOException
import java.security.MessageDigest

/** A file a model needs: its name in the model's folder, its size and its SHA-256. */
class ExpectedFile(val name: String, val bytes: Long, val sha256: String)

/** Whether a model's files are all there and whole. */
sealed interface ModelState {
    data object Ready : ModelState

    /** Files not there at all: the model was never copied in, or only partly. */
    data class Missing(val files: List<String>) : ModelState

    /** Files there but the wrong size or content: a copy cut short, or another version. */
    data class Damaged(val files: List<String>) : ModelState
}

/** Remembers which files already passed the SHA-256 check, so each is hashed once. */
interface VerifiedFiles {
    fun contains(key: String): Boolean
    fun add(key: String)
}

/**
 * Checks a model's files against what they should be. [quick] only looks at names and sizes, so
 * it can run on the main thread; [full] also hashes each file once (650 MB takes a second or two)
 * and remembers it by path, size and modification time, so a file replaced later is hashed again.
 * What counts as missing or damaged, which files to hash and what to remember are Rust's
 * ([ModelCheckRules]); this looks at the files and hashes them.
 */
object ModelCheck {
    fun quick(dir: File, expected: List<ExpectedFile>): ModelState {
        val files = expected.map { File(dir, it.name) }
        val present = BooleanArray(files.size) { files[it].isFile }
        val lengths = LongArray(files.size) { files[it].length() }
        return state(ModelCheckRules.quick(LongArray(expected.size) { expected[it].bytes }, present, lengths), expected)
    }

    /** Blocking; call off the main thread. */
    fun full(dir: File, expected: List<ExpectedFile>, verified: VerifiedFiles): ModelState {
        quick(dir, expected).let { if (it != ModelState.Ready) return it }
        val files = expected.map { File(dir, it.name) }
        val keys = ModelCheckRules.keys(Array(files.size) { files[it].path }, LongArray(files.size) { files[it].length() },
            LongArray(files.size) { files[it].lastModified() })
        val known = BooleanArray(keys.size) { verified.contains(keys[it]) }
        val sums = arrayOfNulls<String>(keys.size)
        // Unreadable or removed during the check: null, which is never remembered.
        for (i in ModelCheckRules.toHash(known)) sums[i] = try { sha256(files[i]) } catch (_: IOException) { null }
        val verdict = ModelCheckRules.verdict(known, Array(expected.size) { expected[it].sha256 }, sums)
        for (at in 2 + verdict[1] until verdict.size) verified.add(keys[verdict[at]])
        return state(verdict, expected)
    }

    /** The state [answer] starts with: its code, the count of the files it names, then them. */
    private fun state(answer: IntArray, expected: List<ExpectedFile>): ModelState {
        val names = (2 until 2 + answer[1]).map { expected[answer[it]].name }
        return when (answer[0]) {
            ModelCheckState.READY -> ModelState.Ready
            ModelCheckState.MISSING -> ModelState.Missing(names)
            else -> ModelState.Damaged(names)
        }
    }

    fun sha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        val buffer = ByteArray(BUFFER_BYTES)
        file.inputStream().use { input ->
            generateSequence { input.read(buffer).takeIf { it >= 0 } }.forEach { digest.update(buffer, 0, it) }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    /** The files that passed, kept in the app's shared preferences. */
    fun verifiedFiles(context: Context): VerifiedFiles {
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        return object : VerifiedFiles {
            override fun contains(key: String) = prefs.getBoolean(key, false)
            override fun add(key: String) = prefs.edit { putBoolean(key, true) }
        }
    }

    private const val PREFS = "model-checks"
    private const val BUFFER_BYTES = 1 shl 20
}
