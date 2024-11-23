# `obfuscat` \[WIP]

Seamlessly upload large files to a git repository without using LFS.

## Caveats

As with all "workarounds", there are some caveats to this approach.
The tradeoffs are acceptable for the benefit they provide.

1. Requires git hooks.
   a. `post-commit` to bypass file size limits break up large files into smaller, compressed files.
   b. `pre-push` to bypass the pack limit, pushing appropriately sized packs.
2. Requires git tags. Reserved tags are:
   a. `obfuscat:encoded:<commit>` to save git state to be pushed to remote.
   b. `obfuscat:decoded:<commit>` to save the git state to be used locally.

## Installation

1. Install git hooks.
   1. `echo 'obfuscat post-commit'` >> .git/hooks/post-commit
   2. `echo 'obfuscat pre-push'` >> .git/hooks/pre-push
2.
