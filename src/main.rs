use cargo_metadata::{Dependency, DependencyKind, Metadata, Package, PackageId};
use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};

/// The details about a single dependency of the workspace
#[derive(Debug, Default)]
struct DependencyDetails {
    url: Option<String>,

    license: Option<String>,

    /// The paths to the workspace metadata files containing at least one crate that dependes on
    /// this dependency
    dependent_workspaces: HashSet<PathBuf>,

    /// The workspace crates that depend on this dependency
    dependent_crates: HashSet<String>,

    /// What kind of dependencies the workspace crates have on this dependency
    dependency_kinds: HashSet<DependencyKind>,
}

#[derive(Parser)]
#[clap(
    about = "Generates a CSV listing of all direct dependencies in one or more cargo workspaces"
)]
struct Opts {
    /// Path to the JSON file(s) containing the output of the `cargo metadata` command
    ///
    /// Each cargo workspace is represented by a single `cargo metadata` JSON output file.  In the
    /// resulting CSV, a column will include which cargo workspace JSON files included which
    /// dependencies.
    metadata_files: Vec<PathBuf>,

    /// When listing dependencies, include crates on registries other than crates.io
    ///
    /// Default is to list only public crates on crates.io
    #[clap(long)]
    include_private_crates: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    // Create a new CSV file
    let path = Path::new("workspace_dependencies.csv");
    let file = File::create(path)?;

    // We'll make a list of all crates, indexed by crate name, with details about what depends on
    // it.
    let mut dep_details: HashMap<String, DependencyDetails> = HashMap::new();

    // Run the `cargo metadata` command
    for metadata_file in opts.metadata_files {
        println!("Processing metadata file: {}", metadata_file.display());

        let metadata: Metadata = serde_json::from_reader(File::open(&metadata_file)?)?;

        // Find the crates that are part of the workspace, as opposed to dependencies.
        let workspace_members: HashMap<PackageId, &Package> = metadata
            .packages
            .iter()
            .filter(|package| metadata.workspace_members.contains(&package.id))
            .map(|package| (package.id.clone(), package))
            .collect();

        // For each crate in the workspace, get its dependencies.
        for (_, workspace_member) in workspace_members {
            let dependency_types = &[
                (
                    DependencyKind::Normal,
                    workspace_member
                        .dependencies
                        .iter()
                        .filter(|dep: &&Dependency| dep.kind == DependencyKind::Normal)
                        .collect::<Vec<_>>(),
                ),
                (
                    DependencyKind::Development,
                    workspace_member
                        .dependencies
                        .iter()
                        .filter(|dep: &&Dependency| dep.kind == DependencyKind::Development)
                        .collect::<Vec<_>>(),
                ),
                (
                    DependencyKind::Build,
                    workspace_member
                        .dependencies
                        .iter()
                        .filter(|dep: &&Dependency| dep.kind == DependencyKind::Build)
                        .collect::<Vec<_>>(),
                ),
            ];

            for (dep_type, dependencies) in dependency_types {
                for dependency in dependencies {
                    // By default we're only interested in dependencies on crates.io public crates
                    let is_public_dep = dependency.registry.is_none() && dependency.path.is_none();
                    if opts.include_private_crates || is_public_dep {
                        if let Some(crate_info) =
                            metadata.packages.iter().find(|p| p.name == dependency.name)
                        {
                            let crate_url = crate_info
                                .homepage
                                .as_deref()
                                .or(crate_info.repository.as_deref());

                            let dep =
                                dep_details
                                    .entry(dependency.name.clone())
                                    .or_insert_with(|| DependencyDetails {
                                        url: crate_url.map(|s| s.to_string()),
                                        license: crate_info.license.clone(),
                                        ..Default::default()
                                    });

                            dep.dependent_workspaces.insert(metadata_file.clone());
                            dep.dependent_crates.insert(workspace_member.name.clone());
                            dep.dependency_kinds.insert(*dep_type);
                        } else {
                            eprintln!(
                                "Warning: No crate found for dependency {} of workspace crate {} in workspace file {}",
                                dependency.name,
                                workspace_member.name,
                                metadata_file.to_string_lossy()
                            );
                        }
                    }
                }
            }
        }
    }

    let mut wtr = csv::Writer::from_writer(file);

    // Write the header row
    wtr.write_record([
        "Crate Name",
        "URL",
        "License",
        "Components",
        "Dependency Of",
        "Dependency Type",
    ])?;

    // Write out all of the dependency info to the CSV, sorted alphabetically by crate name
    let mut dep_details = dep_details.into_iter().collect::<Vec<_>>();
    dep_details.sort_unstable_by_key(|(name, _)| name.clone());

    for (crate_name, details) in dep_details {
        wtr.write_record([
            &crate_name,
            details.url.as_deref().unwrap_or(""),
            details.license.as_deref().unwrap_or(""),
            &details
                .dependent_workspaces
                .into_iter()
                .map(|p| p.file_stem().unwrap().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(","),
            &details
                .dependent_crates
                .into_iter()
                .collect::<Vec<_>>()
                .join(","),
            &details
                .dependency_kinds
                .into_iter()
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ])?;
    }

    // Flush the CSV writer to ensure the file is written
    wtr.flush()?;
    println!("CSV file generated at: {}", path.display());

    Ok(())
}
