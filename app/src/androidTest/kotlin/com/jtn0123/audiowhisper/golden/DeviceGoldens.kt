package com.jtn0123.audiowhisper.golden

import android.app.Instrumentation
import android.os.Bundle
import java.io.File
import java.io.FileNotFoundException
import java.io.InputStream

/**
 * Golden files on a device (docs/phase6-kotlin-references.md): read from the test APK's assets,
 * `app/src/androidTest/assets/golden/<name>.txt`, and, when the instrumentation is given
 * `-e referenceGolden write`, written by the reference's tapes to the app's
 * `files/golden/<name>.txt`, where `scripts/device-goldens.py` pulls them from.
 */
class DeviceGoldens(private val instrumentation: Instrumentation, arguments: Bundle?) : GoldenTape.Store {
    override val request: String? = arguments?.getString(ARGUMENT)
    override val where = "the test APK's assets"

    override fun read(name: String): InputStream? =
        try {
            instrumentation.context.assets.open("golden/$name.txt")
        } catch (_: FileNotFoundException) {
            null
        }

    override fun write(name: String, text: String) {
        val file = File(instrumentation.targetContext.filesDir, "golden/$name.txt")
        checkNotNull(file.parentFile).mkdirs()
        file.writeText(text)
    }

    companion object {
        /** The instrumentation argument that asks the reference's tapes to write their golden files. */
        const val ARGUMENT = "referenceGolden"
    }
}
