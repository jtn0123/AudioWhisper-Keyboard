package dev.patrickgold.florisboard.app;

import dev.patrickgold.jetpref.datastore.model.PreferenceMigrationEntry;
import dev.patrickgold.jetpref.datastore.model.PreferenceModel;
import dev.patrickgold.jetpref.datastore.model.PreferenceType;
import dev.patrickgold.jetpref.datastore.runtime.DataStore;

// Java exposes Jetpref's Kotlin-internal construction for the real JVM datastore tests,
// as in the retained AppPreferenceMigrationTest. Production has no visibility bypass.
public final class SettingsStoreBridge {
    private SettingsStoreBridge() {}
    public static <T extends PreferenceModel> DataStore<T> store(T model) { return new DataStore<>(model); }
    public static PreferenceMigrationEntry entry(String type, String key, String raw) {
        return new PreferenceMigrationEntry(PreferenceMigrationEntry.Action.KEEP_AS_IS,
            PreferenceType.Companion.from(type), key, raw);
    }
    public static String[] snapshot(PreferenceMigrationEntry entry) throws ReflectiveOperationException {
        Object action = null;
        for (var method : PreferenceMigrationEntry.class.getMethods()) {
            if (method.getName().startsWith("getAction$") && method.getParameterCount() == 0) {
                action = method.invoke(entry);
                break;
            }
        }
        if (action == null) throw new NoSuchMethodException("migration action");
        return new String[] {action.toString(), entry.getType().toString(), entry.getKey(), entry.getRawValue()};
    }
}
