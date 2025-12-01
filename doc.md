# Naive Implementation

Did a naive implementation with a hashmap, mapping station names to min, max, total and number of recordings 

Once reading is over, we iterate over each entry, compute the mean, collect it into a vec and then sort by station name. 

then we print everything

time output: 
real    123.57s
user    119.73s
sys     3.57s
cpu     99%

running perf on the small dataset,
we get these culprits:
+   56.51%    34.78%  onebrc           onebrc                onebrc::main
+   18.49%    17.23%  onebrc           onebrc                core::str::iter::SplitInternal<P>::next
+   11.33%    11.32%  onebrc           onebrc                core::str::converts::from_utf8
+   10.67%    10.67%  onebrc           onebrc                core::num::dec2flt::<impl core::str::traits::FromStr for f64>::from_str

We'll skip main for now because I'm challenged, not too sure why main isn't 100%, since time claims 99% cpu time.
let's tackle them one by one

# Tackling core::str::iter::SplitInternal<P>::next
We remove our use of split in the loop and instead use split_once. We can do that because we know each line has only one ';'. This avoid's us iterating over the remaining characters to look for other semi-colons for nothing.

time output:
real    107.86s
user    103.85s
sys     3.76s
cpu     99%

Remaining culprits:
+   71.44%    47.26%  onebrc           onebrc                onebrc::main
+   13.68%    13.67%  onebrc           onebrc                core::str::converts::from_utf8
+   11.89%    11.89%  onebrc           onebrc                core::num::dec2flt::<impl core::str::traits::FromStr for f64>::from_str
