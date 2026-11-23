package com.jtn0123.audiowhisper.dictation

import java.io.File
import java.io.IOException
import java.security.MessageDigest

/**
 * ModelCheck as main had it at b1cf45f18, before its decisions moved to Rust
 * (dictation::model_check), so ModelCheckTraceTest can hold the new code to it. Copied with only
 * the object renamed and verifiedFiles, which needs Android, and its preferences' name left out. Don't change it.
 */
internal object ReferenceModelCheck {
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
            try {
                verified.contains(key) || (sha256(on) == file.sha256).also { if (it) verified.add(key) }
            } catch (_: IOException) {
                false // Unreadable or removed during the check; never cache a failed read.
            }
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

    private const val BUFFER_BYTES = 1 shl 20
}
