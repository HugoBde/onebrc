My attempts at the 1 Billion Rows Challenge.

Tracking my solutions here in hopes of producing a blog article and maybe a youtube video.

Results:

| Latest Improvement | commit | time |
|---|---|---|
| Naive Implementation | e7a25dc | 123.6s |
| Swap split(';') for split_once(';') | 78b1f0f | 107.9s |
| Dropping UTF-8 parsing (part 1) | b4a2efc | 95.6s |
| Dropping UTF-8 parsing (part 2) and Dropping f64 parsing | 591fa0b | 81.2s |
| Changing the hashing algorithm used in our hashmap | a1e1531 | 66.7s |
| Memory mapping the file instead of reading it normally | e626bb6 | 43.0s |
| < mystery improvement from using an input file with realistic values, ignore that > | N/A | 36s |
| Iterating from the back instead of the front when looking for the semi-colon separator | 957e37e | 32.9s |




