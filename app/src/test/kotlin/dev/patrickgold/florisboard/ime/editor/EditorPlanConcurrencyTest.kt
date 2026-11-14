package dev.patrickgold.florisboard.ime.editor

import android.view.inputmethod.InputConnection
import com.jtn0123.audiowhisper.rust.EditorSession
import dev.patrickgold.florisboard.ime.nlp.BreakIteratorGroup
import dev.patrickgold.florisboard.ime.text.composing.Appender
import dev.patrickgold.florisboard.lib.FlorisLocale
import java.lang.reflect.Proxy
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class EditorPlanConcurrencyTest {
    private fun connection() = Proxy.newProxyInstance(InputConnection::class.java.classLoader,
        arrayOf(InputConnection::class.java)) { _, method, _ ->
        when (method.name) {
            "getTextBeforeCursor" -> "abc"
            "getTextAfterCursor", "getSelectedText" -> ""
            else -> if (method.returnType == Boolean::class.javaPrimitiveType) true else null
        }
    } as InputConnection

    @Test fun `a suspended word reply retains its native analysis until the plan finishes`() = runBlocking {
        withTimeout(5_000) {
            supervisorScope {
                EditorSession().use { session ->
                    session.call("config", "0", "1", "1", "0", "1")
                    val entered = CompletableDeferred<Unit>()
                    val resume = CompletableDeferred<Unit>()
                    var words = 0
                    val executor = EditorPlanExecutor(BreakIteratorGroup(), { FlorisLocale.ENGLISH }, { Appender },
                        { _, _ ->
                            if (++words == 1) { entered.complete(Unit); resume.await() }
                            EditorRange(0, 3)
                        }, { 1L })
                    val first = async(start = CoroutineStart.UNDISPATCHED) {
                        executor.execute(session, connection(), "start", arrayOf("3", "3", "1"))
                    }
                    entered.await()
                    val second = async(start = CoroutineStart.UNDISPATCHED) {
                        executor.execute(session, connection(), "start", arrayOf("3", "3", "1"))
                    }
                    try {
                        assertFalse(second.isCompleted, "Another plan consumed a pending native analysis")
                    } finally { resume.complete(Unit) }
                    assertTrue(first.await())
                    assertTrue(second.await())
                    assertTrue(words == 2)
                }
            }
        }
    }
}
