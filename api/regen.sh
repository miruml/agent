#!/bin/bash
set -e

git_repo_root_dir=$(git rev-parse --show-toplevel)
this_dir=$git_repo_root_dir/api
gen_dir=$this_dir/gen
lint_script=$git_repo_root_dir/scripts/lint.sh

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
