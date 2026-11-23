package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifRequest
import kotlinx.coroutines.delay
import java.io.File
import java.net.URI
import java.net.URLDecoder
import java.security.MessageDigest

/**
 * Local GIFs for a fixture build (-PgifFixture=true) without a GIPHY key, so the panel can be
 * driven on an emulator with no network and no key; [GifFixtureReceiver] switches it in. [dir]
 * (files/gif-fixture, put there with `run-as`) holds:
 * `trending.json`; `search-<first 12 hex of the query's SHA-256>.json`, else `search.json`, for a
 * search; `media/<name>` for each preview and GIF, named by its URL's last segment; and
 * optionally `delay-ms`, how long each API answer takes, to cancel one by searching again. A
 * missing answer is a 404. Never in a normal APK.
 */
internal class GifFixtureSource private constructor(private val dir: File) : GifDataSource {
    override val key = "fixture"

    override suspend fun get(url: String, request: IntArray): GifAnswer {
        val file = answer(url) ?: return GifAnswer(MISSING, null)
        if (file.length() > request[GifRequest.MAX_BYTES]) throw TooLargeException(request[GifRequest.MAX_BYTES])
        return GifAnswer(request[GifRequest.OK_STATUS], file.readBytes())
    }

    override suspend fun save(url: String, file: File, request: IntArray): Boolean {
        val answer = get(url, request).body ?: return false
        file.writeBytes(answer)
        return true
    }

    private suspend fun answer(url: String): File? {
        val uri = URI(url)
        val file = if (uri.host == API_HOST) api(uri) else File(dir, "media/" + uri.path.substringAfterLast('/'))
        return file?.takeIf { it.isFile }
    }

    private suspend fun api(uri: URI): File? {
        delay(File(dir, "delay-ms").takeIf { it.isFile }?.readText()?.trim()?.toLongOrNull() ?: 0)
        if (uri.path.endsWith("/trending")) return File(dir, "trending.json")
        val query = uri.rawQuery.orEmpty().split('&').firstOrNull { it.startsWith("q=") } ?: return null
        val digest = MessageDigest.getInstance("SHA-256").digest(URLDecoder.decode(query.removePrefix("q="), "UTF-8").toByteArray())
        val name = "search-" + digest.take(6).joinToString("") { "%02x".format(it) } + ".json"
        return File(dir, name).takeIf { it.isFile } ?: File(dir, "search.json")
    }

    companion object {
        private const val API_HOST = "api.giphy.com"
        private const val MISSING = 404

        /** The fixture in [filesDir], if a test put one there. */
        fun open(filesDir: File): GifFixtureSource? =
            File(filesDir, "gif-fixture").takeIf { File(it, "trending.json").isFile }?.let(::GifFixtureSource)
    }
}
