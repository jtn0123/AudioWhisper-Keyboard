package com.jtn0123.audiowhisper.render

import dev.patrickgold.florisboard.app.FlorisPreferenceModel
import java.io.ByteArrayInputStream
import java.io.DataInputStream
import java.lang.reflect.Modifier
import java.nio.file.Files
import java.nio.file.Path
import java.util.zip.ZipFile
import kotlin.test.Test
import kotlin.test.assertTrue

/** Catches stale Compose callers before R8 packages an APK that fails only on first composition. */
class ComposeStabilityLinkageTest {
    private fun stabilityOwners(bytes: ByteArray): List<String> = DataInputStream(ByteArrayInputStream(bytes)).use { input ->
        require(input.readInt() == 0xcafebabe.toInt())
        input.readInt() // minor + major class-file version
        val count = input.readUnsignedShort()
        val tags = IntArray(count)
        val first = IntArray(count)
        val second = IntArray(count)
        val text = arrayOfNulls<String>(count)
        var index = 1
        while (index < count) {
            val tag = input.readUnsignedByte()
            tags[index] = tag
            when (tag) {
                1 -> text[index] = input.readUTF()
                3, 4 -> input.readInt()
                5, 6 -> { input.readLong(); index++ }
                7 -> first[index] = input.readUnsignedShort()
                8, 16, 19, 20 -> input.readUnsignedShort()
                9, 12 -> {
                    first[index] = input.readUnsignedShort()
                    second[index] = input.readUnsignedShort()
                }
                10, 11, 17, 18 -> input.readInt()
                15 -> { input.readUnsignedByte(); input.readUnsignedShort() }
                else -> error("Unknown JVM constant-pool tag $tag")
            }
            index++
        }
        (1 until count).filter { tags[it] == 9 && text[first[second[it]]] == "\$stable" }
            .map { text[first[first[it]]]!!.replace('/', '.') }
    }

    @Test fun everyReferencedComposeStabilityFieldExistsInTheActualCompiledClasses() {
        val location = Path.of(requireNotNull(FlorisPreferenceModel::class.java.protectionDomain).codeSource.location.toURI())
        val owners = mutableSetOf<String>()
        if (Files.isDirectory(location)) {
            Files.walk(location).use { paths ->
                paths.filter { it.toString().endsWith(".class") }.forEach { owners += stabilityOwners(Files.readAllBytes(it)) }
            }
        } else {
            ZipFile(location.toFile()).use { zip ->
                for (entry in zip.entries().asSequence().filter { it.name.endsWith(".class") }) {
                    owners += stabilityOwners(zip.getInputStream(entry).use { it.readBytes() })
                }
            }
        }
        assertTrue(owners.isNotEmpty(), "No production Compose classes were inspected")
        val loader = FlorisPreferenceModel::class.java.classLoader
        val missing = owners.filter { owner ->
            try {
                val field = Class.forName(owner, false, loader).getField("\$stable")
                field.type != Int::class.javaPrimitiveType || !Modifier.isStatic(field.modifiers)
            } catch (_: NoSuchFieldException) {
                true
            }
        }
        assertTrue(missing.isEmpty(), "Compiled Compose callers reference missing stability fields: $missing")
    }
}
