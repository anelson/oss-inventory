#!/usr/bin/env bash
#
# Automatically collect cargo metadata from every local copy of Elastio workspaces
set -euo pipefail

# Get the script's directory path
script_dir="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"

# Change to the target directory
cd "${script_dir}/../elastio" || exit 1

# Enumerate all directories in the target directory
for dir in */; do
    # Remove trailing slash from directory name
    dir_name="${dir%/}"
    echo "********************************"
    echo "Processing directory ${dir_name}"

    if [[ -f "${dir_name}/Cargo.toml" ]]; then
        pushd "${dir}"

        echo "Performing git pull..."
        git pull --prune > /dev/null || echo "Directory ${dir_name} can't do a git pull"

        # Run `cargo metadata` and save the output to a JSON file
        echo "Gathering cargo metadata..."
        cargo metadata --format-version 1 --all-features  | jq > "${script_dir}/workspaces/${dir_name}.json" || \
            (echo "Directory ${dir_name} isn't a valid Cargo workspace" && rm "${script_dir}/workspaces/${dir_name}.json")
        echo "Done"
        echo ""

        popd
    fi
done

# Handle some exceptions
cargo metadata --format-version 1 --all-features --manifest-path "${script_dir}/../elastio/awry/iscan/Cargo.toml" | jq > "${script_dir}/workspaces/awry.json"

