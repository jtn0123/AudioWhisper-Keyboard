package dev.patrickgold.florisboard.app

import dev.patrickgold.jetpref.datastore.model.PreferenceData
import org.junit.jupiter.api.Test
import java.io.File

class SettingsBaselineCaptureTest {
    @Test fun captureSyntheticDefaults() {
        val model = FlorisPreferenceModelImpl()
        val lines = model.declaredPreferenceEntries.values.map { raw ->
            val p = raw
            fun <T : Any> encoded(p: PreferenceData<T>) = p.serializer.serialize(p.default)
            listOf(p.key, p.type.toString(), encoded(p), (p.default as? Enum<*>)?.javaClass?.enumConstants?.joinToString("|") { it.name } ?: "").joinToString("\t")
        }
        File("../build/stage2-settings/baseline-defaults.tsv").apply { parentFile?.mkdirs(); writeText(lines.joinToString("\n")) }
    }
}
