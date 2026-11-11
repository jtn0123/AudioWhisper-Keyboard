package com.jtn0123.audiowhisper.dictation

import android.content.Context
import java.io.File
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
 */
object ModelCheck {
    fun quick(dir: File, expected: List<ExpectedFile>): ModelState {
        val missing = expected.filterNot { File(dir, it.name).isFile }
        if (missing.isNotEmpty()) return ModelState.Missing(missing.map { it.name })
        val wrongSize = expected.filter { File(dir, it.name).length() != it.bytes }
        return if (wrongSize.isEmpty()) ModelState.Ready else ModelState.Damaged(wrongSize.map { it.name })
    }

    /** Blocking; call off the main thread. */
    fun full(dir: File, expected: List<ExpectedFile>, verified: VerifiedFiles): ModelState {
        val quick = quick(dir, expected)
        if (quick != ModelState.Ready) return quick
        val damaged = expected.filterNot { file ->
            val on = File(dir, file.name)
            val key = "${on.path}:${on.length()}:${on.lastModified()}"
            verified.contains(key) || (sha256(on) == file.sha256).also { if (it) verified.add(key) }
        }
        return if (damaged.isEmpty()) ModelState.Ready else ModelState.Damaged(damaged.map { it.name })
    }

    fun sha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buffer = ByteArray(BUFFER_BYTES)
            while (true) {
                val count = input.read(buffer)
                if (count < 0) break
                digest.update(buffer, 0, count)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    /** The files that passed, kept in the app's shared preferences. */
    fun verifiedFiles(context: Context): VerifiedFiles {
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        return object : VerifiedFiles {
            override fun contains(key: String) = prefs.getBoolean(key, false)
            override fun add(key: String) = prefs.edit().putBoolean(key, true).apply()
        }
    }

    private const val PREFS = "model-checks"
    private const val BUFFER_BYTES = 1 shl 20
}
