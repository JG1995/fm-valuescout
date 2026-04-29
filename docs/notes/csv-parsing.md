# CSV Parsing

In order for this tool to function, the parsing of the CSV file needs to be flawless. Therefore, I will explain each column in the CSV so we can all understand what parsing needs to be done, and which units of measurement each column has. The format will be "[csv header] [(Full Metric Name)]: [description and info]".

1. Unique ID (UID): Unique ID from in-game, use to generate own in-app unique ID.
2. Player (Player): The player's name.
3. Nation (Nationality): A 3-letter code representing the players primary nationality. Should be mapped against the nation the 3-letter code actually represents.
4. 2nd Nat (2nd Nationality): A player's 2nd nationality - For some reason, this is the nation's name written out, not a 3-letter code.
5. Club (Club): A player's club name.
6. Position (Positions): A list of the positions a player can play - Can be stored as a text string, but should be parsed to an array or list of positions.
7. Age (Age): A players age.
8. Height (Height): Integer - Includes unit "cm".
9. Left Foot (Left): String - Might want to convert to a "score".
10. Right Foot (Right): String - Might want to convert to a "score".
11. CA (Current Ability): A column that may be completely absent from the CSV - This is intentional, and the app should gracefully handle it if it is not, but save it if it is.
12. PA (Potential Ability): A column that may be completely absent from the CSV - This is intentional, and the app should gracefully handle it if it is not, but save it if it is.
13. Transfer Value (Transfer Value): A string containing a range, for example, `€160M - €210M`. Save only the highest of these values. Notice that some values may be in millions (M), and some may be in thousands (K) - And some may just be the number outright. The currency may differ from what is in the sample CSV's - Use what the user provides in their CSV.
14. Wage (Wage): A monetary value appended with either p/w (per week), p/m (per month) or p/a (per annum/year). We should save the denomination as it will affect calculations down the line. Notice that some values may be in millions (M), and some may be in thousands (K) - And some may just be the number outright. The currency may differ from what is in the sample CSV's - Use what the user provides in their CSV.
15. Expires (Contract Expiry Date): The contract expiry date - Compare the date listed against the in-game date provided by the user.
16. Appearances (Appearances): The appearances of a player during the season. The first number is the number of games the player started, the number in parens is the number of games the player came on as a substitute. Should probably save both values.
17. Minutes (Minutes Played): Minutes played throughout the season, just a number.
18. Goals (Goals): Integer.
19. Goals From Outside The Box (Goals From Outside The Box): Integer.
20. xG (Expected Goals): Store to 2 decimal places.
21. NP-xG (Non-Penalty Expected Goals): xG but excluding penalties - Store to 2 decimal places.
22. xG-OP (Expected Goals Overperformance): A number stored to 2 decimals, but can be a negative value.
23. xG/Shot (Expected Goals per Shot): Number, store to 2 decimal places.
24. Shots (Shots): Integer.
25. Shots From Outside The Box per 90 (Shots From Outside The Box per 90): Number - Store to 2 decimal places.
26. ShT (Shots on Target): Integer.
27. Pens (Penalties Taken): Integer.
28. Pens S (Penalties Scored): Integer.
29. Free Kick Shots (Free Kick Shots): Integer.
30. Assists (Assists): Integer.
31. xA (Expected Assists): Number to 2 decimal places.
32. Ch C/90 (Chances Created per 90): Number to 1 decimal place.
33. CCC (Clear Cut Chances): Integer.
34. Key (Key Passes): Integer.
35. OP-KP/90 (Open Play Key Passes per 90): Number to 1 decimal place.
36. Cr A (Crosses Attempted): Integer.
37. Cr C (Crosses Completed): Integer.
38. OP-Crs A (Open Play Crosses Attempted): Integer.
39. OP-Crs C (Open Play Crosses Completed): Integer.
40. Pas A (Passes Attempted): Integer.
41. Pas C (Passes Completed): Integer.
42. Psp (Progressive Passes): Integer.
43. Drb (Dribbles Made): Integer.
44. Distance (Distance Covered): Total distance covered - Includes unit "km". Save to 1 decimal place.
45. Sprints/90 (High Intensity Sprints per 90): Number to 1 decimal place.
46. Poss Lost/90 (Possession Lost per 90): Number to 1 decimal place.
47. Tck A (Tackles Attempted): Integer.
48. Tck C (Tackles Completed): Integer.
49. K Tck (Key Tackles): Integer.
50. Itc (Interceptions): Integer.
51. Poss Won/90 (Possession Won per 90): Number to 1 decimal place.
52. Pres A (Pressures Attempted): Integer.
53. Pres C (Pressures Completed): Integer.
54. Blk (Blocks): Integer.
55. Shts Blckd (Shots Blocked): Integer.
56. Clearances (Clearances): Integer.
57. Hdrs A (Aerial Challenges Made): Integer.
58. Hdrs (Aerial Challenges Won): Integer.
59. Hdrs L/90 (Aerial Challenges Lost per 90): Number to 1 decimal point.
60. K Hdrs/90 (Key Headers per 90): Number to 1 decimal point.
61. Clean Sheets (Clean Sheets): Integer.
62. Goals Conceded (Goals Conceded): Integer.
63. Saves/90 (Saves per 90): Number to 1 decimal point.
64. xSv % (Expected Save Percentage): Integer
65. xGP (Expected Goals Prevented): Number to 2 decimal places - Can be negative.
66. Svh (Saves Held): Integer.
67. Svp (Saves Parried): Integer.
68. Svt (Saves Tipped): Integer.
69. Pens Faced (Penalties Faced): Integer.
70. Pens Saved (Penalties Saved): Integer.
71. Fouls Made (Fouls Made): Integer.
72. Fouls Against (Fouls Against): Integer.
73. Yel (Yellow Cards): Integer.
74. Red cards (Red Cards): Integer.
75. Off (Offsides): Integer.
76. MLG (Mistakes Leading To Goal): Integer.
77. Rating (Average Rating): Number to 2 decimal places.
78. PoM (Player of the Match): Integer.
79. Games Won (Games Won): Integer.
80. Games Drawn (Games Drawn): Integer.
81. Games Lost (Games Lost): Integer.
82. Team Goals (Team Goals): Integer.
