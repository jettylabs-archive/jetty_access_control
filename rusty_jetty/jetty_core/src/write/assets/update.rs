//! Update the configuration for changes to users and groups

use std::{collections::BTreeSet, fs};

use anyhow::Result;

use crate::{write::UpdateConfig, Jetty};

use super::{parser::parse_to_file_map, YamlAssetDoc, YamlDefaultPolicy, YamlPolicy};

impl UpdateConfig for YamlAssetDoc {
    fn update_user_name(&mut self, old: &str, new: &str) -> anyhow::Result<bool> {
        let mut modified_policies = false;
        let policies = self.policies.to_owned();
        let new_policies = policies
            .into_iter()
            .map(|mut p| -> Result<_> {
                modified_policies = p.update_user_name(old, new)? || modified_policies;
                Ok(p)
            })
            .collect::<Result<BTreeSet<YamlPolicy>>>()?;
        if modified_policies {
            self.policies = new_policies;
        }

        let mut modified_default_policies = false;
        let default_policies = self.default_policies.to_owned();
        let new_default_policies = default_policies
            .into_iter()
            .map(|mut p| -> Result<_> {
                modified_default_policies =
                    p.update_user_name(old, new)? || modified_default_policies;
                Ok(p)
            })
            .collect::<Result<BTreeSet<YamlDefaultPolicy>>>()?;
        if modified_default_policies {
            self.default_policies = new_default_policies;
        }

        Ok(modified_policies || modified_default_policies)
    }

    fn remove_user_name(&mut self, name: &str) -> anyhow::Result<bool> {
        let mut modified_policies = false;
        let policies = self.policies.to_owned();
        let new_policies = policies
            .into_iter()
            .map(|mut p| -> Result<_> {
                modified_policies = p.remove_user_name(name)? || modified_policies;
                Ok(p)
            })
            .filter(|p| match p {
                Ok(p) => !(p.users.is_none() && p.groups.is_none()),
                Err(_) => true,
            })
            .collect::<Result<BTreeSet<YamlPolicy>>>()?;
        if modified_policies {
            self.policies = new_policies;
        }

        let mut modified_default_policies = false;
        let default_policies = self.default_policies.to_owned();
        let new_default_policies = default_policies
            .into_iter()
            .map(|mut p| -> Result<_> {
                modified_default_policies = p.remove_user_name(name)? || modified_default_policies;
                Ok(p)
            })
            .filter(|p| match p {
                Ok(p) => !(p.users.is_none() && p.groups.is_none()),
                Err(_) => true,
            })
            .collect::<Result<BTreeSet<YamlDefaultPolicy>>>()?;
        if modified_default_policies {
            self.default_policies = new_default_policies;
        }

        Ok(modified_policies || modified_default_policies)
    }

    fn update_group_name(&mut self, old: &str, new: &str) -> anyhow::Result<bool> {
        let mut modified_policies = false;
        let policies = self.policies.to_owned();
        let new_policies = policies
            .into_iter()
            .map(|mut p| -> Result<_> {
                modified_policies = p.update_group_name(old, new)? || modified_policies;
                Ok(p)
            })
            .collect::<Result<BTreeSet<YamlPolicy>>>()?;
        if modified_policies {
            self.policies = new_policies;
        }

        let mut modified_default_policies = false;
        let default_policies = self.default_policies.to_owned();
        let new_default_policies = default_policies
            .into_iter()
            .map(|mut p| -> Result<_> {
                modified_default_policies =
                    p.update_group_name(old, new)? || modified_default_policies;
                Ok(p)
            })
            .collect::<Result<BTreeSet<YamlDefaultPolicy>>>()?;
        if modified_default_policies {
            self.default_policies = new_default_policies;
        }

        Ok(modified_policies || modified_default_policies)
    }

    fn remove_group_name(&mut self, name: &str) -> anyhow::Result<bool> {
        let mut modified_policies = false;
        let policies = self.policies.to_owned();
        let new_policies = policies
            .into_iter()
            .map(|mut p| -> Result<_> {
                modified_policies = p.remove_group_name(name)? || modified_policies;
                Ok(p)
            })
            .filter(|p| match p {
                Ok(p) => !(p.users.is_none() && p.groups.is_none()),
                Err(_) => true,
            })
            .collect::<Result<BTreeSet<YamlPolicy>>>()?;
        if modified_policies {
            self.policies = new_policies;
        }

        let mut modified_default_policies = false;
        let default_policies = self.default_policies.to_owned();
        let new_default_policies = default_policies
            .into_iter()
            .map(|mut p| -> Result<_> {
                modified_default_policies = p.remove_group_name(name)? || modified_default_policies;
                Ok(p)
            })
            .filter(|p| match p {
                Ok(p) => !(p.users.is_none() && p.groups.is_none()),
                Err(_) => true,
            })
            .collect::<Result<BTreeSet<YamlDefaultPolicy>>>()?;
        if modified_default_policies {
            self.default_policies = new_default_policies;
        }

        Ok(modified_policies || modified_default_policies)
    }
}

