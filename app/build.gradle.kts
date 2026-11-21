/*
 * Copyright (C) 2022-2025 The FlorisBoard Contributors
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

import org.jetbrains.kotlin.gradle.dsl.JvmDefaultMode
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import java.util.Properties
import java.io.InputStream
import java.security.MessageDigest
import java.util.zip.ZipFile

plugins {
    alias(libs.plugins.agp.application)
    alias(libs.plugins.kotlin.plugin.compose)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.ksp)
    alias(libs.plugins.mikepenz.aboutlibraries)
}

val projectMinSdk = providers.gradleProperty("projectMinSdk").get()
val projectTargetSdk = providers.gradleProperty("projectTargetSdk").get()
val projectCompileSdk = providers.gradleProperty("projectCompileSdk").get()
val projectVersionCode = providers.gradleProperty("projectVersionCode").get()
val projectVersionName = providers.gradleProperty("projectVersionName").get()
val projectVersionNameSuffix = projectVersionName.substringAfter("-", "").let { suffix ->
    if (suffix.isNotEmpty()) {
        "-$suffix"
    } else {
        suffix
    }
}

// AudioWhisper: GIF search needs a GIPHY API key. It is read from local.properties
// (giphy.apiKey=...) or the GIPHY_API_KEY environment variable, never committed;
// without one the keyboard has no GIF tab.
val giphyApiKey: String = rootProject.file("local.properties")
    .takeIf { it.exists() }
    ?.let { file -> Properties().apply { file.inputStream().use { load(it) } }.getProperty("giphy.apiKey") }
    ?: System.getenv("GIPHY_API_KEY")
    ?: ""
require(giphyApiKey.all { it.isLetterOrDigit() }) { "giphy.apiKey must be letters and digits only" }

// The old JNI/Kotlin dependency is an oracle only in explicit comparison builds.
val sherpaOracle = (providers.gradleProperty("dictationFixture").orNull == "true" ||
    providers.gradleProperty("ttsFixture").orNull == "true") &&
    providers.gradleProperty("jniFreeFixture").orNull != "true"

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_11)
        // Replaces the deprecated -Xjvm-default=all-compatibility.
        jvmDefault.set(JvmDefaultMode.ENABLE)
        freeCompilerArgs.set(listOf(
            "-opt-in=kotlin.contracts.ExperimentalContracts",
        ))
    }
}

android {
    namespace = "dev.patrickgold.florisboard"
    compileSdk = projectCompileSdk.toInt()
    buildToolsVersion = tools.versions.buildTools.get()
    ndkVersion = tools.versions.ndk.get()

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    ksp {
        arg("room.schemaLocation", "$projectDir/schemas")
        arg("room.incremental", "true")
        arg("room.expandProjection", "true")
    }

    defaultConfig {
        // AudioWhisper Keyboard: a personal fork, installed next to stock FlorisBoard.
        applicationId = "com.jtn0123.audiowhisper.keyboard"
        // Built for one phone (Android 17), so the app needs Android 16 (API 36): that is what
        // lets it use Locale.of and drop window colours, haptic settings and version checks for
        // Android it never runs on. See projectMinSdk in gradle.properties.
        minSdk = projectMinSdk.toInt()
        targetSdk = projectTargetSdk.toInt()
        versionCode = projectVersionCode.toInt()
        versionName = projectVersionName.substringBefore("-")

        testInstrumentationRunner = when {
            providers.gradleProperty("settingsFixture").orNull == "true" -> "com.jtn0123.audiowhisper.settingsui.SettingsInstrumentation"
            providers.gradleProperty("ttsFixture").orNull == "true" -> "com.jtn0123.audiowhisper.reader.TtsInstrumentation"
            providers.gradleProperty("dictationFixture").orNull == "true" -> "com.jtn0123.audiowhisper.dictation.DictationInstrumentation"
            providers.gradleProperty("rendererFixture").orNull == "true" -> "com.jtn0123.audiowhisper.render.FixtureInstrumentation"
            providers.gradleProperty("suggestFixture").orNull == "true" -> "com.jtn0123.audiowhisper.suggest.A1Instrumentation"
            else -> "androidx.test.runner.AndroidJUnitRunner"
        }

        buildConfigField("String", "BUILD_COMMIT_HASH", "\"${getGitCommitHash().get()}\"")
        buildConfigField("String", "FLADDONS_API_VERSION", "\"v~draft2\"")
        buildConfigField("String", "FLADDONS_STORE_URL", "\"beta.addons.florisboard.org\"")
        buildConfigField("String", "GIPHY_API_KEY", "\"$giphyApiKey\"")

        // The phone is arm64; the other ABIs in the sherpa-onnx AAR would
        // only add size. Lint's ChromeOsAbiSupport warning is expected and
        // left showing (warnings aren't suppressed here): x86_64 would mean
        // building the Rust, LatinIME's C++ and sherpa-onnx twice and about
        // 30 MB more APK for devices this app isn't for (even the emulators
        // used for testing are arm64).
        ndk { abiFilters += "arm64-v8a" }
    }

    packaging {
        jniLibs {
            // Normal builds package only the pinned C API and onnxruntime payload below.
            excludes += "**/libsherpa-onnx-cxx-api.so"
            // Preserve the checksum-authenticated payload bytes, including ORT ELF padding.
            keepDebugSymbols += setOf("**/libsherpa-onnx-c-api.so", "**/libonnxruntime.so")
            if (!sherpaOracle) excludes += "**/libsherpa-onnx-jni.so"
        }
    }

    bundle {
        language {
            // We disable language split because FlorisBoard does not use
            // runtime Google Play Service APIs and thus cannot dynamically
            // request to download the language resources for a specific locale.
            enableSplit = false
        }
    }

    buildFeatures {
        buildConfig = true
        compose = true
        resValues = true
    }

    buildTypes {
        named("debug") {
            versionNameSuffix = "-debug+${getGitCommitHash(short = true).get()}"

            isDebuggable = true
            isJniDebuggable = false

            // AudioWhisper: the debug build is the one on the phone day to day, so it is shrunk
            // like release. Names aren't obfuscated (proguard-rules.pro), and R8 compiles a
            // debuggable build in its debug mode, keeping line numbers and locals for the
            // debugger.
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            if (providers.gradleProperty("suggestFixture").orNull == "true") proguardFile("suggest-fixture.pro")
            if (providers.gradleProperty("dictationFixture").orNull == "true") proguardFile("dictation-fixture.pro")
            if (providers.gradleProperty("ttsFixture").orNull == "true") proguardFile("tts-fixture.pro")
            if (providers.gradleProperty("rendererFixture").orNull == "true") proguardFile("acceptance-fixture.pro")
            if (providers.gradleProperty("settingsFixture").orNull == "true") proguardFile("settings-fixture.pro")
            isMinifyEnabled = true
            isShrinkResources = true

            resValue("string", "floris_app_name", "AudioWhisper Keyboard")
        }

        create("beta") {
            applicationIdSuffix = ".beta"
            versionNameSuffix = projectVersionNameSuffix

            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            isMinifyEnabled = true
            isShrinkResources = true

            resValue("string", "floris_app_name", "AudioWhisper Keyboard Beta")
        }

        named("release") {
            versionNameSuffix = projectVersionNameSuffix

            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            isMinifyEnabled = true
            isShrinkResources = true

            resValue("string", "floris_app_name", "AudioWhisper Keyboard")
        }

        // AudioWhisper: the build to use every day. Release's shrinking and optimising, not
        // debuggable, and signed with the debug key so it installs over the debug build (and
        // back) keeping the app's data. run-as doesn't work on it: see scripts/install-dev.sh.
        create("personal") {
            initWith(getByName("release"))
            versionNameSuffix = "-personal+${getGitCommitHash(short = true).get()}"
            signingConfig = signingConfigs.getByName("debug")
            matchingFallbacks += listOf("release")
        }

        create("benchmark") {
            initWith(getByName("release"))
            signingConfig = signingConfigs.getByName("debug")
            matchingFallbacks += listOf("release")
        }
    }

    aboutLibraries {
        collect {
            configPath = file("src/main/config")
        }
    }

    androidResources {
        // The dictionary and the n-gram model are read straight out of the APK at their
        // offsets, which needs the entries stored uncompressed.
        noCompress += "dict"
        noCompress += "lm"
        // The app is English only; this also leaves out the libraries' translations.
        // Typing languages are unaffected: layouts and dictionaries are assets.
        localeFilters += "en"

    }

    testOptions {
        unitTests {
            isIncludeAndroidResources = true
        }
        unitTests.all {
            it.useJUnitPlatform()
        }
    }
    if (providers.gradleProperty("settingsFixture").orNull == "true") {
        sourceSets.getByName("debug").kotlin.directories.add("src/settingsFixture/kotlin")
        sourceSets.getByName("debug").manifest.srcFile("src/settingsFixture/AndroidManifest.xml")
    }
    sourceSets.getByName("androidTest").kotlin.directories.add("src/test/kotlin/com/jtn0123/audiowhisper/suggest/a1")
    sourceSets.getByName("androidTest").kotlin.directories.add("src/test/kotlin/com/jtn0123/audiowhisper/gif/shared")
    sourceSets.getByName("androidTest").kotlin.directories.add("src/test/kotlin/dev/patrickgold/florisboard/lib/ext/replay")
    sourceSets.getByName("androidTest").java.directories.add("src/test/java/com/android/inputmethod")
    // The tracked Roboto-Light the Rust text tests use, so the device can record Android's layout of it.
    sourceSets.getByName("androidTest").assets.directories.add("../fastlane/staging/fonts")
}

