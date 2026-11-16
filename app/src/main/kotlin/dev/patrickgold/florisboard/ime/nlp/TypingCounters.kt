package dev.patrickgold.florisboard.ime.nlp

import android.util.Log
import com.jtn0123.audiowhisper.suggest.PrivateTextFile
import java.io.File

/**
 * AudioWhisper: the private accuracy counters' file. The NLP session counts (see
 * lib/rust/crates/nlp/src/counters.rs); this passes it the events only Kotlin sees and keeps the
 * counts in the app's private files, where `run-as` can read them (docs/rust.md). Integers only:
 * no word, character or position is ever in it. Never backed up.
 */
internal class TypingCounters(filesDir: File) {
    private val file = PrivateTextFile(File(filesDir, FILE))

    /** Reads the saved counts. Blocking. */
    fun load() {
        NlpState.call("tallyLoad", file.read())
    }

    /** Whether a count changed since this was last asked. */
    fun changed(): Boolean = NlpState.call("tallyChanged")[0] == "1"

    /** Writes the counts. Blocking. */
    fun save() {
        if (!file.write(NlpState.call("tallyText")[0])) Log.w(TAG, "Could not save the typing counters")
    }

    /** Every count back to zero, on disk too. Blocking. */
    fun reset() {
        NlpState.call("tallyReset")
        file.delete()
    }

    /** A key that types went down: a character, a space or Enter. */
    fun key() = tally("key")

    /** A backspace, with the selection at [start]..[end] before it deleted anything. */
    fun deleted(start: Int, end: Int) = tally("deleted", start, end)

    /** A backspace straight after an auto-correction put back what was typed. */
    fun undone() = tally("undone")

    /** The undo key. */
    fun undo() = tally("undo")

    private fun tally(event: String, start: Int = 0, end: Int = 0) {
        NlpState.call("tally", event, start.toString(), end.toString())
    }

    companion object {
        /** In filesDir, outside the backup rules' include list, like learned-words.txt. */
        const val FILE = "typing-counters.txt"
        private const val TAG = "TypingCounters"
    }
}
