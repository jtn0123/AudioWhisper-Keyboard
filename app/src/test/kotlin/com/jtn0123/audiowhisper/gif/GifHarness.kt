package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifRequest
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import java.io.File
import java.io.IOException
import kotlin.coroutines.CoroutineContext

/** A thread a test steps: tasks wait in order until [runAll]. */
class StepDispatcher : CoroutineDispatcher() {
    private val queue = ArrayDeque<Runnable>()

    override fun dispatch(context: CoroutineContext, block: Runnable) {
        queue.addLast(block)
    }

    /** Runs what is queued, and what that queues: how many ran. */
    fun runAll(): Int {
        var ran = 0
        while (queue.isNotEmpty()) {
            queue.removeFirst().run()
            ran++
        }
        return ran
    }
}

/** A main thread and an IO pool, both stepped; what any coroutine threw is kept in [crashes]. */
class StepThreads {
    val main = StepDispatcher()
    val io = StepDispatcher()
    val crashes = mutableListOf<Throwable>()
    val scope = CoroutineScope(SupervisorJob() + main + CoroutineExceptionHandler { _, e -> crashes += e })

    /** How many coroutines are still running: waiting, queued or about to be. */
    fun active(): Int = scope.coroutineContext[Job]!!.children.count { it.isActive }

    /** Runs both until neither has anything queued. */
    fun drain() {
        while (main.runAll() + io.runAll() > 0) Unit
    }
}

/** What a fake server answers: a status and a body, a body too long, or no answer at all. */
sealed interface FakeReply {
    data class Body(val status: Int, val bytes: ByteArray) : FakeReply
    data class Status(val status: Int) : FakeReply
    data object TooLarge : FakeReply
    data object Unreachable : FakeReply
}

/** One API request waiting for the test to answer it. */
class Pending(val url: String) {
    val reply = CompletableDeferred<FakeReply>()
}

/** The URLs of GIPHY's API, as opposed to its media servers. */
fun isApi(url: String) = url.startsWith("https://api.giphy.com/")

/**
 * The network as the new code sees it ([GifDataSource]): API requests wait for the test, media
 * answer at once with [media]. Records every URL asked for, in order.
 */
internal class FakeSource(override val key: String, private val media: (String) -> FakeReply) : GifDataSource {
    val api = mutableListOf<Pending>()
    val mediaAsked = mutableListOf<String>()

    override suspend fun get(url: String, request: IntArray): GifAnswer {
        val reply = if (isApi(url)) Pending(url).also { api += it }.reply.await() else media(url).also { mediaAsked += url }
        return when (reply) {
            is FakeReply.Body -> when {
                reply.status != request[GifRequest.OK_STATUS] -> GifAnswer(reply.status, null)
                reply.bytes.size > request[GifRequest.MAX_BYTES] -> throw TooLargeException(request[GifRequest.MAX_BYTES])
                else -> GifAnswer(reply.status, reply.bytes)
            }
            is FakeReply.Status -> GifAnswer(reply.status, null)
            FakeReply.TooLarge -> throw TooLargeException(request[GifRequest.MAX_BYTES])
            FakeReply.Unreachable -> throw IOException("unreachable")
        }
    }

    override suspend fun save(url: String, file: File, request: IntArray): Boolean {
        val answer = get(url, request)
        val body = answer.body ?: return false
        file.writeBytes(body)
        return true
    }
}

/** The app a GIF is sent into: what it takes, and every call it got. */
internal class FakeTarget(var acceptsGifs: Boolean, var takesGif: Boolean, var takesText: Boolean) : GifTarget {
    val calls = mutableListOf<String>()

    override fun acceptsGifs() = acceptsGifs.also { calls += "accepts" }

    override fun commitGif(file: File, gif: Gif) = takesGif.also { calls += "gif ${file.name} ${file.length()}" }

    override fun commitText(text: String) = takesText.also { calls += "text ${text.length}" }
}

/** One [GifSearchSession] over a [FakeSource], stepped by the test, at [now]. */
internal class SessionRig(media: (String) -> FakeReply = { FakeReply.Status(404) }, available: Boolean = true) {
    var now = 1_000L
    val threads = StepThreads()
    val source = FakeSource("KEY123", media)
    val logs = mutableListOf<String>()
    var forgotten = 0
    val session = GifSearchSession(threads.scope, threads.io, { now }, { logs += it }, { forgotten++ }, source.takeIf { available })

    /** Answers the last request with [reply], and runs what follows. */
    fun answer(reply: FakeReply) {
        threads.drain()
        source.api.last().reply.complete(reply)
        threads.drain()
    }

    /** [search]'s results once GIPHY answered with [reply]. */
    fun results(search: String, reply: FakeReply): GifSearch.Results {
        session.load(search)
        answer(reply)
        return session.results.value
    }
}
