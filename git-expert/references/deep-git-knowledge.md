# Git Expert Reference — Deep Git Knowledge

## Git Internals

Git is a content-addressable filesystem. Core concepts:
- **Blob**: File content (no filename metadata)
- **Tree**: Directory listing (maps names to blobs/trees)
- **Commit**: Snapshot + metadata (parent, author, message, tree)
- **Ref**: Pointer to a commit (branch, tag, HEAD)

## Reflog

`git reflog` records every HEAD movement. Recovery reference:
```
git reflog                    # View history
git reset --hard HEAD@{n}     # Restore to specific reflog entry
```

## Cherry-Pick

Apply specific commits to the current branch:
```
git cherry-pick <commit-hash>
git cherry-pick --continue    # After resolving conflicts
```

## Stash

Save and restore work-in-progress:
```
git stash push -m "message"   # Save with description
git stash list                # List stashes
git stash pop                 # Apply and remove
git stash drop                # Remove without applying
```

## Worktree

Work with multiple branches simultaneously:
```
git worktree add <path> <branch>
git worktree list
git worktree prune
```
