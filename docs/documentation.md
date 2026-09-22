# EXTREMELY IMPORTANT:

Comments, and usernames are NOT stored as XSS escaped, they are stored as given by the user. The function get_comments, querries the DB for them and escaped them right there, returning them escaped.

REMEMBER: For future use of anything inside users/comments, do xss escaping first before displaying the html.