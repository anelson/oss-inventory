# Inventory Open Source Cargo Worksapce Dependencies

This Rust program generates a CSV listing of all direct dependencies in one or more cargo workspaces. It reads the JSON
output from the `cargo metadata` command for each workspace, and produces a CSV listing containing the following fields:

- Name of the crate URL of the crate's repo (if any) License of the crate Components that this crate is a dependency of
- (workspace metadata files) Name of the cargo workspace crate that this crate is a dependency of Whether this crate is
- a regular dependency, a dev dependency, or a build dependency

It's specific to Elastio's needs to produce a quick list of all open source software packages that we use, as part of
due diligence.  Since almost all of our stack is Rust, this program was enough to gather most of the necessary
information.

## Usage

To use this program, you'll need to have Rust installed on your system. Then, follow these steps:

1. Run `cargo metadata --format-version 1 --all-features` in all of your Rust workspaces that you want to include in the
   inventory.  I put all of mine in this repo in a directory `workspaces/` which is ignored by git.  Name each
   workspace's JSON file with a descriptive name for that workspace, because the name (minus the `.json` extension) will
   be included in the resulting CSV
1. Do `cargo run -- JSON_FILES`.  If you took my advice and put them in `workspaces` you can run `cargo run --
   workspaces/*.json`
1. Take the resulting `workspace_dependencies.csv` file, load it into a spreadsheet, and massage it into a format
   suitable for your data room or due diligence recipient.

## Output

The program generates a CSV file named `workspace_dependencies.csv` in the current working directory. The CSV file
contains information about all direct dependencies in the provided cargo workspaces.

The format of this CSV is described in the code.  I made this format specifically for Elastio's needs, but you can
easily modify the code to produce a different structure as needed.

