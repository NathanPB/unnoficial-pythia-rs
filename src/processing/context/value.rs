use super::{Context, TemplateString};
use crate::processing::PipelineData;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum PrimitiveContextValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ContextValue {
    TemplateString(TemplateString),
    Prim(PrimitiveContextValue),
}

#[derive(Debug, Error)]
pub enum ContextEvaluationError {
    #[error("Placeholder '{0}' could not be resolved.")]
    Interpolation(String),
}

impl PrimitiveContextValue {
    pub fn as_string(&self) -> String {
        match self {
            PrimitiveContextValue::Bool(b) => b.to_string(),
            PrimitiveContextValue::Int(i) => i.to_string(),
            PrimitiveContextValue::Float(f) => f.to_string(),
            PrimitiveContextValue::String(s) => s.clone(),
        }
    }
}

impl ContextValue {
    pub fn to_prim(&self, ctx: &Context) -> Result<PrimitiveContextValue, ContextEvaluationError> {
        match self {
            ContextValue::Prim(p) => Ok(p.clone()),
            ContextValue::TemplateString(s) => {
                Ok(PrimitiveContextValue::String(s.interpolate(ctx)?))
            }
        }
    }
}

impl PipelineData for Context {}

impl Context {
    pub fn get(&self, key: &str) -> Option<ContextValue> {
        match key {
            "site_id" => Some(ContextValue::Prim(PrimitiveContextValue::String(
                self.site.id.to_string(),
            ))),
            "lng" => Some(ContextValue::Prim(PrimitiveContextValue::Float(
                self.site.lon.as_f64().into(),
            ))),
            "lon" => Some(ContextValue::Prim(PrimitiveContextValue::Float(
                self.site.lon.as_f64().into(),
            ))),
            "lat" => Some(ContextValue::Prim(PrimitiveContextValue::Float(
                self.site.lat.as_f64().into(),
            ))),
            "name" => Some(ContextValue::Prim(PrimitiveContextValue::String(
                self.run.name.clone(),
            ))),
            _ => self.run.extra.get(key).cloned(),
        }
    }

    pub fn dir(&self, base: &PathBuf) -> PathBuf {
        let mut path = base.clone();
        path.push(&self.run.name);
        path.push(&self.site.lon.ns(4));
        path.push(&self.site.lat.ew(4));
        path
    }

    pub fn tera(&self) -> Result<tera::Context, ContextEvaluationError> {
        let mut ctx = tera::Context::new();
        ctx.insert("site_id", &self.site.id);
        ctx.insert("soil_id", &self.site.id); // Backwards compatibility. In the original Pythia, the site ID was the soil ID.
        ctx.insert("lng", &self.site.lon.as_f32()); // Backwards compatibility, original Pythia impl used lat/lng instead of lon/lat.
        ctx.insert("lon", &self.site.lon.as_f32());
        ctx.insert("lat", &self.site.lat.as_f32());
        ctx.insert("name", &self.run.name);

        for (k, v) in &self.run.extra {
            ctx.insert(k, &v.to_prim(self)?);
        }

        Ok(ctx)
    }
}
