package com.jtn0123.audiowhisper.rust

/** Copied plans only. SAF, ZIP, preferences and clipboard objects stay on the managed side. */
object BackupPolicy {
    private fun call(op: Int, vararg input: String) = BackupNative.call(op, arrayOf(*input)).asList()
    private val names by lazy { call(8) }
    val metadataName get() = names[0]
    val missingMetadata get() = names[1]
    val mediaDirectory get() = names[2]
    val inputArchiveName get() = names[3]
    fun metadata(packageName: String, version: Int, current: Int) = call(0, packageName, "$version", "$current")
    fun layout(preferences: String) = call(1, preferences)
    fun restore(mask: Int, erase: Boolean, preferences: String, present: List<Boolean>) =
        call(2, "$mask", erase.bit(), preferences, *present.map { it.bit() }.toTypedArray()).chunked(2)
    fun fileName(packageName: String, version: Int, timestamp: Long) = call(3, packageName, "$version", "$timestamp")[0]
    fun manifest(mask: Int, preferences: String) = call(4, "$mask", preferences).chunked(2)
    fun items(kind: String, types: List<String>) = call(5, kind, *types.toTypedArray()).map(String::toInt)
    fun incoming(existing: List<Boolean>) = call(6, *existing.map { it.bit() }.toTypedArray()).map(String::toInt)
    fun selection(mask: Int) = call(7, "$mask")
    fun archive(root: String, entries: List<ExtensionPolicy.ArchiveEntry>) = call(9, root,
        *entries.flatMap { listOf(it.name, it.canonicalPath, "${it.size}", it.directory.bit()) }.toTypedArray())
    private fun Boolean.bit() = if (this) "1" else "0"
}
internal object BackupNative {
    init { RustLibrary.load() }
    @JvmStatic external fun call(operation: Int, input: Array<String>): Array<String>
}
