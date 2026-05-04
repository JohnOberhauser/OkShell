use okshell_config::schema::config::Config;
use schemars::_private::serde_json;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(Config);
    let value = serde_json::to_value(&schema).unwrap();
    let defs = value["$defs"].as_object().unwrap();

    let mut out = String::new();

    out.push_str("# Configuration Schema\n\n");

    // Root Config first
    out.push_str("## Config\n\n");
    render_struct(&mut out, &value, defs);

    // All defs
    for (name, def) in defs {
        if matches!(classify(def), Kind::Scalar) {
            continue;
        }

        out.push_str(&format!("## {}\n\n", name));
        match classify(def) {
            Kind::Struct => render_struct(&mut out, def, defs),
            Kind::Enum => render_enum(&mut out, def),
            Kind::TaggedUnion => render_tagged_union(&mut out, def, defs),
            Kind::Scalar => continue,
        }
    }

    print!("{}", out);
}

enum Kind {
    Struct,
    Enum,
    TaggedUnion,
    Scalar,
}

fn classify(def: &serde_json::Value) -> Kind {
    if def["properties"].is_object() {
        return Kind::Struct;
    }
    if def["type"] == "string" && def["enum"].is_array() {
        return Kind::Enum;
    }
    if def["type"] == "number" || def["type"] == "integer" {
        return Kind::Scalar;
    }
    if def["oneOf"].is_array() {
        let has_objects = def["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["type"] == "object");
        if has_objects {
            return Kind::TaggedUnion;
        }
        return Kind::Enum;
    }
    Kind::Scalar
}

fn get_type(node: &serde_json::Value, defs: &serde_json::Map<String, serde_json::Value>) -> String {
    if let Some(r) = node["$ref"].as_str() {
        let name = r.split('/').last().unwrap_or("?");
        if let Some(def) = defs.get(name) {
            if matches!(classify(def), Kind::Scalar) {
                // Unwrap to the primitive
                return def["type"].as_str().unwrap_or(name).to_string();
            }
        }
        return name.to_string();
    }
    if node["type"] == "array" {
        return format!("Vec<{}>", get_type(&node["items"], defs));
    }
    if let Some(t) = node["type"].as_str() {
        return t.to_string();
    }
    if node["oneOf"].is_array() {
        return "oneOf".to_string();
    }
    "—".to_string()
}

fn format_default(val: &serde_json::Value) -> String {
    if val.is_null() || val.is_object() && val.as_object().unwrap().is_empty() {
        return "—".to_string();
    }
    match val {
        serde_json::Value::String(s) => format!("`{}`", s),
        serde_json::Value::Bool(b) => format!("`{}`", b),
        serde_json::Value::Number(n) => format!("`{}`", n),
        serde_json::Value::Array(a) if a.is_empty() => "`[]`".to_string(),
        _ => "—".to_string(),
    }
}

fn render_struct(
    out: &mut String,
    def: &serde_json::Value,
    defs: &serde_json::Map<String, serde_json::Value>,
) {
    let Some(props) = def["properties"].as_object() else {
        out.push_str("_No properties._\n\n");
        return;
    };

    out.push_str("| Field | Type | Default |\n");
    out.push_str("|-------|------|---------|\n");

    for (field, prop) in props {
        let ty = get_type(prop, defs);
        let default = format_default(&prop["default"]);
        out.push_str(&format!("| `{}` | `{}` | {} |\n", field, ty, default));
    }
    out.push('\n');
}

fn render_enum(out: &mut String, def: &serde_json::Value) {
    out.push_str("| Variant |\n");
    out.push_str("|---------|\n");

    let variants = if def["enum"].is_array() {
        def["enum"].as_array().unwrap().to_vec()
    } else if def["oneOf"].is_array() {
        def["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|v| v["enum"].as_array().cloned().unwrap_or_default())
            .collect()
    } else {
        vec![]
    };

    for v in variants {
        if let Some(s) = v.as_str() {
            out.push_str(&format!("| `{}` |\n", s));
        }
    }
    out.push('\n');
}

fn render_tagged_union(
    out: &mut String,
    def: &serde_json::Value,
    defs: &serde_json::Map<String, serde_json::Value>,
) {
    out.push_str("| Variant | Data |\n");
    out.push_str("|---------|------|\n");

    for variant in def["oneOf"].as_array().unwrap() {
        if variant["type"] == "string" {
            // simple string variants
            if let Some(enums) = variant["enum"].as_array() {
                for v in enums {
                    if let Some(s) = v.as_str() {
                        out.push_str(&format!("| `{}` | — |\n", s));
                    }
                }
            }
        } else if variant["type"] == "object" {
            if let Some(tag) = variant["properties"]["type"]["const"].as_str() {
                // internally tagged: #[serde(tag = "type")]
                let fields: Vec<String> = variant["properties"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(k, _)| *k != "type")
                    .map(|(k, v)| format!("{}: {}", k, get_type(v, defs)))
                    .collect();
                out.push_str(&format!("| `{}` | `{}` |\n", tag, fields.join(", ")));
            } else if let Some(required) = variant["required"].as_array() {
                // externally tagged: { "VariantName": { ... } }
                let name = required[0].as_str().unwrap_or("?");
                let inner = get_type(&variant["properties"][name], defs);
                out.push_str(&format!("| `{}` | `{}` |\n", name, inner));
            }
        }
    }
    out.push('\n');
}
