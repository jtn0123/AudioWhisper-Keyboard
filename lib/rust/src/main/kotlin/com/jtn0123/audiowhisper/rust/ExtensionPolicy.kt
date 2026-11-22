package com.jtn0123.audiowhisper.rust

/** Platform-neutral policy results. Android maps resource keys to its existing UI/error classes. */
object ExtensionPolicy {
    /** An [ExtensionValidationPlan] row. */
    fun validate(rule: ExtensionRule, value: String): List<String> = call(ExtensionOp.VALIDATE, rule.code, value)
    /** An [ExtensionMaintainerRanges] row, or nothing. */
    fun maintainerRanges(value: String): IntArray = call(ExtensionOp.MAINTAINER_RANGES, value).map(String::toInt).toIntArray()
    fun formatMaintainer(name: String, email: String?, url: String?): String =
        call(ExtensionOp.FORMAT_MAINTAINER, name, email.orEmpty(), url.orEmpty())[0]
    /** An [ExtensionComponentPlan] row. */
    fun componentName(value: String): List<String> = call(ExtensionOp.COMPONENT_NAME, value)

    data class ArchiveEntry(val name: String, val canonicalPath: String, val size: Long, val directory: Boolean) {
        /** This entry as an [ExtensionArchiveEntry] row. */
        fun packet() = with(ExtensionArchiveEntry) { row(NAME to name, CANONICAL_PATH to canonicalPath, BYTES to "$size", DIRECTORY to WireFlag.encode(directory)) }
    }
    /** An [ExtensionArchiveAction] for each entry. */
    fun archive(root: String, entries: List<ArchiveEntry>): List<String> =
        call(ExtensionOp.ARCHIVE, root, *entries.flatMap { it.packet().asList() }.toTypedArray())
    /** Canonical JSON from the platform syntax reader; returns an [ExtensionManifest] row. */
    fun manifest(metadata: String, canonical: String = metadata): List<String> = call(ExtensionOp.MANIFEST, metadata, canonical)
    fun group(serialType: String): String = call(ExtensionOp.GROUP, serialType).let {
        check(it[ExtensionGroupPlan.ERROR].isEmpty()) { it[ExtensionGroupPlan.ERROR] }
        it[ExtensionGroupPlan.GROUP]
    }
    fun accepts(requested: String, serialType: String): Boolean = WireFlag.decode(call(ExtensionOp.ACCEPTS, requested, serialType)[0])
    /** An [ExtensionSkipReason], or empty. */
    fun skipReason(supported: Boolean, parsed: Boolean, core: Boolean): String =
        call(ExtensionOp.SKIP_REASON, WireFlag.encode(supported), WireFlag.encode(parsed), WireFlag.encode(core))[0]
    fun archiveNames(names: List<String>): List<Boolean> = call(ExtensionOp.ARCHIVE_NAMES, *names.toTypedArray()).map(WireFlag::decode)
    /** An [ExtensionThemePlan] row. */
    fun themeComponent(id: String, label: String, stylesheet: String, authors: List<String>): List<String> =
        call(ExtensionOp.THEME_COMPONENT, id, label, stylesheet, *authors.toTypedArray())
    fun stylesheet(id: String, path: String?): String = call(ExtensionOp.STYLESHEET, id, path.orEmpty())[0]
    /** An [ExtensionComposerType]. */
    fun composerType(serialType: String?): String = call(ExtensionOp.COMPOSER_TYPE, serialType.orEmpty())[0]
    fun metadataValid(id: String, version: String, title: String, maintainers: String, license: String): Boolean =
        WireFlag.decode(call(ExtensionOp.METADATA_VALID, id, version, title, maintainers, license)[0])

    private fun call(operation: ExtensionOp, vararg input: String): List<String> =
        ExtensionPolicyNative.call(operation.code, arrayOf(*input)).asList()
}
