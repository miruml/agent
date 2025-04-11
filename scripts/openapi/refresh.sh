#!/bin/bash
set -e

openapi_dir=../../../openapi
openapi_configs_dir=$openapi_dir/configs
agent_dir=../../
git_info_file=$agent_dir/scripts/openapi/git-info.txt
gen_dir=./gen

# Check for any changes (staged, unstaged, or untracked)
cd $openapi_configs_dir
pwd

make bundle-agent

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
cd -

# keep track of the branch name + commit hash
{
    echo "Repository Name: $repository_name"
    echo "Branch Name: $branch_name"
    echo "Commit Hash: $commit_hash"
    echo "Commit Message: $commit_message"
} > $git_info_file

# generate the models
make gen

gen_models_dir=$gen_dir/src/models
target_models_dir=$agent_dir/agent/src/models

# replace the target model directories with the generated ones
rm -rf "${target_models_dir:?}"/*

# copy all the files in the generated models dirs to the target models dirs
if [ ! -d "$target_models_dir" ]; then
    mkdir "$target_models_dir"
fi
cp -r "$gen_models_dir"/* "$target_models_dir"
