import org.gradle.api.tasks.testing.logging.TestLogEvent
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

/*
 * Copyright (C) 2025 The FlorisBoard Contributors
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

plugins {
    alias(libs.plugins.agp.library)
    alias(libs.plugins.kotlin.plugin.compose)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.kotlinx.kover)
}

val projectMinSdk = providers.gradleProperty("projectMinSdk").get()
val projectCompileSdk = providers.gradleProperty("projectCompileSdk").get()

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_11)
        freeCompilerArgs.set(listOf(
            "-Xconsistent-data-class-copy-visibility",
        ))
    }
}

android {
    namespace = "org.florisboard.lib.snygg"
    compileSdk = projectCompileSdk.toInt()

    defaultConfig {
        minSdk = projectMinSdk.toInt()

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildFeatures {
        compose = true
    }
    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
        create("beta") {
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

tasks.withType<Test> {
    testLogging {
        events = setOf(TestLogEvent.FAILED, TestLogEvent.PASSED, TestLogEvent.SKIPPED)
    }
    useJUnitPlatform()
    systemProperty("snygg.repoRoot", rootDir.path)
    inputs.files(rootProject.fileTree(rootDir) {
        include("**/*.json", "**/*.flex", "**/*.zip", "**/*.floris")
        exclude("**/build/**", ".git/**", ".gradle/**", ".kotlin/**", ".device-backup/**")
    }).withPropertyName("repoStylesheets")
    val rustHost = rootDir.resolve("lib/rust/build/rust/host")
    dependsOn(":lib:rust:cargoBuildHost")
    inputs.dir(rustHost).withPropertyName("rustHostLibrary").withPathSensitivity(PathSensitivity.RELATIVE)
    systemProperty("java.library.path", rustHost.path)
    // The match tests compare exception messages. Once the JIT compiles a cast that keeps failing,
    // HotSpot throws a shared ClassCastException with no message (after about 6,300 throws on
    // JDK 24), so without this flag whether a message is null depends on test order and timing.
    jvmArgs("--enable-native-access=ALL-UNNAMED", "-XX:-OmitStackTraceInFastThrow")
}

kover {
    useJacoco()
}

dependencies {
    implementation(projects.lib.android)
    implementation(projects.lib.color)
    implementation(projects.lib.kotlin)
    implementation(projects.lib.rust)

    val composeBom = platform(libs.androidx.compose.bom)
    implementation(composeBom)
    // testImplementation(composeBom)
    // androidTestImplementation(composeBom)

    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.material.icons)
    debugImplementation(libs.androidx.compose.ui.tooling)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.coil.compose)
    implementation(libs.coil.gif)
    implementation(libs.kotlinx.serialization.json)

    testImplementation(libs.kotlin.test.junit5)
}

val generateJsonSchema = tasks.register<JavaExec>("generateJsonSchema") {
    dependsOn("build", ":lib:rust:cargoBuildHost")
    systemProperty("java.library.path", rootDir.resolve("lib/rust/build/rust/host").path)
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    mainClass.set("org.florisboard.lib.snygg.SnyggJsonSchemaGenerator")
    args = listOf("schemas/stylesheet.schema.json")
    workingDir = projectDir
    standardOutput = System.out
}

// The generator runs on the debug variant's compiled classes plus its compile classpath.
androidComponents {
    onVariants(selector().withName("debug")) { variant ->
        generateJsonSchema.configure {
            val kotlinCompile = tasks.named<KotlinCompile>("compileDebugKotlin")
            classpath = files(variant.compileClasspath, kotlinCompile.flatMap { it.destinationDirectory })
        }
    }
}

tasks["build"].finalizedBy("generateJsonSchema")
