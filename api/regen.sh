#!/bin/bash
set -e

git_repo_root_dir=$(git rev-parse --show-toplevel)
this_dir=$git_repo_root_dir/api
openapi_dir=$this_dir/openapi
gen_dir=$this_dir/gen
lint_script=$git_repo_root_dir/scripts/lint.sh

# Check for any changes (staged, unstaged, or untracked)
cd "$openapi_dir"
echo ""
echo "Checking for any changes (staged, unstaged, or untracked)"
if [ -z "$(git status --porcelain)" ]; then
    echo "There are no changes to commit"
else
    echo "To keep track of what openapi spec the server is using, we store the GitHub commit of the spec used. For this to be accurate, all current changes to the openapi repository must be committed. Please remove or commit your changes to the openapi repository before refreshing the openapi spec."
    exit 1
fi
cd -

# generate the models
cd "$this_dir"
make gen

# client
gen_client_models_dir=$gen_dir/client/src/models
client_models_target_dir=$git_repo_root_dir/libs/openapi-client/src/models

# replace the target model directories with the generated ones
rm -rf "${client_models_target_dir:?}"/*

# copy all the files in the generated models dirs to the target models dirs
if [ ! -d "$client_models_target_dir" ]; then
    mkdir "$client_models_target_dir"
fi
cp -r "$gen_client_models_dir"/* "$client_models_target_dir"


# server
gen_server_models_dir=$gen_dir/server/src/models
server_models_target_dir=$git_repo_root_dir/libs/openapi-server/src/models

# replace the target model directories with the generated ones
rm -rf "${server_models_target_dir:?}"/*

# copy all the files in the generated models dirs to the target models dirs
if [ ! -d "$server_models_target_dir" ]; then
    mkdir "$server_models_target_dir"
fi
cp -r "$gen_server_models_dir"/* "$server_models_target_dir"

# lint the repository so that generated code doesn't cause huge diffs
$lint_script
