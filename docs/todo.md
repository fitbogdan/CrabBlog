



Failed login/register returns a bare 404 — no feedback to the user.

Rate limits don't show Error 429, and a reason to the user.

Two password fields to make sure they type the correct password

Password peak button

Comment form renders for logged-out users, who then hit a 401.

Templates re-read from disk on every render.

Migrations — schema changes currently require deleting the database.

The ("GET", ["post", id]) arm moves db where others borrow it.