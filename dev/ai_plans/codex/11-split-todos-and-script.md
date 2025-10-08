# Split todos and add helper script

1. Inspect `dev/todo.md` to understand how todo items are structured and decide what should become an issue file.
2. Create per-todo markdown files in `dev/issues`, numbering them sequentially, slugging the titles, and adding `status: open`.
3. Implement `dev/todo.py` to add new todos via `$EDITOR`, fix filenames to match numbering/titles, and search todos by status using ripgrep. Update the flake to include ripgrep if necessary.
