package dev.patrickgold.florisboard.ime.nlp

import android.util.Log
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.suggest.PrivateTextFile
import java.io.File

/**
 * AudioWhisper: the private accuracy counters' file. The NLP session counts, from each typing
 * event (see lib/rust/crates/nlp/src/counters.rs and typing.rs); this keeps the counts in the
 * app's private files, where `run-as` can read them (docs/rust.md). Integers only: no word,
 * character or position is ever in it. Never backed up.
 */
internal class TypingCounters(filesDir: File) {
    private val file = PrivateTextFile(File(filesDir, FILE))

    /** Reads the saved counts. Blocking. */
    fun load() {
        NlpState.call(NlpOp.TALLY_LOAD, file.read())
    }

    /** Writes the counts. Blocking. */
    fun save() {
        if (!file.write(NlpState.call(NlpOp.TALLY_TEXT)[0])) Log.w(TAG, "Could not save the typing counters")
    }

    /** Every count back to zero, on disk too. Blocking. */
    fun reset() {
        NlpState.call(NlpOp.TALLY_RESET)
        file.delete()
    }

    companion object {
        /** In filesDir, outside the backup rules' include list, like learned-words.txt. */
        const val FILE = "typing-counters.txt"
        private const val TAG = "TypingCounters"
    }
}
