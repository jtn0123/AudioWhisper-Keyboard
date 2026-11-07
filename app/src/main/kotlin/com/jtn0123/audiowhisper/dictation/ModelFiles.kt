package com.jtn0123.audiowhisper.dictation

import android.content.Context
import java.io.File

/**
 * Where on-device models live: the app's private files directory. Until
 * there is an in-app download, they are copied in over adb with run-as
 * (debug builds only). Not shared storage: files adb writes there are owned
 * by the shell user and hidden from the app.
 */
object ModelFiles {
    /** Parakeet TDT 0.6B v2, int8: the most accurate model that fits the phone. */
    const val PARAKEET_V2 = "sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8"

    const val ENCODER = "encoder.int8.onnx"
    const val DECODER = "decoder.int8.onnx"
    const val JOINER = "joiner.int8.onnx"
    const val TOKENS = "tokens.txt"
    private val REQUIRED = listOf(ENCODER, DECODER, JOINER, TOKENS)

    fun dir(context: Context, id: String = PARAKEET_V2): File =
        File(File(context.filesDir, "models"), id)

    /** Required files that are not there yet; empty when the model is ready. */
    fun missing(context: Context, id: String = PARAKEET_V2): List<String> =
        REQUIRED.filterNot { File(dir(context, id), it).isFile }
}