impl UpdateConfig for YamlPolicy {
    fn update_user_name(&mut self, old: &str, new: &str) -> anyhow::Result<bool> {
        if let Some(users) = &mut self.users {
            if users.remove(old) {
                users.insert(new.to_string());
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn remove_user_name(&mut self, name: &str) -> anyhow::Result<bool> {
        let mut modified = false;
        let mut set_to_none = false;
        if let Some(users) = &mut self.users {
            if users.remove(name) {
                modified = true;
                if users.is_empty() {
                    set_to_none = true;
                };
            }
        }
        if set_to_none {
            self.users = None;
        }
        Ok(modified)
    }

    fn update_group_name(&mut self, old: &str, new: &str) -> anyhow::Result<bool> {
        if let Some(groups) = &mut self.groups {
            if groups.remove(old) {
                groups.insert(new.to_string());
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn remove_group_name(&mut self, name: &str) -> anyhow::Result<bool> {
        let mut modified = false;
        let mut set_to_none = false;
        if let Some(groups) = &mut self.groups {
            if groups.remove(name) {
                modified = true;
                if groups.is_empty() {
                    set_to_none = true;
                };
            }
        }
        if set_to_none {
            self.groups = None;
        }
        Ok(modified)
    }
}

impl UpdateConfig for YamlDefaultPolicy {
    fn update_user_name(&mut self, old: &str, new: &str) -> anyhow::Result<bool> {
        if let Some(users) = &mut self.users {
            if users.remove(old) {
                users.insert(new.to_string());
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn remove_user_name(&mut self, name: &str) -> anyhow::Result<bool> {
        let mut modified = false;
        let mut set_to_none = false;
        if let Some(groups) = &mut self.groups {
            if groups.remove(name) {
                modified = true;
                if groups.is_empty() {
                    set_to_none = true;
                };
            }
        }
        if set_to_none {
            self.groups = None;
        }
        Ok(modified)
    }

    fn update_group_name(&mut self, old: &str, new: &str) -> anyhow::Result<bool> {
        if let Some(groups) = &mut self.groups {
            if groups.remove(old) {
                groups.insert(new.to_string());
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn remove_group_name(&mut self, name: &str) -> anyhow::Result<bool> {
        let mut modified = false;
        let mut set_to_none = false;
        if let Some(groups) = &mut self.groups {
            if groups.remove(name) {
                modified = true;
                if groups.is_empty() {
                    set_to_none = true;
                };
            }
        }
        if set_to_none {
            self.groups = None;
        }
        Ok(modified)
    }
}

pub(crate) fn update_user_name(_jetty: &Jetty, old: &str, new: &str) -> Result<()> {
    let config = parse_to_file_map()?;
    for (path, mut asset_doc) in config {
        if asset_doc.update_user_name(old, new)? {
            let doc_string = yaml_peg::serde::to_string(&asset_doc)?;
            fs::write(path, doc_string)?;
        };
    }
    Ok(())
}

pub(crate) fn remove_user_name(_jetty: &Jetty, name: &str) -> Result<()> {
    let config = parse_to_file_map()?;
    for (path, mut asset_doc) in config {
        if asset_doc.remove_user_name(name)? {
            let doc_string = yaml_peg::serde::to_string(&asset_doc)?;
            fs::write(path, doc_string)?;
        };
    }
    Ok(())
}

pub(crate) fn update_group_name(_jetty: &Jetty, old: &str, new: &str) -> Result<()> {
    let config = parse_to_file_map()?;
    for (path, mut asset_doc) in config {
        if asset_doc.update_group_name(old, new)? {
            let doc_string = yaml_peg::serde::to_string(&asset_doc)?;
            fs::write(path, doc_string)?;
        };
    }
    Ok(())
}

pub(crate) fn remove_group_name(_jetty: &Jetty, name: &str) -> Result<()> {
    let config = parse_to_file_map()?;
    for (path, mut asset_doc) in config {
        if asset_doc.remove_group_name(name)? {
            let doc_string = yaml_peg::serde::to_string(&asset_doc)?;
            fs::write(path, doc_string)?;
        };
    }
    Ok(())
}
