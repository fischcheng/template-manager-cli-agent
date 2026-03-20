use std::fs;
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::IF_NONE_MATCH;

use crate::error::TmError;
use crate::manifest::{
    Manifest, cached_etag_path, cached_manifest_path, config_dir, remote_manifest_url,
};

#[derive(Debug)]
pub struct ResolvedManifest {
    pub manifest: Manifest,
    pub source: ManifestSource,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ManifestSource {
    Embedded,
    Cache,
    ExplicitPath,
    Remote,
}

pub fn resolve_manifest(
    manifest_path: Option<&std::path::Path>,
) -> Result<ResolvedManifest, TmError> {
    if let Some(path) = manifest_path {
        let manifest = Manifest::from_path(path)?;
        return Ok(ResolvedManifest {
            manifest,
            source: ManifestSource::ExplicitPath,
            warnings: Vec::new(),
        });
    }

    let mut warnings = Vec::new();
    if let Some(url) = remote_manifest_url() {
        match refresh_cache(&url) {
            Ok(Some(manifest)) => {
                return Ok(ResolvedManifest {
                    manifest,
                    source: ManifestSource::Remote,
                    warnings,
                });
            }
            Ok(None) => {}
            Err(err) => warnings.push(format!("remote manifest refresh failed: {err}")),
        }
    }

    if let Ok(path) = cached_manifest_path()
        && path.exists()
    {
        let manifest = Manifest::from_path(&path)?;
        return Ok(ResolvedManifest {
            manifest,
            source: ManifestSource::Cache,
            warnings,
        });
    }

    let manifest = Manifest::embedded()?;
    Ok(ResolvedManifest {
        manifest,
        source: ManifestSource::Embedded,
        warnings,
    })
}

fn refresh_cache(url: &str) -> Result<Option<Manifest>, TmError> {
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
    let mut request = client.get(url);

    if let Ok(etag_path) = cached_etag_path()
        && etag_path.exists()
    {
        let etag = fs::read_to_string(etag_path)?;
        if !etag.trim().is_empty() {
            request = request.header(IF_NONE_MATCH, etag.trim().to_string());
        }
    }

    let response = request.send()?;
    if response.status() == StatusCode::NOT_MODIFIED {
        if let Ok(path) = cached_manifest_path()
            && path.exists()
        {
            return Ok(Some(Manifest::from_path(&path)?));
        }
        return Ok(None);
    }

    if !response.status().is_success() {
        return Ok(None);
    }

    let etag = response
        .headers()
        .get("etag")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let body = response.text()?;
    let manifest = Manifest::from_json_str(&body)?;

    let config_dir = config_dir()?;
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("manifest.json"), &body)?;
    if let Some(etag) = etag {
        fs::write(config_dir.join("manifest.etag"), etag)?;
    }

    Ok(Some(manifest))
}
