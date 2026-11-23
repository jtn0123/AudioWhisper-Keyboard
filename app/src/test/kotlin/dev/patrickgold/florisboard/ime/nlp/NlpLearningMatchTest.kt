package dev.patrickgold.florisboard.ime.nlp

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.KeptWordStore
import com.jtn0123.audiowhisper.rust.NlpLearnPlan
import com.jtn0123.audiowhisper.rust.NlpOp
import com.jtn0123.audiowhisper.rust.NlpSession
import com.jtn0123.audiowhisper.rust.WireFlag
import com.jtn0123.audiowhisper.suggest.LearnedWords
import com.jtn0123.audiowhisper.suggest.ReferenceLatin
import com.jtn0123.audiowhisper.suggest.WordLearner
import java.util.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/** Frozen manager/provider decisions plus the pre-port learner, over real native stores. */
class NlpLearningMatchTest {
    @Test fun finishedSwipedUndoAndPrivateBlockingHaveIdenticalStoreEffects() {
        var effects = 0
        val tape = GoldenTape.ofReference("nlp-learning")
        repeat(200) { seed ->
            val random = Random(20261113L + seed)
            val oldStore = LearnedWords(null)
            val newStore = LearnedWords(null)
            val old = ReferenceLatin.WordLearner(oldStore, { null })
            val actual = WordLearner(newStore) { null }
            val keptOld = KeptWordStore(); val keptNew = KeptWordStore()
            NlpSession().use { nlp ->
                repeat(250) {
                    val raw = listOf(1, 0x81, 0x91, 0xe1, 0x11, 0x21, 0xd1, 0xb1, 0x80001, 0x8021, 0x8011)[random.nextInt(11)]
                    val incognito = random.nextBoolean()
                    val enabled = random.nextBoolean()
                    val setting = random.nextBoolean()
                    val ready = random.nextBoolean()
                    val english = random.nextBoolean()
                    val valid = random.nextBoolean()
                    val word = listOf("zorblax", "Zorb", "GLORP", "mp3", "", "can't")[random.nextInt(6)]
                    val context = arrayOf(raw.toString(), "0", "org.florisboard.nlp.providers.latin", if (english) "en" else "fr",
                        WireFlag.encode(enabled), "1", WireFlag.encode(setting), WireFlag.encode(incognito), "1", "1", "8", "8", "2", if (valid) "8" else "2",
                        "a zorblax", "seed", "", WireFlag.encode(ready), "1", "1", "0", "0")
                    nlp.call(NlpOp.CONTEXT, *context)
                    val password = raw and 0xff0 in setOf(0x80, 0x90, 0xe0)
                    // Not prose: a URL, an e-mail address or a filter, even one asking for corrections.
                    val address = raw and 0xff0 in setOf(0x10, 0x20, 0xb0, 0xd0)
                    val prose = ready && english && enabled && !password && !address && raw and 0x80000 == 0
                    val mayLearn = !incognito && setting && prose
                    val action = random.nextInt(6)
                    val trigger = if (random.nextBoolean()) -1 else 39
                    if (action <= 2) {
                        if (action == 1) { old.dropSwiped(); actual.dropSwiped() }
                        val expected = if (!mayLearn || action == 0 && (trigger == 39 || word.isEmpty() || !valid)) emptyList()
                        else listOf(action.toString(), word, if (action == 2 || action == 1 && !valid) "a zorb " else "a ", if (valid) "2" else "8")
                        val plan = nlp.call(NlpOp.LEARN, action.toString(), trigger.toString(), word, "a ", "2", WireFlag.encode(valid), "a zorb").toList()
                        assertEquals(expected, plan, "learn plan seed $seed")
                        if (expected.isNotEmpty()) {
                            val before = expected[2]; val start = expected[3].toInt()
                            val a = if (action == 2) old.swiped(word, before) else old.finished(word, before, start)
                            val b = if (action == 2) actual.swiped(plan[NlpLearnPlan.WORD], plan[NlpLearnPlan.BEFORE])
                                else actual.finished(plan[NlpLearnPlan.WORD], plan[NlpLearnPlan.BEFORE], plan[NlpLearnPlan.START].toInt())
                            tape.answer("learned", a)
                            assertEquals(a, b)
                        }
                    } else if (action == 3 && mayLearn) {
                        val undone = old.undo(2)
                        tape.answer("undo", undone)
                        assertEquals(undone, actual.undo(2))
                    } else if (action == 4) { old.reset(); actual.reset(); nlp.call(NlpOp.FIELD) }
                    else if (action == 5) {
                        val remembered = WireFlag.decode(nlp.call(NlpOp.KEEP, word)[0])
                        assertEquals(keptOld.add(word, !incognito && prose), keptNew.add(word, remembered))
                    }
                    tape.answer("saved", oldStore.store.savedText())
                    assertEquals(oldStore.store.savedText(), newStore.store.savedText(), "learned effects seed $seed")
                    assertEquals(keptOld.savedText(), keptNew.savedText(), "kept effects seed $seed")
                    effects++
                }
            }
        }
        assertEquals(50_000, effects)
        tape.finish()
        println("NLP learning: 50000 events, 200 seeds, identical learned and blocked stores")
    }
}
