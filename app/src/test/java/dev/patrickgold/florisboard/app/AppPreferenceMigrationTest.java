package dev.patrickgold.florisboard.app;

import dev.patrickgold.jetpref.datastore.model.PreferenceMigrationEntry;
import dev.patrickgold.jetpref.datastore.model.PreferenceType;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;

// jetpref's migration entry is a JVM API internal to its Kotlin datastore module.
// Java exercises that API without loading Android or bypassing Kotlin's visibility checks.
class AppPreferenceMigrationTest {
    // The Android and JVM artifacts give this Kotlin-internal getter different module suffixes.
    private static Object actionOf(PreferenceMigrationEntry entry) throws ReflectiveOperationException {
        for (var method : PreferenceMigrationEntry.class.getMethods()) {
            if (method.getName().startsWith("getAction$") && method.getParameterCount() == 0) {
                return method.invoke(entry);
            }
        }
        throw new NoSuchMethodException("jetpref migration action getter");
    }

    @Test
    void savedDeveloperFlagsAreDiscardedOnUpgrade() throws ReflectiveOperationException {
        for (String flag : new String[] {
            "enabled", "show_primary_clip", "show_input_state_overlay", "show_spelling_overlay",
            "show_inline_autofill_overlay", "show_touch_boundaries", "show_drag_and_drop_helpers",
        }) {
            var entry = new PreferenceMigrationEntry(PreferenceMigrationEntry.Action.KEEP_AS_IS,
                PreferenceType.Companion.from("b"), "devtools__" + flag, "true");
            var migrated = AppPreferenceMigrationKt.migrateAppPreference(entry);
            assertEquals(PreferenceMigrationEntry.Action.RESET, actionOf(migrated), flag);
        }
    }

    @Test
    void ordinarySavedPreferencesRetainTheirValues() throws ReflectiveOperationException {
        var entry = new PreferenceMigrationEntry(PreferenceMigrationEntry.Action.KEEP_AS_IS,
            PreferenceType.Companion.from("b"), "suggestion__enabled", "true");
        var migrated = AppPreferenceMigrationKt.migrateAppPreference(entry);
        assertEquals(entry.getKey(), migrated.getKey());
        assertEquals(entry.getRawValue(), migrated.getRawValue());
        assertEquals(PreferenceMigrationEntry.Action.KEEP_AS_IS, actionOf(migrated));
    }
}
