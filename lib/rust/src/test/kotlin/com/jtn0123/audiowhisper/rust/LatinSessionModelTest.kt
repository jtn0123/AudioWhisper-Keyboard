package com.jtn0123.audiowhisper.rust

import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

/** The n-gram model the app ships, attached to a session the way the app attaches it. */
class LatinSessionModelTest {
    private val model = File("../../app/src/main/assets/dicts/main_en-US.lm")
    private val personal = PersonalWordStore(emptyList(), IntArray(0), emptyList())

    private fun session() = requireNotNull(LatinSession.open(TestDictionaries.shipped.readBytes()))

    /** The strip after a space, for the words before it, newest first, after a sentence start. */
    private fun LatinSession.next(vararg newestFirst: String): List<String> =
        suggest(personal, null, "", newestFirst.asList(), true, false, true, FloatArray(0), 3, SuggestRules.defaultStrength).words

    @Test fun theShippedModelPredictsFromTheWordsBefore() {
        session().use { session ->
            val dictionaryOnly = session.next("you", "Can")
            assertEquals(dictionaryOnly, session.next("you", "See"))
            assertTrue(session.attachModel(model.path, 0, model.length()))
            assertEquals("tell", session.next("you", "Can").first())
            assertNotEquals(session.next("you", "Can"), session.next("you", "See"))
            assertEquals("you", session.next("Thank").first())
        }
    }

    @Test fun aMissingOrDamagedModelIsRefusedAndChangesNothing() {
        session().use { session ->
            val before = session.next("you", "Can")
            assertFalse(session.attachModel(model.path, 0, model.length() - 1))
            assertFalse(session.attachModel(model.path, 4, model.length() - 4))
            assertFalse(session.attachModel(model.path, 0, model.length() + 1))
            assertFalse(session.attachModel(model.path, -1, model.length()))
            assertFalse(session.attachModel(model.path, 0, Long.MAX_VALUE))
            assertFalse(session.attachModel(model.path, Long.MAX_VALUE, 8))
            assertFalse(session.attachModel(model.path + ".missing", 0, 64))
            assertFalse(session.attachModel(TestDictionaries.shipped.path, 0, 64))
            assertEquals(before, session.next("you", "Can"))
        }
    }
}
