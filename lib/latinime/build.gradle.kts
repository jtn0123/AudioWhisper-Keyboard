/*
 * AudioWhisper Keyboard: the minimal Java glue for the AOSP LatinIME dictionary engine, whose
 * natives are now the Rust port's (lib/rust/crates/latin, entry points in awnative's latinime.rs,
 * found by their JNI names). Apache-2.0, see the file headers. src/main/jni keeps LatinIME's C++
 * as the reference the Rust tests compare against (lib/rust/crates/latin-oracle); the app no
 * longer builds or ships it.
 */

plugins {
    alias(libs.plugins.agp.library)
}

val projectMinSdk = providers.gradleProperty("projectMinSdk").get()
val projectCompileSdk = providers.gradleProperty("projectCompileSdk").get()

android {
    namespace = "com.android.inputmethod.latinime"
    compileSdk = projectCompileSdk.toInt()
    // Java only: no Kotlin compile task, no kotlin-stdlib dependency.
    enableKotlin = false

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
}