tasks.withType<Test> {
    useJUnitPlatform()
    // Tests that reach Rust load the computer's build of libawnative (see lib/rust). It's an
    // input, so changing the Rust runs them again.
    val rustHost = rootDir.resolve("lib/rust/build/rust/host")
    dependsOn(":lib:rust:cargoBuildHost")
    inputs.dir(rustHost).withPropertyName("rustHostLibrary").withPathSensitivity(PathSensitivity.RELATIVE)
    systemProperty("java.library.path", rustHost.path)
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}

// AndroidX Test needs two runtime annotations. The full Error Prone compiler-annotation
// jar references javax.lang.model, which Android cannot supply to R8. Package only the
// runtime annotation classes actually used by the test runner; no missing-class suppression.
val fixtureAnnotationSource = configurations.create("fixtureAnnotationSource")
val fixtureAnnotations = tasks.register<Jar>("fixtureAnnotations") {
    from({ zipTree(fixtureAnnotationSource.singleFile) }) {
        include("com/google/errorprone/annotations/CanIgnoreReturnValue.class")
        include("com/google/errorprone/annotations/MustBeClosed.class")
    }
    archiveFileName.set("fixture-runtime-annotations.jar")
    destinationDirectory.set(layout.buildDirectory.dir("fixture-annotations"))
}

