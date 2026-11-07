/*
 * AudioWhisper Keyboard: the AOSP LatinIME native dictionary engine
 * (libjni_latinime.so) plus the minimal Java glue whose class and method names
 * the library registers its JNI methods on. Apache-2.0, see the file headers.
 */

plugins {
    alias(libs.plugins.agp.library)
}

val projectMinSdk: String by project
val projectCompileSdk: String by project

android {
    namespace = "com.android.inputmethod.latinime"
    compileSdk = projectCompileSdk.toInt()
    ndkVersion = tools.versions.ndk.get()

    defaultConfig {
        minSdk = projectMinSdk.toInt()
        consumerProguardFiles("consumer-rules.pro")

        externalNativeBuild {
            ndkBuild {
                arguments("-j${Runtime.getRuntime().availableProcessors()}")
            }
        }

        // The phone is arm64 (S24 Ultra); other ABIs would only add build time and size.
        ndk {
            abiFilters += listOf("arm64-v8a")
        }
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

    externalNativeBuild {
        ndkBuild {
            path("src/main/jni/Android.mk")
        }
    }
}
