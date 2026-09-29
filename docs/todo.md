HOUSEKEEPING: 
CLEAN UP HANDLE_CONNECTION, SEND EVERYONE INTO THEIR OWN FILES


## Major features:
Display image on the post page too

later: Reply to individual comments.

## Safety/Bugs:

!! GET form appends ? to edit

Devtool to easily change account status, quickly, for testing.

Way later: Migrations — schema changes currently require deleting the database.



## Frontend polish:

Beautiful error screens. Plus for rate limiting no redirect, just render on page



## PERFORMANCE:

Templates re-read from disk on every render, optimise

Implement connection pooling instead of mutex locking (make sure to record before and after time of that)




