/*
 * AudioWhisper Keyboard: the keyboard's Rust code, which cargo builds into one library,
 * libawnative.so, and the Kotlin that loads and calls it. The Rust workspace lives here too
 * (Cargo.toml, crates/); docs/rust.md explains the layout and the rules.
 *
 * cargo builds the library twice: for the phone, packaged as this module's arm64-v8a native
 * library, and for this computer, which the JVM unit tests here and in :app load.
 */

import org.gradle.process.ExecOperations
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import javax.inject.Inject

plugins {
    alias(libs.plugins.agp.library)
}

val projectMinSdk = providers.gradleProperty("projectMinSdk").get()
val projectNativeMinSdk = providers.gradleProperty("projectNativeMinSdk").get()
val projectCompileSdk = providers.gradleProperty("projectCompileSdk").get()

// One definition is compiled by Rust and generates the Java constants used by both Kotlin modules.
abstract class GenerateRenderProtocol : DefaultTask() {
    @get:Input abstract val protocolName: Property<String>
    @get:InputFile abstract val source: RegularFileProperty
    @get:OutputDirectory abstract val outputDir: DirectoryProperty
    @TaskAction fun generate() {
        val constants = Regex("pub const ([A-Z_0-9]+): i32 = (-?[0-9]+);")
            .findAll(source.get().asFile.readText()).map { match ->
                "    public static final int ${match.groupValues[1]} = ${match.groupValues[2]};"
            }.joinToString("\n")
        val output = outputDir.get().file("com/jtn0123/audiowhisper/rust/${protocolName.get()}.java").asFile
        output.parentFile.mkdirs()
        output.writeText("// Generated from the canonical Rust protocol.\npackage com.jtn0123.audiowhisper.rust;\n" +
            "public final class ${protocolName.get()} {\n    private ${protocolName.get()}() {}\n$constants\n}\n")
    }
}
val generateRenderProtocol = tasks.register<GenerateRenderProtocol>("generateRenderProtocol") {
    protocolName.set("RenderProtocol")
    source.set(layout.projectDirectory.file("crates/render/src/protocol.rs"))
    outputDir.set(layout.buildDirectory.dir("generated/renderProtocol"))
}

val generateInputProtocol = tasks.register<GenerateRenderProtocol>("generateInputProtocol") {
    protocolName.set("InputProtocol")
    source.set(layout.projectDirectory.file("crates/input/src/protocol.rs"))
    outputDir.set(layout.buildDirectory.dir("generated/inputProtocol"))
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_11)
    }
}

