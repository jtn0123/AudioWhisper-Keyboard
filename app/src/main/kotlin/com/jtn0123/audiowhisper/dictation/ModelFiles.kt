package com.jtn0123.audiowhisper.dictation

import android.content.Context
import java.io.File

/**
 * Where on-device models live: the app's private files directory. Until
 * there is an in-app download, they are copied in over adb with run-as,
 * which needs the debug build; the personal build installs over it and keeps
 * them (scripts/install-dev.sh). Not shared storage, even the app's own
 * Android/data folder: files adb writes there are owned by the shell user and
 * hidden from the app.
 */
object ModelFiles {
    /** Parakeet TDT 0.6B v2, int8: the most accurate model that fits the phone. */
    const val PARAKEET_V2 = "sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8"

    const val ENCODER = "encoder.int8.onnx"
    const val DECODER = "decoder.int8.onnx"
    const val JOINER = "joiner.int8.onnx"
    const val TOKENS = "tokens.txt"

    /** sherpa-onnx's release of the model (also csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8). */
    val EXPECTED = listOf(
        ExpectedFile(ENCODER, 652_282_309, "92a10bd9c23936ea0566c29f963bc57c2a5df4261b6a8921c39abea66557ca8b"),
        ExpectedFile(DECODER, 7_257_753, "4d097bae3bac5c96018d5293ba39f1b657296102b142affda40378f38b020f41"),
        ExpectedFile(JOINER, 1_739_080, "77a29f3e5ac7d37074e0af465391b80f314015d5b97f0f90c6c7767f1d56f428"),
        ExpectedFile(TOKENS, 9_384, "ec182b70dd42113aff6c5372c75cac58c952443eb22322f57bbd7f53977d497d"),
    )

    fun dir(context: Context, id: String = PARAKEET_V2): File =
        File(File(context.filesDir, "models"), id)

    /** Names and sizes only, so cheap enough for the main thread. */
    fun quickState(context: Context): ModelState = ModelCheck.quick(dir(context), EXPECTED)

    /** Also checks each file's SHA-256, once per file. Blocking: off the main thread only. */
    fun fullState(context: Context): ModelState =
        ModelCheck.full(dir(context), EXPECTED, ModelCheck.verifiedFiles(context))
}
