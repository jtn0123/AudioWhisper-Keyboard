/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
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

package org.florisboard.lib.snygg

import org.florisboard.lib.snygg.value.SnyggDefinedVarValue
import org.florisboard.lib.snygg.value.SnyggInheritValue
import org.florisboard.lib.snygg.value.SnyggVarValue

/**
 * Main object for defining all known Snygg property names.
 *
 * snygg = Swedish for stylish
 */
object Snygg {
    const val Background = "background"
    const val Foreground = "foreground"

    const val BackgroundImage = "background-image"
    const val ContentScale = "content-scale"

    const val BorderColor = "border-color"
    const val BorderStyle = "border-style" // unsupported as of now
    const val BorderWidth = "border-width"

    const val FontFamily = "font-family"
    const val FontSize = "font-size"
    const val FontStyle = "font-style"
    const val FontWeight = "font-weight"
    const val LetterSpacing = "letter-spacing"
    const val LineHeight = "line-height"

    const val Margin = "margin"
    const val Padding = "padding"

    const val ShadowColor = "shadow-color"
    const val ShadowElevation = "shadow-elevation"

    const val Shape = "shape"
    const val Clip = "clip"

    const val Src = "src"

    const val TextAlign = "text-align"
    const val TextDecorationLine = "text-decoration-line"
    const val TextMaxLines = "text-max-lines"
    const val TextOverflow = "text-overflow"
}

object SnyggSpec : SnyggSpecDecl({
    meta.title = "Snygg Stylesheet Specification"
    meta.description = "This document describes the Snygg stylesheet specification."

    annotation(SnyggAnnotationRule.Defines).singleSet {
        pattern(SnyggVarValue.VariableNameRegex) {
            any()
        }
    }

    annotation(SnyggAnnotationRule.Font).multipleSets {
        for ((name, flags, encoders) in SnyggCompilerAdapter.properties(font = true)) {
            name {
                if (flags.second) required()
                encoders.forEach { add(it) }
            }
        }
    }
    elements {
        implicit { add(SnyggInheritValue); add(SnyggDefinedVarValue) }
        for ((name, flags, encoders) in SnyggCompilerAdapter.properties(font = false)) {
            name {
                if (flags.first) inheritsImplicitly()
                encoders.forEach { add(it) }
            }
        }
    }
})
