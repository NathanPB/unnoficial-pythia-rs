use super::{Context, ContextEvaluationError};
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::LazyLock;

static RE_TEMPLATE_STRING: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(\$\{[^}]+}|[^$]+)").unwrap());

#[derive(Clone, Debug)]
pub struct TemplateString(Vec<TemplateStringFragment>);

#[derive(Clone, Debug)]
enum TemplateStringFragment {
    Literal(String),
    Template(String),
}

impl TemplateString {
    pub fn interpolate(&self, ctx: &Context) -> Result<String, ContextEvaluationError> {
        let mut s = String::new();
        for fragment in &self.0 {
            match fragment {
                TemplateStringFragment::Literal(l) => s.push_str(l),
                TemplateStringFragment::Template(k) => {
                    let value = ctx
                        .get(k)
                        .ok_or(ContextEvaluationError::Interpolation(k.to_string()))?;
                    s.push_str(value.to_prim(ctx)?.as_string().as_str());
                }
            }
        }
        Ok(s)
    }
}

impl<'de> Deserialize<'de> for TemplateString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let fragments: Vec<TemplateStringFragment> = RE_TEMPLATE_STRING
            .captures_iter(&s)
            .map(|cap| {
                let matched = &cap[0];
                if matched.starts_with("${") && matched.ends_with('}') {
                    let placeholder = matched.trim_start_matches("${").trim_end_matches('}');
                    TemplateStringFragment::Template(placeholder.to_string())
                } else {
                    TemplateStringFragment::Literal(matched.to_string())
                }
            })
            .collect();

        if fragments.is_empty() {
            return Err(serde::de::Error::custom(format!(
                "Invalid template string: '{}' contains no valid fragments (expected at least one placeholder in the format '${{...}}')",
                s
            )));
        }

        Ok(TemplateString(fragments))
    }
}

impl Serialize for TemplateString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = String::new();
        for fragment in &self.0 {
            match fragment {
                TemplateStringFragment::Literal(l) => s.push_str(l),
                TemplateStringFragment::Template(t) => s.push_str(&format!("${{{}}}", t)),
            }
        }
        serializer.serialize_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use crate::data::GeoDeg;
    use crate::sites::Site;
    use std::path::PathBuf;

    #[test]
    fn test_template_string() {
        let ctx = Context {
            site: Site {
                id: 0,
                lon: GeoDeg::from(15.222),
                lat: GeoDeg::from(-15.23133),
            },
            run: crate::config::runs::RunConfig {
                name: String::from("r1"),
                template: PathBuf::from("dummy"),
                extra: [
                    (
                        "foo".to_string(),
                        ContextValue::Prim(PrimitiveContextValue::String("foo".to_string())),
                    ),
                    (
                        "bar".to_string(),
                        ContextValue::Prim(PrimitiveContextValue::String("bar".to_string())),
                    ),
                    (
                        "baz".to_string(),
                        ContextValue::TemplateString(
                            serde_json::from_str::<TemplateString>(r#""${foo}-${bar}""#).unwrap(),
                        ),
                    ),
                    (
                        "buz".to_string(),
                        ContextValue::TemplateString(
                            serde_json::from_str::<TemplateString>(r#""${baz}-baz-${baz}""#)
                                .unwrap(),
                        ),
                    ),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };

        assert_eq!(
            ctx.run.extra.get("baz").map(|v| v.to_prim(&ctx).unwrap()),
            Some(PrimitiveContextValue::String("foo-bar".to_string()))
        );
        assert_eq!(
            ctx.run.extra.get("buz").map(|v| v.to_prim(&ctx).unwrap()),
            Some(PrimitiveContextValue::String(
                "foo-bar-baz-foo-bar".to_string()
            ))
        );
    }
}
