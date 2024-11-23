# Notes

So this tool should archive, compress and split the files.

Split files should be in one place.

Two options.

Config for what files to compress as globs.

1. Replace the file it replaced with a short manifest. Not sure if this would cuause problems regarding git.
2. Put a huge manifest in 1 spot, probably easier.

Maybe we store git diff in archive?
So we unarchive the parts,

We can use git annotated tags and use it store config? of some sort? probably more related to diffing files.

We essentially need a way to easily switch between the remote (archived) version and the working version.
