//! The extension editor's draft: ThemeExtensionEditor's meta, dependencies and components, read
//! from the `extension.json` Kotlin reports and written back as Compose's `handleSave` wrote it,
//! `ExtensionJsonConfig.encodeToString<Extension>(editor.build())`: `$` first, then the fields
//! in their declared order, a null or default field left out, as `encodeDefaults = false` leaves
//! it, and the Material You flags always written, as `@EncodeDefault` writes them.
use extensions::maintainer;
use serde_json::{Map, Value, json};
use wire::extensions::theme;

/// The serial type of the only extension the editor edits.
pub const THEME: &str = "ime.extension.theme";

/// ExtensionMeta's fields; the optional ones `None` when the manifest has none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meta {
    pub id: String,
    pub version: String,
    pub title: String,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub homepage: Option<String>,
    pub issue_tracker: Option<String>,
    /// Each as `ExtensionMaintainer.toString` writes it.
    pub maintainers: Vec<String>,
    pub license: String,
}

/// A theme component as ThemeExtensionComponentEditor holds it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Comp {
    pub id: String,
    pub label: String,
    pub authors: Vec<String>,
    pub night: bool,
    /// The Material You palette style, contrast level and spec version, by enum name.
    pub flags: [String; 3],
    /// The stylesheet path as written: empty for `stylesheets/<id>.json`.
    pub path: String,
}

impl Comp {
    /// Where the component keeps its stylesheet, `stylesheetPath()`.
    pub fn stylesheet(&self) -> String {
        let path = extensions::stylesheet_path(&snygg::text(&self.id), &snygg::text(&self.path));
        String::from_utf16_lossy(&path)
    }
    /// `build()`'s plan: the id and label trimmed, the blank authors dropped and the path kept
    /// only when not blank; or why it cannot be built, which `check` threw.
    fn built(&self) -> Result<Value, String> {
        let u = |s: &str| snygg::text(s);
        let authors: Vec<_> = self.authors.iter().map(|a| u(a)).collect();
        let plan = extensions_theme_plan(&u(&self.id), &u(&self.label), &u(&self.path), &authors);
        let s = |i: usize| String::from_utf16_lossy(&plan[i]);
        if !plan[theme::ERROR].is_empty() {
            return Err(s(theme::ERROR));
        }
        let authors: Vec<String> = (theme::AUTHORS..plan.len()).map(s).collect();
        let present = wire::flag::decode(&plan[theme::STYLESHEET_PRESENT]);
        let path = present.then(|| s(theme::STYLESHEET));
        Ok(component(
            &s(theme::ID),
            &s(theme::LABEL),
            &authors,
            self,
            path,
        ))
    }
}

/// `ExtensionPolicy.themeComponent`, through the extensions crate's own entry point.
fn extensions_theme_plan(
    id: &[u16],
    label: &[u16],
    path: &[u16],
    authors: &[Vec<u16>],
) -> Vec<Vec<u16>> {
    let mut input = vec![id.to_vec(), label.to_vec(), path.to_vec()];
    input.extend(authors.iter().cloned());
    extensions::call(wire::extensions::THEME_COMPONENT, &input).unwrap_or_default()
}

/// A component's JSON, its fields in ThemeExtensionComponentImpl's order.
fn component(id: &str, label: &str, authors: &[String], c: &Comp, path: Option<String>) -> Value {
    let mut out = Map::new();
    out.insert("id".into(), json!(id));
    out.insert("label".into(), json!(label));
    out.insert("authors".into(), json!(authors));
    if !c.night {
        out.insert("isNight".into(), json!(false));
    }
    let [palette, contrast, spec] = &c.flags;
    let flags = json!({"paletteStyle": palette, "contrastLevel": contrast, "specVersion": spec});
    out.insert("materialYouFlags".into(), flags);
    if let Some(path) = path {
        out.insert("stylesheet".into(), json!(path));
    }
    Value::Object(out)
}

/// The extension being edited.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Draft {
    pub meta: Meta,
    pub dependencies: Vec<String>,
    pub themes: Vec<Comp>,
}

fn string(v: &Value, key: &str) -> Result<String, &'static str> {
    v.get(key)
        .and_then(Value::as_str)
        .map(Into::into)
        .ok_or("settings extension editor manifest field")
}
fn optional(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(Into::into)
}
fn strings(v: Option<&Value>) -> Result<Vec<String>, &'static str> {
    let Some(all) = v else { return Ok(vec![]) };
    all.as_array()
        .ok_or("settings extension editor manifest list")?
        .iter()
        .map(|s| {
            s.as_str()
                .map(Into::into)
                .ok_or("settings extension editor manifest list")
        })
        .collect()
}

