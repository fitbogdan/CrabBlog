## Major features:
Edit posts, at least the text, bonus the image

Store emails

Way later: Edit comment (For user and for admin)



## Safety/Bugs:

!! Reading multipart assumes image is the first part, thus if you send a post with no image it results in a bad request because it cannot find the image. !!


!! GET form appends ? to edit


Devtool to easily change account status, quickly, for testing.

Way later: Migrations — schema changes currently require deleting the database.

Control formatting of usernames and of passwords

Rate limiting on sending comments



## Frontend polish:

Two password fields to make sure they type the correct password

Password peak button

Translate comment time to readable text in the client's timezone if possible

If post has no photo just display one of those good looking squares with a color and a letter on it or something





## PERFORMANCE:

EASY ONE: Instead of doing request = String::from_utf8_lossy(&request_bytes) on every request, (Including multipart requests where its redundant to turn the body into a utf8 string), split the request bytes into:
    Header, Body.

    Later each function can take their own desired parts and decode them



Templates re-read from disk on every render, optimise

Implement connection pooling instead of mutex locking (make sure to record before and after time of that)





