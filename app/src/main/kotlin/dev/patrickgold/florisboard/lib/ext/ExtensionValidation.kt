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

package dev.patrickgold.florisboard.lib.ext

import com.jtn0123.audiowhisper.rust.ExtensionPolicy
import dev.patrickgold.florisboard.lib.ValidationResult
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponent
import dev.patrickgold.florisboard.lib.ValidationRule
import org.florisboard.lib.snygg.SnyggStylesheet
import org.florisboard.lib.snygg.value.SnyggVarValue

object ExtensionValidation {
    private fun native(rule: String, value: String): ValidationResult {
        val plan = ExtensionPolicy.validate(rule, value)
        if (plan[1].isEmpty()) return ValidationResult.resultValid()
        val resource = resources.getValue(plan[1])
        return when {
            plan[0] == "valid" -> ValidationResult.resultValid(hint = resource)
            plan[2].isEmpty() -> ValidationResult.resultInvalid(error = resource)
            else -> ValidationResult.resultInvalid(resource, plan[2] to plan[3])
        }
    }

    private val resources = mapOf(
        "ext__validation__enter_package_name" to R.string.ext__validation__enter_package_name,
        "ext__validation__error_package_name" to R.string.ext__validation__error_package_name,
        "ext__validation__enter_version" to R.string.ext__validation__enter_version,
        "ext__validation__enter_title" to R.string.ext__validation__enter_title,
        "ext__validation__enter_maintainer" to R.string.ext__validation__enter_maintainer,
        "ext__validation__enter_license" to R.string.ext__validation__enter_license,
        "ext__validation__enter_component_id" to R.string.ext__validation__enter_component_id,
        "ext__validation__error_component_id" to R.string.ext__validation__error_component_id,
        "ext__validation__enter_component_label" to R.string.ext__validation__enter_component_label,
        "ext__validation__hint_component_label_to_long" to R.string.ext__validation__hint_component_label_to_long,
        "ext__validation__error_author" to R.string.ext__validation__error_author,
        "ext__validation__error_stylesheet_path_blank" to R.string.ext__validation__error_stylesheet_path_blank,
        "ext__validation__error_stylesheet_path" to R.string.ext__validation__error_stylesheet_path,
    )

    val MetaId = ValidationRule<String> {
        forKlass = ExtensionMeta::class
        forProperty = "id"
        validator { str -> native("id", str) }
    }

    val MetaVersion = ValidationRule<String> {
        forKlass = ExtensionMeta::class
        forProperty = "version"
        validator { str -> native("version", str) }
    }

    val MetaTitle = ValidationRule<String> {
        forKlass = ExtensionMeta::class
        forProperty = "title"
        validator { str -> native("title", str) }
    }

    val MetaMaintainers = ValidationRule<String> {
        forKlass = ExtensionMeta::class
        forProperty = "maintainers"
        validator { str -> native("maintainers", str) }
    }

    val MetaLicense = ValidationRule<String> {
        forKlass = ExtensionMeta::class
        forProperty = "license"
        validator { str -> native("license", str) }
    }

    val ComponentId = ValidationRule<String> {
        forKlass = ExtensionComponent::class
        forProperty = "id"
        validator { str -> native("componentId", str) }
    }

    val ComponentLabel = ValidationRule<String> {
        forKlass = ExtensionComponent::class
        forProperty = "label"
        validator { str -> native("componentLabel", str) }
    }

    val ComponentAuthors = ValidationRule<String> {
        forKlass = ExtensionComponent::class
        forProperty = "authors"
        validator { str -> native("componentAuthors", str) }
    }

    val ThemeComponentStylesheetPath = ValidationRule<String> {
        forKlass = ThemeExtensionComponent::class
        forProperty = "stylesheetPath"
        validator { str -> native("stylesheet", str) }
    }

    val ThemeComponentVariableName = ValidationRule<String> {
        forKlass = SnyggStylesheet::class
        forProperty = "propertyName"
        validator { input ->
            val str = input.trim()
            when {
                str.isBlank() -> resultInvalid(error = R.string.ext__validation__enter_property)
                !SnyggVarValue.VariableNameRegex.matches(str) -> {
                    resultInvalid(error = R.string.ext__validation__error_property, "variable_name_regex" to SnyggVarValue.VariableNameRegex)
                }
                else -> resultValid()
            }
        }
    }

    val SnyggStaticColorValue = ValidationRule<String> {
        forKlass = org.florisboard.lib.snygg.value.SnyggStaticColorValue::class
        forProperty = "color"
        validator { input ->
            val str = input.trim()
            when {
                str.isBlank() -> resultInvalid(error = R.string.ext__validation__enter_color)
                org.florisboard.lib.snygg.value.SnyggStaticColorValue.deserialize(str).isFailure -> {
                    resultInvalid(error = R.string.ext__validation__error_color)
                }
                else -> resultValid()
            }
        }
    }

    val SnyggDpShapeValue = ValidationRule<String> {
        forKlass = org.florisboard.lib.snygg.value.SnyggDpShapeValue::class
        forProperty = "corner"
        validator { str ->
            val floatValue = str.toFloatOrNull()
            when {
                str.isBlank() -> resultInvalid(error = R.string.ext__validation__enter_dp_size)
                floatValue == null -> resultInvalid(error = R.string.ext__validation__enter_valid_number)
                floatValue < 0f -> resultInvalid(error = R.string.ext__validation__enter_positive_number)
                else -> resultValid()
            }
        }
    }

    val SnyggPercentShapeValue = ValidationRule<String> {
        forKlass = org.florisboard.lib.snygg.value.SnyggPercentShapeValue::class
        forProperty = "corner"
        validator { str ->
            val intValue = str.toIntOrNull()
            when {
                str.isBlank() -> resultInvalid(error = R.string.ext__validation__enter_percent_size)
                intValue == null -> resultInvalid(error = R.string.ext__validation__enter_valid_number)
                intValue < 0 || intValue > 100 -> resultInvalid(error = R.string.ext__validation__enter_number_between_0_100)
                intValue > 50 -> resultValid(hint = R.string.ext__validation__hint_value_above_50_percent)
                else -> resultValid()
            }
        }
    }
}

fun ExtensionMeta.validate(): Boolean = ExtensionPolicy.metadataValid(
    id, version, title, maintainers.joinToString("\n"), license,
)