// A source set absent from normal APKs. Its shell receiver changes only controlled UI facts.
if (providers.gradleProperty("dictationFixture").orNull == "true") {
    android.sourceSets.getByName("debug") {
        kotlin.directories.add("src/dictationFixture/test/kotlin")
        manifest.srcFile("src/dictationFixture/AndroidManifest.xml")
    }
}

dependencies {
    // Enforced so the BOM's stable Compose versions win over transitive pre-releases: material-kolor 5
    // asks for material3 1.5.0-alpha22, and every material3 API it calls already exists in the BOM's 1.4.0.
    val composeBom = enforcedPlatform(libs.androidx.compose.bom)
    implementation(composeBom)
    // testImplementation(composeBom)
    // androidTestImplementation(composeBom)

    implementation(libs.androidx.activity.compose)
    // Version/checksum-pinned C payload is extracted below; no sherpa classes ship normally.
    androidTestImplementation(files("libs/sherpa-onnx-2.0.1.aar"))
    if (sherpaOracle) implementation(files("libs/sherpa-onnx-2.0.1.aar"))
    implementation(libs.androidx.activity.ktx)
    implementation(libs.androidx.autofill)
    implementation(libs.androidx.collection.ktx)
    implementation(libs.androidx.compose.material.icons)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.runtime.livedata)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.core.splashscreen)
    implementation(libs.androidx.emoji2)
    implementation(libs.androidx.exifinterface)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.profileinstaller)
    ksp(libs.androidx.room.compiler)
    implementation(libs.androidx.room.runtime)
    implementation(libs.cache4k)
    implementation(libs.kotlin.reflect)
    implementation(libs.kotlinx.coroutines)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.mikepenz.aboutlibraries.core)
    implementation(libs.mikepenz.aboutlibraries.compose)
    implementation(libs.patrickgold.compose.tooltip)
    implementation(libs.patrickgold.jetpref.datastore.model)
    ksp(libs.patrickgold.jetpref.datastore.model.processor)
    implementation(libs.patrickgold.jetpref.datastore.ui)
    implementation(libs.patrickgold.jetpref.material.ui)

    implementation(projects.lib.android)
    implementation(projects.lib.color)
    implementation(projects.lib.compose)
    implementation(projects.lib.kotlin)
    // AOSP LatinIME dictionary engine (Apache-2.0): word suggestions and autocorrect.
    // The keyboard's Rust code (libawnative): glide typing's scorer.
    implementation(projects.lib.rust)
    testImplementation(testFixtures(projects.lib.rust))
    androidTestImplementation(testFixtures(projects.lib.rust))
    implementation(projects.lib.snygg)

    testImplementation(libs.kotlin.test.junit5)
    testImplementation(libs.icu4j)
    // SettingsLicencesMatchTest runs AboutLibraries' AndroidParser (org.json) and the TagSoup
    // release Html.fromHtml parses with.
    testImplementation(libs.org.json)
    testImplementation(libs.tagsoup)
    fixtureAnnotationSource(libs.errorprone.annotations)
    androidTestImplementation(files(fixtureAnnotations))
    androidTestImplementation(libs.androidx.test.ext)
    androidTestImplementation(libs.androidx.test.runner)
}

