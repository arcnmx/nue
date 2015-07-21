#!/bin/bash
set -eu

TAG=$1

cd target/doc

git init
git config user.name "arcnmx"
git config user.email "arcnmx@users.noreply.github.com"

git remote add origin "https://$GH_TOKEN@github.com/$TRAVIS_REPO_SLUG"
git fetch origin gh-pages
git checkout -b gh-pages
git reset origin/gh-pages

git add -A .
git commit -m "$TAG"
git push -q origin HEAD:gh-pages
