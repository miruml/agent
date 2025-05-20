#!/bin/bash
set -e

this_files_dir=$(pwd)
openapi_dir=../../openapi
openapi_dir=$(realpath $openapi_dir)
agent_dir=../
agent_dir=$(realpath $agent_dir)
git_info_file=$agent_dir/api/git-info.txt
gen_dir=./gen

cd $openapi_dir

# Check for any changes (staged, unstaged, or untracked)
echo ""
echo "Checking for any changes (staged, unstaged, or untracked)"
if [ -z "$(git status --porcelain)" ]; then
    echo "There are no changes to commit. Proceeding to refresh the openapi spec for the backend."
else
    echo "To keep track of what openapi spec the server is using, we store the GitHub commit of the spec used. For this to be accurate, all current changes to the openapi repository must be committed. Please remove or commit your changes to the openapi repository before refreshing the openapi spec for the backend."
    exit 1
fi
repository_name=$(git remote get-url origin)
branch_name=$(git branch --show-current)
commit_hash=$(git rev-parse HEAD)
commit_message=$(git log -1 --pretty=%B)

# keep track of the branch name + commit hash
{
    echo "Repository Name: $repository_name"
    echo "Branch Name: $branch_name"
    echo "Commit Hash: $commit_hash"
    echo "Commit Message: $commit_message"
} > $git_info_file

# generate the models
cd "$this_files_dir"
make gen

# client
gen_client_models_dir=$gen_dir/client/src/models
client_models_target_dir=$agent_dir/openapi/client/src/models

# replace the target model directories with the generated ones
rm -rf "${client_models_target_dir:?}"/*

# copy all the files in the generated models dirs to the target models dirs
if [ ! -d "$client_models_target_dir" ]; then
    mkdir "$client_models_target_dir"
fi
cp -r "$gen_client_models_dir"/* "$client_models_target_dir"


# server
gen_server_models_dir=$gen_dir/server/src/models
server_models_target_dir=$agent_dir/openapi/server/src/models

# replace the target model directories with the generated ones
rm -rf "${server_models_target_dir:?}"/*

# copy all the files in the generated models dirs to the target models dirs
if [ ! -d "$server_models_target_dir" ]; then
    mkdir "$server_models_target_dir"
fi
cp -r "$gen_server_models_dir"/* "$server_models_target_dir"
