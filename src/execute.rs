use crate::config::get_config;
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path};
use zip::{write::SimpleFileOptions, ZipWriter};

#[derive(Deserialize, Debug)]
pub struct ModConfig {
    id: String,
    name: Option<String>,
    description: String,
    version: String,
    homepage: Option<String>,
    author: Option<String>,

    icon: String,
    readme: Option<String>,
    changelog: Option<String>,

    #[serde(default)]
    project: ProjectConfig,

    #[serde(default)]
    dependencies: Vec<ModDependency>,

    #[serde(default)]
    extra_files: Vec<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ProjectConfig {
    csharp: Option<String>,
    godot: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ModDependency {
    id: Option<String>,
    thunderstore_version: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GDWeaveManifest {
    pub id: String,
    pub assembly_path: Option<String>,
    pub pack_path: Option<String>,
    pub dependencies: Vec<String>,
    pub metadata: Option<GDWeaveMetadata>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GDWeaveMetadata {
    pub name: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ThunderstoreManifest {
    name: String,
    version_number: String,
    website_url: String,
    description: String,
    dependencies: Vec<String>,
}

fn zip_dir(
    zip: &mut ZipWriter<std::fs::File>,
    path: &Path,
    prefix: &Path,
    ignore: &[String],
) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            // a bit of a hacky way of stripping unnecessary dll compilation files from the zips, hopefully that's ok!
            let lossy_string = file_name.to_string_lossy().to_string();
            if ignore.contains(&lossy_string)
                || lossy_string.ends_with(".deps.json")
                || lossy_string.ends_with(".pdb") {
                continue;
            }
        }

        let relative_path = path.strip_prefix(prefix)?;
        if path.is_dir() {
            zip_dir(zip, &path, prefix, ignore)?;
        } else {
            zip.start_file_from_path(relative_path, SimpleFileOptions::default())?;
            zip.write_all(&std::fs::read(&path)?)?;
        }
    }

    Ok(())
}

pub fn process(cfg_path: &Path) -> anyhow::Result<()> {
    let manifestation_config = get_config().unwrap_or_default();

    let cfg_dir = dunce::canonicalize(cfg_path.parent().expect("Failed to get parent directory"))?;
    let cfg = std::fs::read_to_string(cfg_path)?;
    let cfg: ModConfig = toml::from_str(&cfg)?;

    let work_dir = cfg_path
        .parent()
        .expect("Failed to get parent directory")
        .join("manifestation");
    if work_dir.exists() {
        std::fs::remove_dir_all(&work_dir)?;
    }
    std::fs::create_dir_all(&work_dir)?;
    let work_dir = dunce::canonicalize(&work_dir)?;

    let mod_dir = work_dir.join("GDWeave").join("mods").join(&cfg.id);
    std::fs::create_dir_all(&mod_dir)?;

    if let Some(ref csharp) = cfg.project.csharp {
        let csharp = cfg_dir.join(&csharp);
        let gdweave_path = manifestation_config
            .gdweave_path
            .expect("No GDWeave path provided - did you run the setup?");

        let status = std::process::Command::new("dotnet")
            .env("GDWeavePath", gdweave_path)
            .arg("build")
            .arg(&csharp)
            .arg("-c")
            .arg("Release")
            .arg("-o")
            .arg(&mod_dir)
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to build C# project");
        }
    }

    if let Some(ref godot) = cfg.project.godot {
        let godot = cfg_dir.join(&godot);
        let godot_path = manifestation_config
            .godot_path
            .expect("No Godot editor path provided - did you run the setup?");

        let export_presets_cfg = godot
            .parent()
            .expect("Failed to get parent directory")
            .join("export_presets.cfg");
        if !export_presets_cfg.exists() {
            let export_preset = r#"
[preset.0]
name="manifestation"
platform="Windows Desktop"
runnable=true
export_filter="all_resources"
include_filter=""
exclude_filter=""

[preset.0.options]
application/modify_resources=false
"#
            .trim();

            std::fs::write(&export_presets_cfg, export_preset)?;
        }

        let status = std::process::Command::new(godot_path)
            .arg("--no-window")
            .arg("--path")
            .arg(&godot.parent().expect("Failed to get parent directory"))
            .arg("--export-pack")
            .arg("manifestation")
            .arg(&mod_dir.join(format!("{}.pck", cfg.id)))
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to export Godot project");
        }
    }

    let gdweave_manifest = GDWeaveManifest {
        id: cfg.id.clone(),
        assembly_path: if cfg.project.csharp.is_some() {
            Some(format!("{}.dll", cfg.id))
        } else {
            None
        },
        pack_path: if cfg.project.godot.is_some() {
            Some(format!("{}.pck", cfg.id))
        } else {
            None
        },
        dependencies: cfg
            .dependencies
            .iter()
            .filter_map(|dep| dep.id.clone())
            .collect(),
        metadata: Some(GDWeaveMetadata {
            name: Some(cfg.name.clone().unwrap_or(cfg.id.clone())),
            author: cfg.author.clone(),
            version: Some(cfg.version.clone()),
            description: Some(cfg.description.clone()),
            homepage: cfg.homepage.clone(),
        }),
    };
    std::fs::write(
        mod_dir.join("manifest.json"),
        serde_json::to_string_pretty(&gdweave_manifest)?,
    )?;

    let thunderstore_manifest = ThunderstoreManifest {
        name: cfg.name.unwrap_or(cfg.id.clone()),
        version_number: cfg.version.clone(),
        website_url: cfg.homepage.unwrap_or_default(),
        description: cfg.description,
        dependencies: cfg
            .dependencies
            .iter()
            .filter_map(|dep| dep.thunderstore_version.clone())
            .collect(),
    };
    std::fs::write(
        work_dir.join("manifest.json"),
        serde_json::to_string_pretty(&thunderstore_manifest)?,
    )?;

    if let Some(ref readme) = cfg.readme {
        let readme = cfg_dir.join(readme);
        std::fs::copy(&readme, work_dir.join("README.md"))?;
    }

    if let Some(ref changelog) = cfg.changelog {
        let changelog = cfg_dir.join(changelog);
        std::fs::copy(&changelog, work_dir.join("CHANGELOG.md"))?;
    }

    let icon = cfg_dir.join(cfg.icon);
    let image = image::open(&icon)?;
    if image.width() != 256 || image.height() != 256 {
        anyhow::bail!("Icon must be 256x256");
    }

    std::fs::copy(&icon, work_dir.join("icon.png"))?;
    std::fs::copy(&icon, mod_dir.join("icon.png"))?;

    for extra_file in cfg.extra_files {
        let extra_file = cfg_dir.join(extra_file);
        std::fs::copy(&extra_file, mod_dir.join(extra_file.file_name().unwrap()))?;
    }

    let ignore = vec!["thunderstore.zip".to_string(), format!("{}.zip", cfg.id)];

    let github_zip_writer = std::fs::File::create(work_dir.join(format!("{}.zip", cfg.id)))?;
    let mut github_zip = ZipWriter::new(github_zip_writer);
    zip_dir(&mut github_zip, &mod_dir, &mod_dir, &ignore)?;
    github_zip.finish()?;

    let thunderstore_zip_writer = std::fs::File::create(work_dir.join("thunderstore.zip"))?;
    let mut thunderstore_zip = ZipWriter::new(thunderstore_zip_writer);
    zip_dir(&mut thunderstore_zip, &work_dir, &work_dir, &ignore)?;
    thunderstore_zip.finish()?;

    Ok(())
}
