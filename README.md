# `git-gfs`

Seamlessly manage large files without the (probable) costs of `git lfs`.

## Development status

Currently a work in progress.
The current target state is to upload all large files as git objects in a single repository.

This target may be extended to bypass limits by git providers.

## Installation

Please only install if you're confident you can undo any mess created by this software.

Do not rely on this software (yet).

## Setup

```sh
# add `gfs` filter to git config.
git config --local set "filter.gfs.clean" "git-gfs clean %f"
git config --local set "filter.gfs.clean" "git-gfs smudge %f"
git config --local set "filter.gfs.required" "true"
```

```sh
# add `pre-push` git hook.
echo $'#!/bin/sh\n\ngit-gfs pre-push' > .git/hooks/pre-push && chmod +x .git/hooks/pre-push
```

## Usage

These examples assume this is what is in `.gitattributes`.

```sh
# add file pattern to `.gitattributes`,
# with the gfs filter
echo $'*.bigfile filter=gfs -text\n' >> .gitattributes
```

### Checking in files

This has one test, and it seems to work.

```sh
# once you're big file is created,
# check it in
git add path/to/phat.bigfile

# the command `git-gfs clean path/to/phat.bigfile` is now running,
# splitting the file into parts and storing them inside the git database.

# commit your file
git commit -m 'test: add phat bigfile'
```

### Pushing files to the remote

This isn't tested! Likely doesn't work.

```sh
# push your change to the remote.
git push

# Uploading one part at a time first as references in the `pre-push` phase.
# Once complete, it will upload the pointers to the remote.
```

### Checking out files

This isn't tested! Likely doesn't work.
