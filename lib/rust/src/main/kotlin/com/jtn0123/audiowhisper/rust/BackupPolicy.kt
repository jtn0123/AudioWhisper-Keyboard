package com.jtn0123.audiowhisper.rust

/** Copied plans only. SAF, ZIP, preferences and clipboard objects stay on the managed side. */
object BackupPolicy {
    private fun call(op: BackupOp, vararg input: String) = BackupNative.call(op.code, arrayOf(*input)).asList()
    private val names by lazy { call(BackupOp.NAMES) }
    val metadataName get() = names[BackupNames.METADATA]
    val missingMetadata get() = names[BackupNames.MISSING_METADATA]
    val mediaDirectory get() = names[BackupNames.MEDIA_DIRECTORY]
    val inputArchiveName get() = names[BackupNames.INPUT_ARCHIVE]
    /** A [BackupMetadataStatus] row. */
    fun metadata(packageName: String, version: Int, current: Int) = call(BackupOp.METADATA, packageName, "$version", "$current")
    /** A [BackupLayout] row of paths. */
    fun layout(preferences: String) = call(BackupOp.LAYOUT, preferences)
    /** A [BackupEffectRow] for each step. [present] has a flag for each [BackupLayout] group. */
    fun restore(mask: Int, erase: Boolean, preferences: String, present: List<Boolean>) = call(BackupOp.RESTORE,
        *with(BackupRestoreRequest) { row(MASK to "$mask", ERASE to WireFlag.encode(erase), PREFERENCES to preferences) },
        *present.map(WireFlag::encode).toTypedArray()).chunked(BackupEffectRow.SIZE)
    fun fileName(packageName: String, version: Int, timestamp: Long) = call(BackupOp.FILE_NAME, packageName, "$version", "$timestamp")[0]
    /** A [BackupManifestRow] for each selected group. */
    fun manifest(mask: Int, preferences: String) = call(BackupOp.MANIFEST, "$mask", preferences).chunked(BackupManifestRow.SIZE)
    fun items(kind: String, types: List<String>) = call(BackupOp.ITEMS, kind, *types.toTypedArray()).map(String::toInt)
    fun incoming(existing: List<Boolean>) = call(BackupOp.INCOMING, *existing.map(WireFlag::encode).toTypedArray()).map(String::toInt)
    /** A [BackupSelection] row. */
    fun selection(mask: Int) = call(BackupOp.SELECTION, "$mask")
    fun archive(root: String, entries: List<ExtensionPolicy.ArchiveEntry>) =
        call(BackupOp.ARCHIVE, root, *entries.flatMap { it.packet().asList() }.toTypedArray())
}
