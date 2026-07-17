#!/usr/bin/env bash
set -euo pipefail

root="${CODEX_WORKTREE_PATH:-$(pwd)}"
cd -- "${root}"
root="$(pwd -P)"
state_root="${root}/.codex-worktree"
cargo_target="${CARGO_TARGET_DIR:-${state_root}/cargo-target}"
narrated_root="${NARRATED_REPLAY_ROOT:-${state_root}/narrated-record-replay}"

if [[ -L "${state_root}" ]]; then
  printf '%s\n' "Refusing symlinked worktree state root: ${state_root}" >&2
  exit 2
fi

directories=(
  "${state_root}"
  "${state_root}/home"
  "${state_root}/scratch"
  "${state_root}/state"
  "${state_root}/tmp"
  "${state_root}/validation-artifacts"
  "${narrated_root}"
  "${cargo_target}"
)
for directory in "${directories[@]}"; do
  if [[ -L "${directory}" || ( -e "${directory}" && ! -d "${directory}" ) ]]; then
    printf '%s\n' "Refusing unsafe worktree directory: ${directory}" >&2
    exit 2
  fi
  mkdir -p -- "${directory}"
done

umask 077
build_dir="$(mktemp -d "${state_root}/.env-build.XXXXXX")"
env_file="${build_dir}/env.sh"
exec 3> "${env_file}"

write_export() {
  printf 'export %s=%q\n' "$1" "$2" >&3
}

write_export CODEX_WORKTREE_PATH "${root}"
write_export CARGO_TARGET_DIR "${cargo_target}"
write_export NARRATED_REPLAY_ROOT "${narrated_root}"
write_export TMPDIR "${state_root}/tmp"
write_export CODEX_SCRATCH_ROOT "${state_root}/scratch"
write_export CODEX_VALIDATION_ARTIFACTS "${state_root}/validation-artifacts"
exec 3>&-
chmod 600 "${env_file}"
mv -f -- "${env_file}" "${state_root}/env.sh"
rmdir -- "${build_dir}"

printf '%s\n' "Wrote ${state_root}/env.sh"
