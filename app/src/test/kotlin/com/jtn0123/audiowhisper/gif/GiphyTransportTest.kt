package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifRequest
import com.jtn0123.audiowhisper.rust.GifSessionPlan
import com.sun.net.httpserver.HttpServer
import kotlinx.coroutines.runBlocking
import java.io.File
import java.net.InetAddress
import java.net.InetSocketAddress
import java.nio.file.Files
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * [Giphy] makes each request as Rust's `GifRequest` row says, against a server on this machine's
 * loopback (never GIPHY): the status asked for is read and nothing else, a body past its cap is
 * refused without being held, and a download lands whole in the file asked for.
 */
class GiphyTransportTest {
    private val server = HttpServer.create(InetSocketAddress(InetAddress.getLoopbackAddress(), 0), 0).apply {
        createContext("/") { exchange ->
            val (status, body) = replies.getValue(exchange.requestURI.path)
            exchange.sendResponseHeaders(status, if (body.isEmpty()) -1 else body.size.toLong())
            exchange.responseBody.use { it.write(body) }
        }
        start()
    }
    private val replies = HashMap<String, Pair<Int, ByteArray>>()
    private val giphy = Giphy("unused")
    private val api = GifSessionPlan.request(0)

    private fun url(path: String) = "http://127.0.0.1:${server.address.port}$path"

    @AfterTest fun stop() = server.stop(0)

    @Test
    fun `each kind of request has its own cap, a timeout and the status it reads`() {
        val media = (0..2).map { GifSessionPlan.request(it) }
        assertEquals(listOf(2, 4, 8).map { it * 1024 * 1024 }, media.map { it[GifRequest.MAX_BYTES] })
        assertEquals(listOf(10_000, 15_000, 15_000), media.map { it[GifRequest.TIMEOUT_MS] })
        assertEquals(listOf(200, 200, 200), media.map { it[GifRequest.OK_STATUS] })
    }

    @Test
    fun `reads the body of an answer with the status asked for`() = runBlocking {
        val body = "{\"title\":\"café\"}".encodeToByteArray()
        replies["/ok"] = 200 to body
        val answer = giphy.get(url("/ok"), api)
        assertEquals(200, answer.status)
        assertContentEquals(body, answer.body)
    }

    @Test
    fun `any other status is read without its body`() = runBlocking {
        for (status in listOf(201, 401, 404, 429, 500)) {
            replies["/s$status"] = status to "refused".encodeToByteArray()
            val answer = giphy.get(url("/s$status"), api)
            assertEquals(status, answer.status)
            assertNull(answer.body)
        }
    }

    @Test
    fun `a body one byte past its cap is refused`() = runBlocking {
        val cap = intArrayOf(1_000, 10_000, 200)
        replies["/at"] = 200 to ByteArray(1_000)
        replies["/past"] = 200 to ByteArray(1_001)
        assertEquals(1_000, giphy.get(url("/at"), cap).body?.size)
        assertEquals(1_000, assertFailsWith<TooLargeException> { giphy.get(url("/past"), cap) }.maxBytes)
    }

    @Test
    fun `a download lands whole in its file, or says nothing was written`() = runBlocking {
        val dir = Files.createTempDirectory("gif-transport").toFile()
        val gif = ByteArray(5_000) { it.toByte() }
        replies["/gif"] = 200 to gif
        replies["/gone"] = 404 to ByteArray(0)
        val file = File(dir, "a.gif.part")
        assertTrue(giphy.save(url("/gif"), file, GifSessionPlan.request(2)))
        assertContentEquals(gif, file.readBytes())
        assertFalse(giphy.save(url("/gone"), File(dir, "b.gif.part"), GifSessionPlan.request(2)))
        assertFalse(File(dir, "b.gif.part").exists())
        assertTrue(dir.deleteRecursively())
    }
}
