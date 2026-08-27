# Development Plan:

## 0. Foundation (do first, blocks everything else):
### Planned Features:
    -> Route matching for /post/{id} pattern (parse path segments, extract id, dispatch)
    -> Full HTTP request parsing: headers, Content-Length, body (needed for comments/admin POSTs)
    -> Concurrency model decision: thread-per-connection instead of the current blocking
       single-connection loop (small change now, avoids a rewrite once more routes exist)
    -> Storage decision: pick DB approach (e.g. rusqlite) or file-backed JSON; replace the
       hardcoded items() vec and give PostCard an id-based lookup


## 1. Sync blog functionalities (Reader-Focused):
**Browse, Read, Open posts.**
### Planned Features:
    -> Scrollable Post list with pagination (10 posts per page)
    -> Store posts in db, open posts based on URL. (depends on Section 0: routing + storage)
    -> Add/Read comments on posts (depends on Section 0: body parsing)
    -> Like posts


## 2. Sync Admin Blog Functionalities:
### Planned Features:
    -> Auth mechanism for admin routes (even a hardcoded header token for v1)
    -> Delete/Add/Update posts.
    -> Delete comments.
    -> Disable comments.
    -> Pin post.


## 3. Async concurrency (scale beyond thread-per-connection):
### Planned Features:
    -> Ability to serve multiple clients at the same time (Crucial)
    -> Revisit once thread-per-connection (Section 0) shows real limits
