#!/usr/bin/env bash
set -eu

if [ "$#" -gt 1 ]; then
  printf 'usage: %s [directory]\n' "$0" >&2
  exit 2
fi

cleanup=0
if [ "$#" -eq 1 ]; then
  workdir=$1
  mkdir -p "$workdir"
else
  workdir=$(mktemp -d "${TMPDIR:-/tmp}/git-db-example.XXXXXX")
  cleanup=1
fi

finish() {
  if [ "$cleanup" -eq 1 ]; then
    rm -rf "$workdir"
  fi
}
trap finish EXIT HUP INT TERM

if [ -e "$workdir/.git" ]; then
  printf 'refusing to use an existing Git repository: %s\n' "$workdir" >&2
  exit 1
fi

cd "$workdir"

# Keep every Git invocation non-interactive and pager-free.
git_prolly() {
  GIT_PAGER=cat PAGER=cat git-prolly "$@"
}

git init -q -b main
git config user.name 'Example User'
git config user.email example@example.com

git_prolly db init
printf '\n== Create a table and write rows ==\n'
git_prolly db table create users
git_prolly db put users alice '{"name":"Alice","role":"admin"}'
git_prolly db put users bob '{"name":"Bob","role":"viewer"}'
git_prolly db get users alice

git_prolly db status
printf '\n== Review, stage, and commit ==\n'
git_prolly db diff
git_prolly db add users
git_prolly db diff --staged
git_prolly db commit -m 'seed users'
git_prolly db log

printf '\n== Update and delete rows ==\n'
git_prolly db put users alice '{"name":"Alice","role":"owner"}'
git_prolly db rm users bob
git_prolly db diff
git_prolly db add -A
git_prolly db commit -m 'update users'
git_prolly db get users alice
git_prolly db get users bob || true
git_prolly db log

printf '\n== Export and import a table ==\n'
git_prolly db table export users -F users.json
cat users.json
printf '\n'
git_prolly db table import users_archive -c -F users.json
git_prolly db add users_archive
git_prolly db commit -m 'import users archive'
git_prolly db ls -v

if [ "$cleanup" -eq 0 ]; then
  printf '\nGit database: %s\n' "$workdir"
fi
