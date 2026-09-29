MASSIVE:
Implement a deadline to the read_bytes function, to limit the use slow loris on my server.

After 10 seconds if the request didn't complete, return.

The real long term fix is switching to async though.





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




