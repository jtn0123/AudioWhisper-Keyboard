//! The records, enums, bit fields and interfaces an api declares, nested in it or each in a file
//! of its own.

use super::{Type, constants, records};
use crate::Module;
use crate::format::{kdoc, kotlin_declaration};
use crate::kotlin::{KotlinFile, kotlin_header};

/// A record or enum at `indent`.
pub(super) fn type_text(ty: &Type, indent: &str) -> String {
    match ty {
        Type::Record {
            name,
            doc,
            data,
            fields,
            constants: kept,
        } => {
            let mut text = if doc.is_empty() {
                String::new()
            } else {
                kdoc(doc, indent)
            };
            let fields: Vec<String> = fields
                .iter()
                .map(|(name, ty)| format!("val {name}: {ty}"))
                .collect();
            let head = format!("{}class {name}", if *data { "data " } else { "" });
            if kept.is_empty() {
                text.push_str(&kotlin_declaration(indent, &head, &fields, ""));
            } else {
                text.push_str(&kotlin_declaration(indent, &head, &fields, " {"));
                let inner = format!("{indent}        ");
                text.push_str(&format!("{indent}    companion object {{\n"));
                text.push_str(&constants(kept, &inner));
                text.push_str(&format!("{indent}    }}\n{indent}}}\n"));
            }
            text
        }
        Type::Enum { name, doc, entries } => {
            format!(
                "{}{indent}enum class {name} {{ {} }}\n",
                kdoc(doc, indent),
                entries.join(", ")
            )
        }
        Type::Bits { .. } => records::bits_text(ty, indent),
        Type::Interface { name, doc, members } => {
            let mut text = kdoc(doc, indent);
            let single = matches!(members, [(declaration, _)] if declaration.starts_with("fun "));
            let fun = if single { "fun " } else { "" };
            text.push_str(&format!("{indent}{fun}interface {name} {{\n"));
            let inner = format!("{indent}    ");
            for (at, (declaration, doc)) in members.iter().enumerate() {
                if at > 0 && !doc.is_empty() {
                    text.push('\n');
                }
                if !doc.is_empty() {
                    text.push_str(&kdoc(doc, &inner));
                }
                text.push_str(&format!("{inner}{declaration}\n"));
            }
            text.push_str(&format!("{indent}}}\n"));
            text
        }
    }
}

/// The Kotlin files for top-level `types`, one each, generated from `module`'s declarations.
pub fn type_files(types: &[(&str, Module, Type)]) -> Vec<(Module, KotlinFile)> {
    types
        .iter()
        .map(|(module, kotlin, ty)| {
            let name = match ty {
                Type::Record { name, .. }
                | Type::Enum { name, .. }
                | Type::Bits { name, .. }
                | Type::Interface { name, .. } => *name,
            };
            let source = format!("src/declarations/{module}.rs");
            let mut text = kotlin_header(&source, *kotlin, &[]);
            text.push_str(&type_text(ty, ""));
            let path = format!("{}/{name}.kt", kotlin.package().replace('.', "/"));
            (*kotlin, KotlinFile { path, text })
        })
        .collect()
}