android {
    namespace = "com.jtn0123.audiowhisper.rust"
    compileSdk = projectCompileSdk.toInt()
    // The NDK's clang links the library for Android (see cargoBuildAndroid).
    ndkVersion = tools.versions.ndk.get()

    defaultConfig {
        minSdk = projectMinSdk.toInt()
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
        create("beta") {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    testFixtures { enable = true }
    sourceSets.getByName("testFixtures").kotlin.directories.apply {
        clear()
        add("src/testFixtures/test/kotlin")
    }

    testOptions {
        unitTests.all {
            it.useJUnitPlatform()
        }
    }
}


dependencies {
    implementation(libs.kotlinx.coroutines)
    testImplementation(libs.kotlin.test.junit5)
}

/** Runs `cargo [args]` in the Rust workspace. */
abstract class Cargo @Inject constructor(private val exec: ExecOperations) : DefaultTask() {
    /** The cargo executable: rustup's, which reads rust-toolchain.toml. */
    @get:Input abstract val cargo: Property<String>

    @get:Input abstract val args: ListProperty<String>

    @get:Input abstract val environment: MapProperty<String, String>

    /** What cargo reads: the manifests, the pinned toolchain, its config, and the crates. */
    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val sources: ConfigurableFileCollection

    @get:Internal abstract val workspace: DirectoryProperty

    @TaskAction
    fun run() {
        exec.exec {
            workingDir = workspace.get().asFile
            commandLine(listOf(cargo.get()) + this@Cargo.args.get())
            environment(this@Cargo.environment.get())
        }
        afterCargo()
    }

    /** What to do once cargo has succeeded. */
    protected open fun afterCargo() = Unit
}

/** Builds libawnative with cargo and copies it to [outputDir]/[subdirectory]. */
abstract class CargoBuild @Inject constructor(exec: ExecOperations) : Cargo(exec) {
    /** Where cargo leaves the library, relative to the workspace. */
    @get:Input abstract val builtFile: Property<String>

    /** The ABI directory, for Android; empty for this computer. */
    @get:Input abstract val subdirectory: Property<String>

    @get:OutputDirectory abstract val outputDir: DirectoryProperty

    override fun afterCargo() {
        val built = workspace.file(builtFile).get().asFile
        val target = outputDir.get().asFile.resolve(subdirectory.get()).resolve(built.name)
        target.parentFile.mkdirs()
        built.copyTo(target, overwrite = true)
    }
}

val rustWorkspace = layout.projectDirectory
val cargoSources = files(
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    ".cargo/config.toml",
    fileTree("crates"),
)
// Gradle may not see the shell's PATH (Android Studio doesn't), so look where rustup installs.
val cargoExecutable = providers.environmentVariable("CARGO_HOME")
    .orElse(providers.systemProperty("user.home").map { "$it/.cargo" })
    .map { home -> File(home, "bin/cargo").takeIf { it.canExecute() }?.path ?: "cargo" }

/** Where cargo puts release builds; build.target-dir in .cargo/config.toml. */
fun releaseDir(target: String?) = listOfNotNull("build/cargo", target, "release").joinToString("/")

/** This computer's file name for a library. */
val hostLibrary = System.mapLibraryName("awnative")

fun Cargo.configureCargo(vararg cargoArgs: String) {
    group = "rust"
    cargo.set(cargoExecutable)
    args.addAll(*cargoArgs)
    sources.from(cargoSources)
    workspace.set(rustWorkspace)
}

val androidTarget = "aarch64-linux-android"

val cargoBuildAndroid = tasks.register<CargoBuild>("cargoBuildAndroid") {
    description = "Builds libawnative.so for arm64 Android."
    configureCargo("build", "--release", "--locked", "--package", "awnative", "--target", androidTarget)
    if (providers.gradleProperty("suggestFixture").orNull == "true") args.addAll("--features", "latin-fixture")
    if (providers.gradleProperty("ttsFixture").orNull == "true") args.addAll("--features", "tts-fixture")
    if (providers.gradleProperty("dictationFixture").orNull == "true") args.addAll("--features", "dictation-fixture")
    val ndkDirectory = androidComponents.sdkComponents.ndkDirectory
    environment.put(
        "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER",
        ndkDirectory.map { ndk -> ndkClang(ndk.asFile, projectNativeMinSdk) },
    )
    builtFile.set("${releaseDir(androidTarget)}/libawnative.so")
    subdirectory.set("arm64-v8a")
    outputDir.set(layout.buildDirectory.dir("rust/jniLibs"))
}

val cargoBuildHost = tasks.register<CargoBuild>("cargoBuildHost") {
    description = "Builds libawnative for this computer, for the JVM unit tests."
    configureCargo("build", "--release", "--locked", "--package", "awnative")
    builtFile.set("${releaseDir(null)}/$hostLibrary")
    subdirectory.set("")
    outputDir.set(layout.buildDirectory.dir("rust/host"))
}

/** The NDK's clang for linking arm64 code against Android [api]. */
fun ndkClang(ndk: File, api: String): String {
    val os = System.getProperty("os.name").lowercase()
    val host = when {
        os.contains("mac") -> "darwin-x86_64" // also on Apple silicon: the NDK ships one build
        os.contains("windows") -> "windows-x86_64"
        else -> "linux-x86_64"
    }
    val suffix = if (os.contains("windows")) ".cmd" else ""
    return ndk.resolve("toolchains/llvm/prebuilt/$host/bin/aarch64-linux-android$api-clang$suffix").path
}

val cargoTest = tasks.register<Cargo>("cargoTest") {
    description = "Runs the everyday Rust unit tests without the C++ oracle."
    configureCargo("test", "--locked")
}

val cargoClippy = tasks.register<Cargo>("cargoClippy") {
    description = "Lints the Rust code, failing on any warning."
    configureCargo("clippy", "--all-targets", "--locked", "--", "-D", "warnings")
}

val cargoOracleTest = tasks.register<Cargo>("cargoOracleTest") {
    description = "Runs the full Rust workspace tests, including LatinIME's C++ comparisons."
    configureCargo("test", "--workspace", "--features", "latin/oracle", "--locked")
}

val cargoOracleClippy = tasks.register<Cargo>("cargoOracleClippy") {
    description = "Lints the full workspace and the oracle tests, failing on any warning."
    configureCargo("clippy", "--workspace", "--all-targets", "--features", "latin/oracle", "--locked", "--", "-D", "warnings")
}

val cargoFmtCheck = tasks.register<Cargo>("cargoFmtCheck") {
    description = "Checks the Rust code is formatted as rustfmt would."
    configureCargo("fmt", "--all", "--check")
}

tasks.named("check") {
    dependsOn(cargoTest, cargoClippy, cargoFmtCheck, ":app:checkNativeDeclarations")
}

tasks.register("oracleCheck") {
    group = "verification"
    description = "Checks the full Rust workspace with the opt-in LatinIME C++ oracle."
    dependsOn(cargoOracleTest, cargoOracleClippy, cargoFmtCheck)
}

androidComponents {
    onVariants { variant ->
        variant.sources.java?.addGeneratedSourceDirectory(generateRenderProtocol, GenerateRenderProtocol::outputDir)
        variant.sources.java?.addGeneratedSourceDirectory(generateInputProtocol, GenerateRenderProtocol::outputDir)
        variant.sources.jniLibs?.addGeneratedSourceDirectory(cargoBuildAndroid, CargoBuild::outputDir)
    }
}

tasks.withType<Test>().configureEach {
    // The library as an input, not just a dependency, so changing the Rust runs the tests again.
    inputs.files(cargoBuildHost).withPropertyName("rustHostLibrary").withPathSensitivity(PathSensitivity.RELATIVE)
    systemProperty("java.library.path", layout.buildDirectory.dir("rust/host").get().asFile.path)
    // Newer JDKs warn when code loads a native library unless allowed to.
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}
