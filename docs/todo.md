## Major features:
Admin accounts + Admin Console





## Frontend polish:

Two password fields to make sure they type the correct password

Password peak button

Translate comment time to readable text in the client's timezone if possible





## PERFORMANCE:

Templates re-read from disk on every render, optimise

Implement connection pooling instead of mutex locking (make sure to record before and after time of that)






## Safety/Bugs:

    Way later: Migrations — schema changes currently require deleting the database.