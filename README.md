# Git Flow

## Status

1. `Uninitialized`: This is a directory that has not been initialized with `git init`.

2. `Initialized`: This is a directory that has been initialized with `git init` but does not have any commits yet.

3. `Detached HEAD`: This is when you're not currently on any branch, often as a result of checking out a specific commit.

4. `Dirty`: This means there are uncommitted changes in the working directory or staging area.

5. `Clean`: This means there are no uncommitted changes in the working directory or staging area.

6. `Merge Conflict`: This status occurs when a merge cannot be performed automatically and requires manual intervention to resolve conflicts.

7. `Rebasing`: This status is active when a rebase operation is in progress.

8. `Bisecting`: This status is active when a bisect operation is in progress.

9. `Cherry-Picking`: This status is active when a cherry-pick operation is in progress.

10. `Ahead`: This means your local branch has commits that haven't been pushed to the remote branch.

11. `Behind`: This means the remote branch has commits that haven't been pulled to the local branch.

12. `Diverged`: This means the local branch and the remote branch have both had commits that the other one doesn't have.

13. `Stashing`: This status is active when changes have been temporarily stored with `git stash` to be reapplied later.

14. `Applying Stash`: This status is active when changes stored in a stash are being reapplied to the working directory.

15. `Reverting`: This status is active when a revert operation is in progress.

16. `Amending`: This status is active when an amend operation is in progress, i.e., modifying the last commit.

17. `Interactive Rebase`: This status is active when an interactive rebase operation is in progress.

18. `Tagging`: This status is active when a tagging operation is in progress.

19. `Pushing`: This status is active when a push operation is in progress.

20. `Pulling`: This status is active when a pull operation is in progress.

- `DirtyUnstaged`: This status indicates there are changes in the working directory that have not been staged or committed yet. In this state, you cannot make a commit until you stage the changes with `git add`.

- `DirtyStaged`: This status indicates there are changes that have been staged but not committed yet. In this state, you can make a commit.

- `CleanUnstaged`: This status indicates there are no uncommitted changes in the working directory and nothing has been staged. This means you have not made any changes since the last commit.

- `CleanStaged`: This status indicates there are no uncommitted changes in the working directory, but there are changes that have been staged. This means you have made some changes, staged them with `git add`, but have not committed them yet.

- `DirtyUnstaged`: Changes in the working directory that have not been staged or committed.
- `DirtyStaged`: Changes that have been staged but not committed.
- `Clean`: No uncommitted changes in the working directory and nothing has been staged.
- `PartialCommit`: This status indicates that some changes have been committed, while other changes remain uncommitted in the working directory or staging area.
- `Staged`: No uncommitted changes in the working directory, but there are changes that have been staged.

- `FullyStaged`: This status indicates that all changes in the working directory have been staged. This means you have added all changes with `git add` and they are ready to be committed.

- `PartiallyStaged`: This status indicates that some changes in the working directory have been staged, while others have not. This means you have added some changes with `git add`, but not all, and they are ready to be committed.