impl Draft {
    /// The `extension.json` Kotlin reports: a theme extension as kotlinx encoded it.
    pub fn parse(json: &str) -> Result<Self, &'static str> {
        let v: Value =
            serde_json::from_str(json).map_err(|_| "settings extension editor manifest")?;
        let m = v.get("meta").ok_or("settings extension editor meta")?;
        let meta = Meta {
            id: string(m, "id")?,
            version: string(m, "version")?,
            title: string(m, "title")?,
            description: optional(m, "description"),
            keywords: m
                .get("keywords")
                .filter(|k| !k.is_null())
                .map(|k| strings(Some(k)))
                .transpose()?,
            homepage: optional(m, "homepage"),
            issue_tracker: optional(m, "issueTracker"),
            maintainers: strings(m.get("maintainers"))?,
            license: string(m, "license")?,
        };
        let themes = v
            .get("themes")
            .and_then(Value::as_array)
            .ok_or("settings extension editor themes")?;
        Ok(Self {
            meta,
            dependencies: strings(v.get("dependencies").filter(|d| !d.is_null()))?,
            themes: themes.iter().map(read_comp).collect::<Result<_, _>>()?,
        })
    }
    /// A new theme extension, as ExtensionEditScreen made one: `createLocalId("themes", now)`.
    pub fn fresh(now: &str) -> Self {
        Self {
            meta: Meta {
                id: format!("local.themes.{now}"),
                version: "0.0.0".into(),
                title: "My themes".into(),
                maintainers: vec!["Local".into()],
                license: "(none specified)".into(),
                ..Meta::default()
            },
            ..Self::default()
        }
    }
    fn meta_json(&self) -> Value {
        let m = &self.meta;
        let mut out = Map::new();
        out.insert("id".into(), json!(m.id));
        out.insert("version".into(), json!(m.version));
        out.insert("title".into(), json!(m.title));
        let optional = [
            ("description", m.description.as_ref().map(|d| json!(d))),
            ("keywords", m.keywords.as_ref().map(|k| json!(k))),
            ("homepage", m.homepage.as_ref().map(|h| json!(h))),
            ("issueTracker", m.issue_tracker.as_ref().map(|i| json!(i))),
        ];
        for (key, value) in optional {
            if let Some(value) = value {
                out.insert(key.into(), value);
            }
        }
        out.insert("maintainers".into(), json!(m.maintainers));
        out.insert("license".into(), json!(m.license));
        Value::Object(out)
    }
    fn manifest(&self, themes: Vec<Value>) -> String {
        let mut out = Map::new();
        out.insert("$".into(), json!(THEME));
        out.insert("meta".into(), self.meta_json());
        if !self.dependencies.is_empty() {
            out.insert("dependencies".into(), json!(self.dependencies));
        }
        out.insert("themes".into(), Value::Array(themes));
        snygg::json_write::pretty(&Value::Object(out))
    }
    /// The manifest Save writes, `editor.build()` encoded; or why a component cannot be built.
    pub fn json(&self) -> Result<String, (usize, String)> {
        let themes = self.themes.iter().enumerate();
        let built = themes.map(|(i, c)| c.built().map_err(|e| (i, e)));
        Ok(self.manifest(built.collect::<Result<_, _>>()?))
    }
    /// The draft as the editor holds it, nothing trimmed or dropped: the copy Kotlin keeps for
    /// the theme editor, which reads a component as it was typed.
    pub fn raw_json(&self) -> String {
        let themes = self.themes.iter();
        let raw = themes.map(|c| {
            component(
                &c.id,
                &c.label,
                &c.authors,
                c,
                Some(c.path.clone()).filter(|p| !p.is_empty()),
            )
        });
        self.manifest(raw.collect())
    }
}

fn read_comp(v: &Value) -> Result<Comp, &'static str> {
    let flags = v.get("materialYouFlags");
    let flag = |key: &str| {
        flags
            .and_then(|f| optional(f, key))
            .ok_or("settings extension editor flags")
    };
    Ok(Comp {
        id: string(v, "id")?,
        label: string(v, "label")?,
        authors: strings(v.get("authors"))?,
        night: v.get("isNight").and_then(Value::as_bool).unwrap_or(true),
        flags: [
            flag("paletteStyle")?,
            flag("contrastLevel")?,
            flag("specVersion")?,
        ],
        path: optional(v, "stylesheet").unwrap_or_default(),
    })
}

/// A maintainer line as Apply keeps it, `ExtensionMaintainer.fromOrTakeRaw(line).toString()`:
/// the name, email and website the pattern finds, written again; the line itself when it does
/// not match.
pub fn maintainer_text(line: &str) -> String {
    let t = snygg::text(line);
    let Some(r) = maintainer::parse(&t) else {
        return line.into();
    };
    let part = |at: usize| -> Option<Vec<u16>> {
        let (start, end) = (
            usize::try_from(r[at]).ok()?,
            usize::try_from(r[at + 1]).ok()?,
        );
        t.get(start..end).map(<[u16]>::to_vec)
    };
    let Some(name) = part(0) else {
        return line.into();
    };
    let (email, url) = (part(2).unwrap_or_default(), part(4).unwrap_or_default());
    String::from_utf16_lossy(&maintainer::format(&name, &email, &url))
}
