HOUSEKEEPING: 
CLEAN UP HANDLE_CONNECTION, SEND EVERYONE INTO THEIR OWN FILES


## Major features:
Way later: Edit comment (For user and for admin)

Display image on the post page too

later: Reply to individual comments.

## Safety/Bugs:

!! GET form appends ? to edit

Devtool to easily change account status, quickly, for testing.

Way later: Migrations — schema changes currently require deleting the database.



## Frontend polish:

Create default image if I sent the post with no image, just add that one.

If post has no photo just display one of those good looking squares with a color and a letter on it or something

Beautiful error screens. Plus for rate limiting no redirect, just render on page



## PERFORMANCE:

Templates re-read from disk on every render, optimise

Implement connection pooling instead of mutex locking (make sure to record before and after time of that)





