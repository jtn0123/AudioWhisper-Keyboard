package com.jtn0123.audiowhisper.stores

import com.jtn0123.audiowhisper.rust.ClipFlag
import com.jtn0123.audiowhisper.rust.StoredClip
import com.jtn0123.audiowhisper.rust.StoredWord
import com.jtn0123.audiowhisper.rust.WordFlag
import java.io.File
import java.security.MessageDigest
import kotlin.io.path.createTempDirectory
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.boolean
import kotlinx.serialization.json.int
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.long

/*
 * The canonical rows the stores' goldens hash, exactly as main's StoresFixtureWriter (scripts/stores-fixtures)
 * hashed them: fields as `length:text`, `~` for none, joined by `|`; a list's rows joined by newlines, SHA-256.
 * The goldens hold hashes, so a mismatch names a step and never prints a row. The device tests compile this too.
 */

internal fun key(vararg fields: Any?) = fields.joinToString("|") { if (it == null) "~" else it.toString().let { s -> "${s.length}:$s" } }

internal fun sha(text: String): String = MessageDigest.getInstance("SHA-256").digest(text.toByteArray(Charsets.UTF_8))
    .joinToString("") { "%02x".format(it) }

internal fun digest(rows: List<String>) = sha(rows.joinToString("\n"))

private fun StoredClip.has(flag: ClipFlag) = flags and flag.code != 0
private fun StoredClip.bit(flag: ClipFlag) = if (has(flag)) 1 else 0

/** A stored clip's row as main's ClipboardItem gave it: id, type, text, URI, time, pinned, MIME types, sensitive,
 * remote. */
internal fun clipKey(c: StoredClip) = key(c.id, c.kind, c.text.takeIf { c.has(ClipFlag.HAS_TEXT) },
    c.uri.takeIf { c.has(ClipFlag.HAS_URI) }, c.created, c.bit(ClipFlag.PINNED), c.mimeTypes, c.bit(ClipFlag.SENSITIVE),
    c.bit(ClipFlag.REMOTE))

/** A stored word's row as main's UserDictionaryEntry gave it: id, word, frequency, locale, shortcut. */
internal fun wordKey(w: StoredWord) = key(w.id, w.word, w.frequency,
    w.locale.takeIf { w.flags and WordFlag.HAS_LOCALE.code != 0 }, w.shortcut.takeIf { w.flags and WordFlag.HAS_SHORTCUT.code != 0 })

internal fun golden(name: String): JsonObject =
    Json.parseToJsonElement(checkNotNull(object {}.javaClass.getResource("/stores/$name")) { "no golden $name" }.readText())
        .jsonObject

internal fun JsonObject.long(name: String) = getValue(name).jsonPrimitive.long
internal fun JsonObject.int(name: String) = getValue(name).jsonPrimitive.int
internal fun JsonObject.bool(name: String) = getValue(name).jsonPrimitive.boolean
internal fun JsonObject.string(name: String) = getValue(name).jsonPrimitive.content
internal fun JsonObject.stringOrNull(name: String) = get(name)?.takeUnless { it is JsonNull }?.jsonPrimitive?.content
internal fun JsonObject.array(name: String): JsonArray = getValue(name).jsonArray
internal fun JsonObject.obj(name: String): JsonObject = getValue(name).jsonObject
internal val JsonElement.obj: JsonObject get() = jsonObject

/** Runs [block] in a new directory, removed after. */
internal fun <T> inTempDir(block: (File) -> T): T {
    val dir = createTempDirectory("aw-stores").toFile()
    try {
        return block(dir)
    } finally {
        dir.deleteRecursively()
    }
}
