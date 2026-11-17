package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.settings.SettingsDefaults
import dev.patrickgold.jetpref.datastore.runtime.FileBasedStorage
import dev.patrickgold.jetpref.datastore.runtime.LoadStrategy
import dev.patrickgold.jetpref.datastore.runtime.PersistStrategy
import kotlinx.coroutines.runBlocking
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * The Rust keyboard and Rust settings are the defaults (docs/decisions.md, "Rust draws by
 * default"). A stored choice still wins: the switches keep their keys, and Jetpref stores only
 * values someone set, so a user who turned one off keeps it off.
 */
class RustByDefaultTest {
    private val keyboard = "keyboard__rust_renderer_prototype"
    private val settings = "other__rust_settings_prototype"
    private val directory = File("../build/rust-by-default").apply { mkdirs() }

    private fun load(name: String, lines: List<String>): Pair<FlorisPreferenceModelImpl, String> = runBlocking {
        val before = File(directory, "$name-before.prefs").apply { writeText(lines.joinToString("") { "$it\n" }) }
        val model = FlorisPreferenceModelImpl()
        val store = SettingsStoreBridge.store(model)
        store.init(LoadStrategy.UseReader(FileBasedStorage(before.absolutePath)), PersistStrategy.Disabled).getOrThrow()
        val after = File(directory, "$name-after.prefs")
        store.export(FileBasedStorage(after.absolutePath)).getOrThrow()
        model to after.readText()
    }

    @Test fun bothSwitchesDefaultToRust() {
        assertTrue(SettingsDefaults.boolean(keyboard))
        assertTrue(SettingsDefaults.boolean(settings))
        val model = FlorisPreferenceModelImpl()
        runBlocking { SettingsStoreBridge.store(model).init(LoadStrategy.Disabled, PersistStrategy.Disabled).getOrThrow() }
        assertTrue(model.keyboard.rustRendererPrototype.get())
        assertTrue(model.other.rustSettingsPrototype.get())
    }

    @Test fun aFreshInstallDrawsInRustAndStoresNoChoice() {
        val (model, stored) = load("fresh", listOf("b;internal__is_ime_set_up;true"))
        assertTrue(model.keyboard.rustRendererPrototype.get())
        assertTrue(model.other.rustSettingsPrototype.get())
        // Nothing pins the new default into the file, so a later default would still apply.
        assertFalse(keyboard in stored)
        assertFalse(settings in stored)
    }

    @Test fun anExplicitFalseIsKept() {
        val (model, stored) = load("off", listOf("b;$keyboard;false", "b;$settings;false"))
        assertFalse(model.keyboard.rustRendererPrototype.get())
        assertFalse(model.other.rustSettingsPrototype.get())
        assertTrue("b;$keyboard;false" in stored.lines())
        assertTrue("b;$settings;false" in stored.lines())
    }

    @Test fun anExplicitTrueIsKept() {
        val (model, stored) = load("on", listOf("b;$keyboard;true", "b;$settings;true"))
        assertTrue(model.keyboard.rustRendererPrototype.get())
        assertTrue(model.other.rustSettingsPrototype.get())
        assertEquals(listOf("b;$keyboard;true", "b;$settings;true"), stored.lines().filter { "rust_" in it })
    }
}
