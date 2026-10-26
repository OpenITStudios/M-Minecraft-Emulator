#!/usr/bin/env python3
"""Complete Sandboxie v1.18.6 source candidate from a pinned ZIP archive.

Only writes ordinary Git source files in the Local branch. Does not run the
installer, build binaries, touch Sandboxie services or inspect account data.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import zipfile

TAG = "v1.18.6"
COMMIT = "cdba7096240b93d79c5373aa81a75a1c5c71b883"
TREE = "f690477c07a40810f479f53973334ceae7e0ab13"
ARCHIVE_SHA256 = "22d8b004d4fe8dcf05654c4704a8b121c730352c1b406d2f1dad564dea451b33"
FILE_COUNT = 1627
ZIP_ROOT = "Sandboxie-1.18.6/"
OWNED = "engine/isolation-candidates/sandboxie-v1.18.6"


def guard(ok, why):
    if not ok:
        raise ValueError("Sandboxie source import refused: " + why)


def git(repo, *args, input=None):
    p = subprocess.run(["git", "-C", str(repo), *args], input=input, capture_output=True, check=False)
    guard(p.returncode == 0, f"Git {args[0]} failed: {p.stderr.decode(errors='replace')[:350]}")
    return p.stdout


def blob_hash(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\x00" + data).hexdigest()


def check_zip(path):
    with path.open("rb") as source_zip:
        digest = hashlib.file_digest(source_zip, "sha256").hexdigest()
    guard(digest == ARCHIVE_SHA256, "unexpected source ZIP SHA-256")
    result = {}
    with zipfile.ZipFile(path) as archive:
        for entry in archive.infolist():
            guard(not entry.flag_bits & 1, "encrypted ZIP entry")
            guard(entry.filename.startswith(ZIP_ROOT), "unexpected source ZIP root")
            if entry.is_dir():
                continue
            relative = entry.filename[len(ZIP_ROOT):]
            parts = relative.split("/")
            guard(all(p and p not in (".", "..") for p in parts) and "\\" not in relative
                  and not relative.startswith("/") and "\x00" not in relative,
                  "unsafe archive entry")
            guard(relative not in result, "duplicate ZIP entry")
            special = stat.S_IFMT(entry.external_attr >> 16)
            guard(special in (0, stat.S_IFREG), "ZIP entry is not a regular file")
            guard(entry.file_size <= 8 * 1024 * 1024, "oversized source file")
            data = archive.read(entry)
            mode = "100755" if relative == "Installer/Languages.iss" else "100644"
            result[relative] = (mode, blob_hash(data), data)
    guard(len(result) == FILE_COUNT, "wrong tracked file count")
    guard(len({k.casefold() for k in result}) == len(result), "case-insensitive path collision")
    return result


def import_archive(repo, archive_path, expected_head):
    repo = repo.resolve(strict=True)
    archive_path = archive_path.resolve(strict=True)
    guard(re.fullmatch("[0-9a-f]{40}", expected_head) is not None, "expected HEAD SHA missing")
    guard(Path(git(repo, "rev-parse", "--show-toplevel").decode().strip()).resolve(strict=True) == repo,
          "invalid Git repository root")
    guard(git(repo, "branch", "--show-current").decode().strip() == "Local", "not Local branch")
    guard(git(repo, "rev-parse", "HEAD").decode().strip() == expected_head, "Local HEAD changed")
    guard(not git(repo, "status", "--porcelain=v1", "--untracked-files=all").strip(),
          "working copy or index dirty")
    source = check_zip(archive_path)
    installed = {}
    raw = git(repo, "ls-tree", "-r", "-z", "HEAD", "--", OWNED)
    for record in raw.split(b"\0"):
        if not record:
            continue
        head, name = record.split(b"\t", 1)
        mode, kind, sha = head.decode().split(" ")
        name = name.decode("utf-8")
        guard(kind == "blob" and name.startswith(OWNED + "/"), "unexpected existing Git object")
        rel = name[len(OWNED) + 1:]
        guard(rel in source and mode == source[rel][0],
              "unexpected existing candidate path or mode: " + rel)
        # Git attributes can normalize ZIP bytes before indexing; the final
        # exact upstream tree SHA validates every existing Git blob as one unit.
        installed[rel] = sha
    root = repo / OWNED
    guard(not root.is_symlink(), "candidate folder is a symlink")
    missing = [(relative, spec) for relative, spec in source.items() if relative not in installed]
    if not missing:
        current = git(repo, "rev-parse", "HEAD:" + OWNED).decode().strip()
        guard(current == TREE, "candidate not complete despite matching file count")
        return {"status": "SOURCE_ALREADY_COMPLETE", "tree": current, "count": FILE_COUNT,
                "head": expected_head, "added": 0, "driverBuilt": False}
    for relative, (_, digest, data) in missing:
        dest = root.joinpath(*relative.split("/"))
        guard(not dest.exists() and not dest.is_symlink(), "would overwrite source: " + relative)
        current = root
        for part in relative.split("/")[:-1]:
            current = current / part
            guard(not current.is_symlink(), "path contains a symlink")
        dest.parent.mkdir(parents=True, exist_ok=True)
        with dest.open("xb") as out:
            out.write(data)
        if relative == "Installer/Languages.iss" and os.name != "nt":
            dest.chmod(0o755)
        guard(blob_hash(dest.read_bytes()) == digest, "copied file bytes mismatch")
    # Preserve committed blobs already checked out with Windows line endings.
    paths = b"".join((OWNED + "/" + relative).encode("utf-8") + b"\0" for relative, _ in missing)
    git(repo, "-c", "core.autocrlf=false", "add", "-f",
        "--pathspec-from-file=-", "--pathspec-file-nul", input=paths)
    git(repo, "update-index", "--chmod=+x", "--", OWNED + "/Installer/Languages.iss")
    staged_root = git(repo, "write-tree").decode().strip()
    staged_tree = git(repo, "rev-parse", staged_root + ":" + OWNED).decode().strip()
    guard(staged_tree == TREE, "staged source tree differs from official v1.18.6; no commit")
    staged_names = git(repo, "diff", "--cached", "--name-only", "-z").split(b"\0")
    guard(all(not n or n.decode("utf-8").startswith(OWNED + "/") for n in staged_names),
          "unrelated changes staged for commit")
    git(repo, "commit", "-m", "feat(engine): complete genuine Sandboxie v1.18.6 source candidate")
    final_head = git(repo, "rev-parse", "HEAD").decode().strip()
    guard(git(repo, "rev-parse", "HEAD:" + OWNED).decode().strip() == TREE,
          "committed tree differs from official source")
    return {"status": "FULL_V1186_SOURCE_OWNED_NOT_BUILT", "tree": TREE,
            "count": FILE_COUNT, "added": len(missing), "head": final_head,
            "driverBuilt": False, "installerExecuted": False, "minecraftVerified": False}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--archive", type=Path, required=True)
    p.add_argument("--repo-root", type=Path, required=True)
    p.add_argument("--expect-head", required=True)
    args = p.parse_args()
    try:
        print(json.dumps(import_archive(args.repo_root, args.archive, args.expect_head), indent=2))
    except (ValueError, OSError, zipfile.BadZipFile) as exc:
        print(str(exc), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
