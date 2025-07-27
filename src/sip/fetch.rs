use crate::sip::package::PackageRecord;
use chrono::Utc;
use reqwest::Error as ReqwestError;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct PyPiInfo {
    version: String,
    home_page: Option<String>,
    project_urls: Option<std::collections::HashMap<String, String>>,
}

#[derive(Deserialize)]
struct PyPiReleaseFile {
    packagetype: String,
    url: String,
    digests: HashMap<String, String>,
}

#[derive(Deserialize)]
struct PyPiResponse {
    info: PyPiInfo,
    releases: HashMap<String, Vec<PyPiReleaseFile>>,
}

pub async fn fetch_from_pypi(name: &str) -> Result<PackageRecord, ReqwestError> {
    let url = format!("https://pypi.org/pypi/{}/json", name);
    let resp: PyPiResponse = reqwest::get(&url).await?.json().await?;

    let version = resp.info.version;
    let files = resp
        .releases
        .get(&version)
        .expect("no release files for version");

    // pick sdist or fallback to first
    let file = files
        .iter()
        .find(|f| f.packagetype == "sdist")
        .unwrap_or(&files[0]);

    let sha = file.digests.get("sha256").expect("sha256 missing");

    let source = resp
        .info
        .home_page
        .or_else(|| {
            resp.info
                .project_urls
                .and_then(|m| m.get("Homepage").cloned())
        })
        .unwrap_or_else(|| format!("https://pypi.org/project/{}", name));

    Ok(PackageRecord {
        name: name.to_string(),
        version,
        hash: format!("sha256:{}", sha),
        trust_score: 0.0,
        endorsed_by: Vec::new(),
        last_reviewed: Utc::now().date_naive().to_string(),
        source,
    })
}

#[derive(Deserialize)]
struct CrateData {
    #[serde(rename = "max_version")]
    version: String,
}

#[derive(Deserialize)]
struct CratesResponse {
    #[serde(rename = "crate")]
    krate: CrateData,
}

#[derive(Deserialize)]
struct VersionData {
    checksum: String,
}

#[derive(Deserialize)]
struct VersionResponse {
    version: VersionData,
}

pub async fn fetch_from_crates(name: &str) -> Result<PackageRecord, ReqwestError> {
    // 1. get max_version
    let cr_url = format!("https://crates.io/api/v1/crates/{}", name);
    let cr: CratesResponse = reqwest::get(&cr_url).await?.json().await?;
    let version = cr.krate.version;

    // 2. get checksum
    let ver_url = format!("https://crates.io/api/v1/crates/{}/{}", name, version);
    let vr: VersionResponse = reqwest::get(&ver_url).await?.json().await?;
    let sha = vr.version.checksum;

    let source = format!("https://crates.io/crates/{}/{}", name, version);

    Ok(PackageRecord {
        name: name.to_string(),
        version,
        hash: format!("sha256:{}", sha),
        trust_score: 0.0,
        endorsed_by: Vec::new(),
        last_reviewed: Utc::now().date_naive().to_string(),
        source,
    })
}