fun getGitCommitHash(short: Boolean = false): Provider<String> {
    if (!File(".git").exists()) {
        return providers.provider { "null" }
    }

    val execProvider = providers.exec {
        if (short) {
            commandLine("git", "rev-parse", "--short", "HEAD")
        } else {
            commandLine("git", "rev-parse", "HEAD")
        }
    }
    return execProvider.standardOutput.asText.map { it.trim() }
}

// C API, ORT and the test-only JNI oracle come from the same checksum-pinned release.
val verifySherpa = tasks.register("verifySherpa") {
    val aar = layout.projectDirectory.file("libs/sherpa-onnx-2.0.1.aar")
    inputs.file(aar)
    doLast {
        fun digest(input: InputStream): String = input.use { stream ->
            val hash = MessageDigest.getInstance("SHA-256")
            val buffer = ByteArray(1 shl 20)
            while (true) { val count = stream.read(buffer); if (count < 0) break; hash.update(buffer, 0, count) }
            hash.digest().joinToString("") { "%02x".format(it) }
        }
        check(digest(aar.asFile.inputStream()) == "a8d09d1d32f3eb8520482c6d3d43fae674681c5b050385967c70a3c2c9bf0ee7")
        ZipFile(aar.asFile).use { zip ->
            check(digest(zip.getInputStream(zip.getEntry("jni/arm64-v8a/libsherpa-onnx-c-api.so"))) ==
                "0eadae93c4d595a26584d7eebde7b397b9a36d01eee3ebc7262e0d34df2fce10")
        }
    }
}
tasks.named("preBuild") { dependsOn(verifySherpa) }

// No repackaged/downloaded binary: extract exactly these two files after authenticating the AAR.
abstract class ExtractSherpaC : DefaultTask() {
    @get:InputFile abstract val aar: RegularFileProperty
    @get:OutputDirectory abstract val outputDirectory: DirectoryProperty
    @TaskAction fun extract() {
        val output = outputDirectory.get().asFile
        ZipFile(aar.get().asFile).use { zip ->
            for (name in listOf("libsherpa-onnx-c-api.so", "libonnxruntime.so")) {
                val target = output.resolve("arm64-v8a/$name")
                target.parentFile.mkdirs()
                zip.getInputStream(zip.getEntry("jni/arm64-v8a/$name")).use { input ->
                    target.outputStream().use { input.copyTo(it) }
                }
            }
        }
    }
}
val extractSherpaC = tasks.register<ExtractSherpaC>("extractSherpaC") {
    dependsOn(verifySherpa)
    aar.set(layout.projectDirectory.file("libs/sherpa-onnx-2.0.1.aar"))
    outputDirectory.set(layout.buildDirectory.dir("generated/sherpa-c"))
}
if (!sherpaOracle) {
    androidComponents.onVariants { variant ->
        variant.sources.jniLibs?.addGeneratedSourceDirectory(extractSherpaC) { it.outputDirectory }
    }
}

// Check the APK itself, including unused production declarations that R8 may discard, and the
// JNI types of every Rust entry point against its declaration's JVM descriptor.
val checkNativeDeclarations = tasks.register<Exec>("checkNativeDeclarations") {
    group = "verification"
    description = "Rejects production native declarations missing from the normal debug APK, " +
        "or whose Rust entry points take or return other JNI types."
    dependsOn("assembleDebug")
    workingDir(rootDir)
    // The descriptors come from each module's debug classes. lib/rust's build generates the
    // Kotlin of the generated boundary, and keeps a copy of the Rust entry points it compiled.
    val modules = rootProject.subprojects.flatMap { listOf("--module", it.projectDir.relativeTo(rootDir).path) }
    commandLine(
        listOf("python3", "-I", "scripts/native-declarations-check.py", "app/build/outputs/apk/debug/app-debug.apk") +
            modules +
            listOf("--generated-kotlin", "lib/rust/build/generated/nativeBoundary/kotlin") +
            listOf("--generated-rust", "lib/rust/build/rust/boundary/arm64-v8a.rs"),
    )
}
tasks.named("check") { dependsOn(checkNativeDeclarations) }
