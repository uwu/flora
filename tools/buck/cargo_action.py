#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path


def resolve_workspace_root(marker: str) -> Path:
    marker_path = Path(marker).resolve()
    for candidate in [marker_path.parent, *marker_path.parents]:
        cargo_toml = candidate / 'Cargo.toml'
        if cargo_toml.exists():
            return candidate
    raise RuntimeError(f'failed to resolve workspace root from marker: {marker}')


def build_env(overrides: list[str]) -> dict[str, str]:
    env = dict(os.environ)
    for item in overrides:
        if '=' not in item:
            raise RuntimeError(f'invalid env override {item!r}, expected KEY=VALUE')
        key, value = item.split('=', 1)
        env[key] = value

    # Nix wrappers often export "--resource-dir=..." but bindgen expects "-resource-dir=...".
    # Normalize it here so Buck builds remain portable across shells/environments.
    bindgen_args = env.get('BINDGEN_EXTRA_CLANG_ARGS')
    if bindgen_args:
        env['BINDGEN_EXTRA_CLANG_ARGS'] = bindgen_args.replace('--resource-dir=', '-resource-dir=')

    return env


def run_command(cmd: list[str], cwd: Path, env: dict[str, str]) -> int:
    proc = subprocess.run(cmd, cwd=cwd, env=env)
    return proc.returncode


def choose_library_artifact(filenames: list[str], kind: str) -> str | None:
    paths = [Path(name) for name in filenames]

    if kind == 'lib':
        preferred = ['.rlib', '.a', '.so', '.dylib', '.dll']
    elif kind == 'proc-macro':
        preferred = ['.so', '.dylib', '.dll', '.rlib']
    else:
        preferred = ['.rlib', '.a', '.so', '.dylib', '.dll']

    for extension in preferred:
        for path in paths:
            if path.suffix == extension:
                return str(path)

    if paths:
        return str(paths[-1])

    return None


def run_artifact_mode(args: argparse.Namespace) -> int:
    workspace_root = resolve_workspace_root(args.workspace_marker)
    env = build_env(args.env)

    cmd = [
        'cargo',
        'build',
        '--message-format=json-render-diagnostics',
        '--package',
        args.package,
    ]

    if args.release:
        cmd.append('--release')

    if args.kind in ('lib', 'proc-macro'):
        cmd.append('--lib')
    elif args.kind == 'bin':
        if args.target_name:
            cmd.extend(['--bin', args.target_name])

    cmd.extend(args.cargo_args)

    proc = subprocess.Popen(
        cmd,
        cwd=workspace_root,
        env=env,
        stdout=subprocess.PIPE,
        stderr=None,
        text=True,
        bufsize=1,
    )

    selected_artifact: str | None = None

    assert proc.stdout is not None

    for line in proc.stdout:
        line = line.rstrip('\n')
        if not line:
            continue

        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            print(line, file=sys.stderr)
            continue

        if payload.get('reason') != 'compiler-artifact':
            continue

        package_id = payload.get('package_id', '')
        package_matches = package_id.startswith(f'{args.package} ') or f'#{args.package}@' in package_id
        if not package_matches:
            continue

        target = payload.get('target', {})
        target_name = target.get('name')
        target_kinds = set(target.get('kind', []))

        if args.target_name and target_name != args.target_name:
            continue

        if args.kind == 'bin' and 'bin' not in target_kinds:
            continue
        if args.kind == 'lib' and 'lib' not in target_kinds:
            continue
        if args.kind == 'proc-macro' and 'proc-macro' not in target_kinds:
            continue

        if args.kind == 'bin':
            executable = payload.get('executable')
            if executable:
                selected_artifact = executable
        else:
            filenames = payload.get('filenames', [])
            artifact = choose_library_artifact(filenames, args.kind)
            if artifact:
                selected_artifact = artifact

    rc = proc.wait()
    if rc != 0:
        return rc

    if not selected_artifact:
        print(
            f'no artifact found for package={args.package} kind={args.kind} target_name={args.target_name}',
            file=sys.stderr,
        )
        return 1

    artifact_path = Path(selected_artifact)
    output_path = Path(args.out)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(artifact_path, output_path)

    if args.kind == 'bin':
        output_mode = output_path.stat().st_mode
        output_path.chmod(output_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    return 0


def run_command_mode(args: argparse.Namespace) -> int:
    workspace_root = resolve_workspace_root(args.workspace_marker)
    env = build_env(args.env)

    cmd = ['cargo', *args.cargo_args]
    if not args.cargo_args:
        print('no cargo arguments provided for command mode', file=sys.stderr)
        return 1

    rc = run_command(cmd, workspace_root, env)
    if rc != 0:
        return rc

    output_path = Path(args.out)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text('ok\n', encoding='utf-8')
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description='buck helper for cargo actions')
    subparsers = parser.add_subparsers(dest='mode', required=True)

    artifact = subparsers.add_parser('artifact')
    artifact.add_argument('--workspace-marker', required=True)
    artifact.add_argument('--out', required=True)
    artifact.add_argument('--package', required=True)
    artifact.add_argument('--kind', choices=['bin', 'lib', 'proc-macro'], required=True)
    artifact.add_argument('--target-name')
    artifact.add_argument('--release', action='store_true')
    artifact.add_argument('--env', action='append', default=[])
    artifact.add_argument('cargo_args', nargs=argparse.REMAINDER)

    command = subparsers.add_parser('command')
    command.add_argument('--workspace-marker', required=True)
    command.add_argument('--out', required=True)
    command.add_argument('--env', action='append', default=[])
    command.add_argument('cargo_args', nargs=argparse.REMAINDER)

    return parser.parse_args()


def normalize_remainder(args: argparse.Namespace) -> None:
    if hasattr(args, 'cargo_args') and args.cargo_args and args.cargo_args[0] == '--':
        args.cargo_args = args.cargo_args[1:]


def main() -> int:
    args = parse_args()
    normalize_remainder(args)

    if args.mode == 'artifact':
        return run_artifact_mode(args)
    if args.mode == 'command':
        return run_command_mode(args)

    print(f'unsupported mode: {args.mode}', file=sys.stderr)
    return 1


if __name__ == '__main__':
    raise SystemExit(main())
