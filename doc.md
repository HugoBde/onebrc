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

# Tackling core::str::converts::from_utf8 PART 1
We notice that we don't really need to interpret all the bytes as character. utf 8 parsing is probably very expensive due to the checks needed. We partially get rid of utf8 parsing by swapping from f.lines to f.split(b'\n'). We change our hashmap key to be a vec of bytes. Do note we still do utf 8 parsing for the temperature, but it's tightly coupled with the last big culprit, parsing the string into f64. let's see if we can tackle both at the same time

we maintain the use of split_once by switching to the nightly toolchain and enabling the newly added feature slice_split_once because the code for that feature looks very simple and hard to mess up so I trust it

time output:
real    95.58s
user    91.46s
sys     3.87s
cpu     99%

Remaining culprits:
+   75.22%    49.77%  onebrc           onebrc                               [.] onebrc::main
+   10.65%    10.65%  onebrc           onebrc                               [.] core::num::dec2flt::<impl core::str::traits::FromStr for f64>::from_str
+    7.92%     7.91%  onebrc           onebrc                               [.] core::str::converts::from_utf8 // still here but almost halved

