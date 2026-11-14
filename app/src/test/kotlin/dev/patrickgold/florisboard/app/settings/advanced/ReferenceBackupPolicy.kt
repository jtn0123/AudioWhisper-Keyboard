package dev.patrickgold.florisboard.app.settings.advanced

/** Frozen branches from BackupScreen/RestoreScreen and ClipboardManager before package 18.
 * Android/file effects are recorded as pairs, not performed. No native calls. */
object ReferenceBackupPolicy {
    fun metadata(packageName: String, versionCode: Int, current: Int): List<String> {
        val warning = when {
            versionCode != current -> "different_version"
            !packageName.startsWith("dev.patrickgold.florisboard") -> "different_vendor"
            else -> ""
        }
        val error = when {
            packageName.isBlank() || versionCode < 64 -> "invalid_metadata"
            else -> ""
        }
        return listOf(warning, error)
    }
    fun fileName(packageName: String, versionCode: Int, timestamp: Long) = "backup_${packageName}_${versionCode}_${timestamp}.zip"
    fun layout(preferences: String) = listOf(preferences, "files/ime/keyboard", "files/ime/theme",
        "clipboard/clipboard_text_items.json", "clipboard/clipboard_images.json", "clipboard/clipboard_video.json")
    fun restore(mask: Int, erase: Boolean, preferences: String, present: List<Boolean>): List<List<String>> {
        val paths = layout(preferences)
        val out = mutableListOf<List<String>>()
        if (mask and 1 != 0 && present[0]) out += listOf(if (erase) "preferences-erase" else "preferences-merge", paths[0])
        if (mask and 2 != 0) {
            if (erase) out += listOf("clear-directory", paths[1])
            if (present[1]) out += listOf("copy-directory", paths[1])
        }
        if (mask and 4 != 0) {
            if (erase) out += listOf("clear-directory", paths[2])
            if (present[2]) out += listOf("copy-directory", paths[2])
        }
        if (erase) out += listOf("clear-clipboard", "")
        if (mask and 8 != 0 && present[3]) out += listOf("text", paths[3])
        if (mask and 16 != 0 && present[4]) out += listOf("image", paths[4])
        if (mask and 32 != 0 && present[5]) out += listOf("video", paths[5])
        return out
    }
    fun manifest(mask: Int, preferences: String) = layout(preferences).mapIndexedNotNull { i, path ->
        if (mask and (1 shl i) != 0) listOf("$i", path) else null
    }
    fun incoming(existing: List<Boolean>) = existing.indices.filter { !existing[it] }
    fun selection(mask: Int): List<String> {
        val selected = List(6) { mask and (1 shl it) != 0 }
        val clips = selected.drop(3)
        return listOf(if (selected.any { it }) "1" else "0", if (clips.any { it }) "1" else "0",
            if (clips.none { it }) "off" else if (clips.all { it }) "on" else "mixed")
    }
}
