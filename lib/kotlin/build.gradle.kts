/*
 * Copyright (C) 2024-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import org.gradle.api.tasks.testing.logging.TestLogEvent
import org.jetbrains.kotlin.gradle.dsl.JvmDefaultMode
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.kotlinx.kover)
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

sourceSets {
    maybeCreate("main").apply {
        java.srcDir("src/main/kotlin")
    }
}

// The operation codes and flags this module's own native calls use, generated from lib/rust's
// crates/wire by lib/rust's generateNativeBoundary (docs/rust.md, "The generated boundary").
sourceSets.named("main") {
    java.srcDir(
        files(project(":lib:rust").layout.buildDirectory.dir("generated/nativeBoundary/libKotlin"))
            .builtBy(":lib:rust:generateNativeBoundary"),
    )
}

// GoldenTape (docs/phase6-kotlin-references.md), compiled into these tests from its one file:
// it is lib/rust's test fixture, and that is an Android library's, which this JVM module cannot
// depend on.
sourceSets.named("test") {
    java.srcDir("../rust/src/testFixtures/test/kotlin/com/jtn0123/audiowhisper/golden")
}

tasks {
    compileKotlin {
        compilerOptions.jvmTarget = JvmTarget.JVM_11
        // Replaces the deprecated -Xjvm-default=all-compatibility.
        compilerOptions.jvmDefault = JvmDefaultMode.ENABLE
        compilerOptions.freeCompilerArgs = listOf(
            "-opt-in=kotlin.contracts.ExperimentalContracts",
        )
    }
    // JUnit 6 requires Java 17, so the tests (and with them the test classpaths, which Gradle
    // matches to compileTestJava's target) move to 17. The library itself still targets Java 11.
    compileTestJava {
        sourceCompatibility = "17"
        targetCompatibility = "17"
    }
    compileTestKotlin {
        compilerOptions.jvmTarget = JvmTarget.JVM_17
    }
}

tasks.withType<Test> {
    val rustHost = project(":lib:rust").layout.buildDirectory.dir("rust/host")
    dependsOn(":lib:rust:cargoBuildHost")
    inputs.dir(rustHost).withPropertyName("rustHostLibrary").withPathSensitivity(PathSensitivity.RELATIVE)
    systemProperty("java.library.path", rustHost.get().asFile.path)
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    testLogging {
        events = setOf(TestLogEvent.FAILED, TestLogEvent.PASSED, TestLogEvent.SKIPPED)
    }
    useJUnitPlatform()
}

kover {
    useJacoco()
}

dependencies {
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.kotlinx.coroutines)

    testImplementation(libs.junit.jupiter.params)
    testImplementation(libs.kotlin.test.junit5)
}

