// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use defaults_rs::{Domain, PrefValue, Preferences};
use std::collections::{HashMap, HashSet};
use toml::Table;
use toml_edit::{DocumentMut, Item};

use crate::domains::convert::toml_edit_to_toml;

/// Collect all tables in `[set]`, parse with `toml_edit` to properly handle inline tables,
/// and return a map domain → settings.
pub async fn collect(doc: &DocumentMut) -> Result<HashMap<String, Table>> {
    let mut out = HashMap::new();

    if let Some(Item::Table(set)) = doc.get("set") {
        for (domain, item) in set {
            if let Item::Table(t) = item {
                collect_table(domain, t, &mut out)?;
            }
        }
    }

    Ok(out)
}

fn collect_table(
    domain: &str,
    table: &toml_edit::Table,
    out: &mut HashMap<String, Table>,
) -> Result<()> {
    let mut settings = Table::new();

    for (key, item) in table {
        match item {
            Item::Value(v) => {
                settings.insert(key.to_string(), toml_edit_to_toml(v)?);
            }
            Item::Table(t) => {
                collect_table(&format!("{domain}.{key}"), t, out)?;
            }
            _ => {}
        }
    }

    if !settings.is_empty() {
        out.insert(domain.to_string(), settings);
    }
    Ok(())
}

/// Returns all system domains as strings.
pub fn get_sys_domain_strings() -> Result<HashSet<String>> {
    let doms: HashSet<String> = Preferences::list_domains()?
        .iter()
        .map(|f| f.to_string())
        .collect();

    Ok(doms)
}

/// Given the TOML domain and key, figure out the true domain-key pair for targeting system domains.
/// As an extra argument, this function also receives the current domains list for validation.
#[must_use]
pub fn get_effective_sys_domain_key(domain: &str, key: &str) -> (String, String) {
    let dom = {
        if domain.strip_prefix("NSGlobalDomain.").is_some() {
            // NSGlobalDomain.foo -> NSGlobalDomain
            "NSGlobalDomain".into()
        } else if domain == "NSGlobalDomain" {
            domain.into()
        } else {
            // anything else gets com.apple.
            format!("com.apple.{domain}")
        }
    };

    let k = {
        if dom == "NSGlobalDomain" && domain.starts_with("NSGlobalDomain.") {
            // NSGlobalDomain.foo + key  -> foo.key
            let rest = &domain["NSGlobalDomain.".len()..];
            format!("{rest}.{key}")
        } else {
            key.into()
        }
    };
    (dom, k)
}

/// Read the current value of a defaults key, if any.
pub async fn read_current(eff_domain: &str, eff_key: &str) -> Option<PrefValue> {
    let domain_obj = if eff_domain == "NSGlobalDomain" {
        Domain::Global
    } else {
        Domain::User(eff_domain.to_string())
    };

    (Preferences::read(domain_obj, eff_key)).ok()
}
