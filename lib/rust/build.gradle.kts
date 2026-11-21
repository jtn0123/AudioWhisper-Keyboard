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
import java.io.ByteArrayOutputStream
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
        beforeCargo()
        val output = ByteArrayOutputStream()
        exec.exec {
            workingDir = workspace.get().asFile
            commandLine(listOf(cargo.get()) + this@Cargo.args.get() + outputArgs())
            environment(this@Cargo.environment.get())
            if (readsOutput()) standardOutput = output
        }
        afterCargo(output.toString(Charsets.UTF_8))
    }

    /** What to do before cargo runs. */
    protected open fun beforeCargo() = Unit

    /**
     * Arguments after [args] naming this task's outputs, which [args] cannot hold: an input made
     * from an output would make the task depend on itself.
     */
    protected open fun outputArgs() = emptyList<String>()

    /** Whether [afterCargo] reads cargo's standard output, which then stays off the console. */
    protected open fun readsOutput() = false

    /** What to do once cargo has succeeded, given its standard output if [readsOutput]. */
    protected open fun afterCargo(output: String) = Unit
}

/**
 * Builds libawnative with cargo and copies it to [outputDir]/[subdirectory], and the generated
 * entry points compiled into it to [boundaryStubs]. Its arguments must ask for cargo's JSON
 * messages, which say where awnative's build.rs wrote them.
 */
abstract class CargoBuild @Inject constructor(exec: ExecOperations) : Cargo(exec) {
    /** Where cargo leaves the library, relative to the workspace. */
    @get:Input abstract val builtFile: Property<String>

    /** The ABI directory, for Android; empty for this computer. */
    @get:Input abstract val subdirectory: Property<String>

    @get:OutputDirectory abstract val outputDir: DirectoryProperty

    /**
     * A copy of the generated entry points (crates/boundary) this build compiled into the library,
     * which :app:checkNativeDeclarations reads: the Rust side as built, even if the declarations
     * have changed since.
     */
    @get:OutputFile abstract val boundaryStubs: RegularFileProperty

    override fun readsOutput() = true

    override fun afterCargo(output: String) {
        val built = workspace.file(builtFile).get().asFile
        val target = outputDir.get().asFile.resolve(subdirectory.get()).resolve(built.name)
        target.parentFile.mkdirs()
        built.copyTo(target, overwrite = true)
        // Cargo reports each build script's OUT_DIR, whether it ran again or not.
        val outDir = Regex("\"out_dir\":\"([^\"]+)\"")
        val awnative = output.lineSequence()
            .filter { "\"reason\":\"build-script-executed\"" in it && "/crates/awnative#" in it }
            .mapNotNull { outDir.find(it)?.groupValues?.get(1) }
            .singleOrNull() ?: throw GradleException("cargo did not say where awnative's build.rs wrote")
        File(awnative, "boundary.rs").copyTo(boundaryStubs.get().asFile, overwrite = true)
    }
}

/**
 * Writes the Kotlin side of the generated JNI boundary into [outputDir], and the part lib/kotlin
 * compiles (crates/wire's groups for its own calls) into [libKotlinDir], by running
 * crates/boundary on this computer; awnative's build.rs writes the Rust side.
 */
abstract class GenerateNativeBoundary @Inject constructor(
    exec: ExecOperations,
    private val fileSystem: FileSystemOperations,
) : Cargo(exec) {
    @get:OutputDirectory abstract val outputDir: DirectoryProperty

    @get:OutputDirectory abstract val libKotlinDir: DirectoryProperty

    // A class dropped from the declarations must not leave its file behind.
    override fun beforeCargo() {
        fileSystem.delete { delete(outputDir, libKotlinDir) }
    }

    override fun outputArgs() = listOf(outputDir.get().asFile.path, libKotlinDir.get().asFile.path)
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
    configureCargo(
        "build", "--release", "--locked", "--message-format=json-render-diagnostics",
        "--package", "awnative", "--target", androidTarget,
    )
    if (providers.gradleProperty("suggestFixture").orNull == "true") args.addAll("--features", "latin-fixture")
    if (providers.gradleProperty("ttsFixture").orNull == "true") args.addAll("--features", "tts-fixture")
    if (providers.gradleProperty("dictationFixture").orNull == "true") args.addAll("--features", "dictation-fixture")
    // The experimental GPU renderer, for measuring it again; no build that ships includes it.
    if (providers.gradleProperty("hybridRenderer").orNull == "true") args.addAll("--features", "hybrid")
    val ndkDirectory = androidComponents.sdkComponents.ndkDirectory
    environment.put(
        "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER",
        ndkDirectory.map { ndk -> ndkClang(ndk.asFile, projectNativeMinSdk) },
    )
    builtFile.set("${releaseDir(androidTarget)}/libawnative.so")
    subdirectory.set("arm64-v8a")
    outputDir.set(layout.buildDirectory.dir("rust/jniLibs"))
    boundaryStubs.set(layout.buildDirectory.file("rust/boundary/arm64-v8a.rs"))
}

val cargoBuildHost = tasks.register<CargoBuild>("cargoBuildHost") {
    description = "Builds libawnative for this computer, for the JVM unit tests."
    configureCargo(
        "build", "--release", "--locked", "--message-format=json-render-diagnostics", "--package", "awnative",
    )
    builtFile.set("${releaseDir(null)}/$hostLibrary")
    subdirectory.set("")
    outputDir.set(layout.buildDirectory.dir("rust/host"))
    boundaryStubs.set(layout.buildDirectory.file("rust/boundary/host.rs"))
}

val generateNativeBoundary = tasks.register<GenerateNativeBoundary>("generateNativeBoundary") {
    description = "Generates the Kotlin side of the JNI boundary from crates/boundary's declarations."
    configureCargo("run", "--quiet", "--locked", "--package", "boundary", "--")
    // Only the generator, its declarations, and the manifests and toolchain cargo reads, decide
    // what it writes.
    sources.setFrom(
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/config.toml",
        fileTree("crates") { include("*/Cargo.toml") }, fileTree("crates/boundary"), fileTree("crates/wire"),
    )
    outputDir.set(layout.buildDirectory.dir("generated/nativeBoundary/kotlin"))
    // lib/kotlin adds this directory to its sources (see its build script).
    libKotlinDir.set(layout.buildDirectory.dir("generated/nativeBoundary/libKotlin"))
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

// The code only Android compiles (surfaces, the presenter, JNI windows) and the off-by-default GPU
// renderer (the `hybrid` feature), which nothing else builds; host clippy sees neither.
val cargoAndroidClippy = tasks.register<Cargo>("cargoAndroidClippy") {
    description = "Lints libawnative as Android builds it, failing on any warning."
    configureCargo("clippy", "--locked", "--package", "awnative", "--target", androidTarget, "--", "-D", "warnings")
}

val cargoAndroidHybridClippy = tasks.register<Cargo>("cargoAndroidHybridClippy") {
    description = "Lints libawnative for Android with the experimental GPU renderer, failing on any warning."
    configureCargo(
        "clippy", "--locked", "--package", "awnative", "--target", androidTarget, "--features", "hybrid",
        "--", "-D", "warnings",
    )
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
    dependsOn(
        cargoTest, cargoClippy, cargoAndroidClippy, cargoAndroidHybridClippy, cargoFmtCheck,
        ":app:checkNativeDeclarations",
    )
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
        variant.sources.kotlin?.addGeneratedSourceDirectory(generateNativeBoundary, GenerateNativeBoundary::outputDir)
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
