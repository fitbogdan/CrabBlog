## Major features:
-> Upload pictures to the server from /write



## Safety/Bugs:

Devtool to easily change account status, quickly, for testing.

Way later: Migrations — schema changes currently require deleting the database.


## Frontend polish:

Two password fields to make sure they type the correct password

Password peak button

Translate comment time to readable text in the client's timezone if possible





## PERFORMANCE:

Templates re-read from disk on every render, optimise

Implement connection pooling instead of mutex locking (make sure to record before and after time of that)





