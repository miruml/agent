#!/bin/bash
set -e

# add any new packages to the local apt repository
aptly repo add miru-agent ./dist/miru-agent*.deb

# remove any 'SNAPSHOT' packages from the local apt repository
aptly repo remove miru-agent 'miru-agent (*SNAPSHOT*)'

# publish the local apt repository
aptly publish update stable

# update the apt repository on github pages
agent_dist_dir=../../agent-dist
cd $agent_dist_dir

git switch gh-pages

# replace the existing apt repository with the new one
rm -rf "*"
cp -r "$HOME"/.aptly/public/* ./