HOUSEKEEPING: 
CLEAN UP HANDLE_CONNECTION, SEND EVERYONE INTO THEIR OWN FILES


## Major features:
Way later: Edit comment (For user and for admin)

Display image on the post page too

later: Reply to individual comments.

## Safety/Bugs:

Subcomment thread only replaces one color - of the parent comment

--- Rewrite /post/create to use the new function: get_multipart_parts, instead of just taking the bytes by index ---


!! GET form appends ? to edit


Devtool to easily change account status, quickly, for testing.

Way later: Migrations — schema changes currently require deleting the database.

Rate limiting on sending comments



## Frontend polish:

Create default image if I sent the post with no image, just add that one.

If post has no photo just display one of those good looking squares with a color and a letter on it or something

Beautiful error screens. Plus for rate limiting no redirect, just render on page



## PERFORMANCE:

EASY ONE: Instead of doing request = String::from_utf8_lossy(&request_bytes) on every request, (Including multipart requests where its redundant to turn the body into a utf8 string), split the request bytes into:
    Header, Body.

    Later each function can take their own desired parts and decode them



Templates re-read from disk on every render, optimise

Implement connection pooling instead of mutex locking (make sure to record before and after time of that)





