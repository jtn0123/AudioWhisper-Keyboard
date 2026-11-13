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

rootProject.name = "FlorisBoard"

pluginManagement {
    repositories {
        gradlePluginPortal()
        google()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
        // AudioWhisper: jetpref is a snapshot that Sonatype no longer serves, so a copy is kept in
        // the repo (third_party/m2/README.md) and is the only place jetpref is looked for. Nothing
        // else resolves from it, and nothing resolves from ~/.m2, so every machine builds alike.
        exclusiveContent {
            forRepository { maven(rootDir.resolve("third_party/m2")) { name = "jetprefCopy" } }
            filter { includeGroup("dev.patrickgold.jetpref") }
        }
    }

    versionCatalogs {
        create("tools") {
            from(files("gradle/tools.versions.toml"))
        }
    }
}
enableFeaturePreview("TYPESAFE_PROJECT_ACCESSORS")

include(":app")
//include(":benchmark")
include(":lib:android")
include(":lib:color")
include(":lib:compose")
include(":lib:kotlin")
include(":lib:rust")
include(":lib:snygg")
