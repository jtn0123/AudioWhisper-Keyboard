package com.jtn0123.audiowhisper.rust

/** Platform-neutral policy results. Android maps resource keys to its existing UI/error classes. */
object ExtensionPolicy {
    // Validation: status, resource key, optional argument name and regex.
    fun validate(rule: String, value: String): List<String> = ExtensionPolicyNative.call(0, arrayOf(rule, value)).asList()
    fun maintainerRanges(value: String): IntArray = ExtensionPolicyNative.call(1, arrayOf(value)).map(String::toInt).toIntArray()
    fun formatMaintainer(name: String, email: String?, url: String?): String =
        ExtensionPolicyNative.call(2, arrayOf(name, email.orEmpty(), url.orEmpty()))[0]
    fun componentName(value: String): List<String> = ExtensionPolicyNative.call(3, arrayOf(value)).asList()

    data class ArchiveEntry(val name: String, val canonicalPath: String, val size: Long, val directory: Boolean)
    fun archive(root: String, entries: List<ArchiveEntry>): List<String> = ExtensionPolicyNative.call(4,
        (listOf(root) + entries.flatMap { listOf(it.name, it.canonicalPath, it.size.toString(), if (it.directory) "1" else "0") }).toTypedArray()).asList()
    /** Canonical JSON from the platform syntax reader; returns serial type, group and component field. */
    fun manifest(metadata: String, canonical: String = metadata): List<String> = ExtensionPolicyNative.call(5, arrayOf(metadata, canonical)).asList()
    fun group(serialType: String): String = ExtensionPolicyNative.call(6, arrayOf(serialType)).let {
        check(it[0].isEmpty()) { it[0] }
        it[1]
    }
    fun accepts(requested: String, serialType: String): Boolean = ExtensionPolicyNative.call(7, arrayOf(requested, serialType))[0] == "1"
    fun skipReason(supported: Boolean, parsed: Boolean, core: Boolean): String =
        ExtensionPolicyNative.call(8, arrayOf(supported.bit(), parsed.bit(), core.bit()))[0]
    fun archiveNames(names: List<String>): List<Boolean> = ExtensionPolicyNative.call(9, names.toTypedArray()).map { it == "1" }
    /** Error, trimmed id/label, optional stylesheet and its presence bit, filtered authors. */
    fun themeComponent(id: String, label: String, stylesheet: String, authors: List<String>): List<String> =
        ExtensionPolicyNative.call(10, (listOf(id, label, stylesheet) + authors).toTypedArray()).asList()
    fun stylesheet(id: String, path: String?): String = ExtensionPolicyNative.call(11, arrayOf(id, path.orEmpty()))[0]
    fun composerType(serialType: String?): String = ExtensionPolicyNative.call(14, arrayOf(serialType.orEmpty()))[0]
    fun metadataValid(id: String, version: String, title: String, maintainers: String, license: String): Boolean =
        ExtensionPolicyNative.call(15, arrayOf(id, version, title, maintainers, license))[0] == "1"
    private fun Boolean.bit() = if (this) "1" else "0"
}

internal object ExtensionPolicyNative {
    init { RustLibrary.load() }
    @JvmStatic external fun call(operation: Int, input: Array<String>): Array<String>
}
