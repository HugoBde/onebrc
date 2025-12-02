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
+   87.34%    34.35%  onebrc           onebrc                onebrc::main
+   19.72%    18.22%  onebrc           onebrc                core::str::iter::SplitInternal<P>::next
+   11.04%    11.03%  onebrc           onebrc                core::str::converts::from_utf8
+   10.31%    10.31%  onebrc           onebrc                core::num::dec2flt::<impl core::str::traits::FromStr for f64>::from_str

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
+   86.60%    46.77%  onebrc           onebrc                onebrc::main
+   13.19%    13.19%  onebrc           onebrc                core::num::dec2flt::<impl core::str::traits::FromStr for f64>::from_str
+   11.98%    11.98%  onebrc           onebrc                core::str::converts::from_utf8

# Tackling core::str::converts::from_utf8 PART 1
We notice that we don't really need to interpret all the bytes as character. utf 8 parsing is probably very expensive due to the checks needed. We partially get rid of utf8 parsing by swapping from f.lines to f.split(b'\n'). We change our hashmap key to be a vec of bytes. Do note we still do utf 8 parsing for the temperature, but it's tightly coupled with the last big culprit, parsing the string into f64. let's see if we can tackle both at the same time

we maintain the use of split_once by switching to the nightly toolchain and enabling the newly added feature slice_split_once because the code for that feature looks very simple and hard to mess up so I trust it

time output:
real    95.58s
user    91.46s
sys     3.87s
cpu     99%

Remaining culprits:
+   91.31%    49.76%  onebrc           onebrc                onebrc::main
+   10.85%    10.84%  onebrc           onebrc                core::num::dec2flt::<impl core::str::traits::FromStr for f64>::from_str
+    7.61%     7.61%  onebrc           onebrc                core::str::converts::from_utf8                                                                             ▒
+    7.56%     7.56%  onebrc           onebrc                <std::hash::random::DefaultHasher as core::hash::Hasher>::write                                            ▒
+    6.33%     6.33%  onebrc           libc.so.6             0x000000000016cb99

# Tackling core::str::converts::from_utf8 PART 2 and tackling core::num::dec2flt::<impl core::str::traits::FromStr for f64>::from_str
The last remaining bit of utf8 parsing is when we need to parse the temperature reading. it's also where our other big culprit is. Can we drop both parsing by taking advantage of the guaranteed format: -?\d\d?.\d
Turns out we can, so we do that. 

time output:
real    81.19s
user    76.87s
sys     4.07s
cpu     99%

Remaining culprits:
+   99.14%    61.27%  onebrc           onebrc                onebrc::main
+    8.96%     8.95%  onebrc           onebrc                <std::hash::random::DefaultHasher as core::hash::Hasher>::write
+    7.29%     7.29%  onebrc           libc.so.6             0x000000000016cb99
+    6.46%     6.46%  onebrc           libc.so.6             malloc

# Tackling <std::hash::random::DefaultHasher as core::hash::Hasher>::write
Tackling the first few culprits has dropped the runtime from 120 seconds to 80 seconds, that's a decent 1/3 down. Doing so has revealed some new culprits to target, the first of which is the hashing used with the HashMap. The algorithm used in the std lib HashMap (SipHash 1-3) is somewhat cryptographically secure (it aims at preventing HashDoS attacks). Switching for the FxHash algorithm (used by rustc itself) gives us our next speed up, as we don't need the security provided by SipHash and would much rather process our 1 billion rows asap

time output:
real    66.71s
user    62.78s
sys     3.62s
cpu     99%

# mmap'ing the file
Ok, at this point, most actual expensive functions have been eliminated. Now let's look at functions who are expensive with their children. 
The first one we can easily tackle is the whole BufReader reading calls:
+   30.58%     0.00%  onebrc   onebrc                std::io::BufRead::read_until (inlined)
+   30.58%     0.00%  onebrc   onebrc                std::io::read_until (inlined)
Let's get rid of read calls and instead mmap the file into RAM cache and ask the OS to let us directly address this area of memory, instead of copying it to ours repeatedly

time output:
real    42.97s
user    40.60s
sys     2.12s
cpu     99%

Another optimisation mmap'ing does is it means that the lifetime of the station name is the lifetime of the mmap (basically the whole program) as opposed to the lifetime of the line it was on. That means we can switch our keys from owned vecs of bytes to references to the mmap'ed memory region instead, getting rid of lots of potential copying. jk, it doesn't change much because we only do a copy when we encounter a new station, which rarely ever happens, but it still feels really nice to get rid of that copy so I'll keep it. the slight time decrease is more likely noise than actual improvement from this change.

time output:
real    41.87s
user    39.51s
sys     2.14s
cpu     99%

# The next day
I changed the measurements data to use one that was generated with the Java generator instead of the Python one, that shaved another ~10%. This is due to reduced collisions in the HashMap I believe, since we now spend less time in HashMap::get_mut

time output:
real    36.02s
user    31.65s
sys     4.08s
cpu     99%

# Splitting is being expensive again
As we improve various areas of the code, our earlier improvements start becoming a more important part of our run time again. The first case we can easily tackle is splitting:
+   16.13%     0.00%  onebrc  onebrc  core::slice::<impl [T]>::split_once (inlined)
And a easy way to improve that is by looking at a line in our data:
Andorra la Vella;7.5
And noticing that the semi-colon is closer to the end that the beginning (at least like 99.999% of the time). Why iterate forward when we can start from the end of the line? Let's replace split_once for rsplit_once and shave another 10% of our runtime.

time output:
real    32.92s
user    29.43s
sys     2.71s
cpu     97%

We can also notice that temperatures are guaranteed to be 3 to 5 characters long: (i.e: 1.2, -12.2). That means the semi colon is always 4 to 6 characters from the end. Let's keep that in mind as we may use this fact down the line to do some further low level optimisation!!

 
# Redoing our parsing
+   33.38%     0.00%  onebrc  onebrc  [.] onebrc::i64_parser_old (inlined)
After the latest improvements, we notice our temperature parsing code is being quite expensive again, let's dive deeper to see where we can improve it
The code makes use of lots of branching instructions (loops and if statements). These are generally not great for performance as they prevent  instruction pipelining, a mechanism CPUs use to run faster. We rewrote our parsing code to get rid of the for loops, always skip the decimal point without using an if statement, and mashing everything together in a match. Still a branch in a very tight loop, but at least we only have one jump happening per temperature parsing. Our new function is now running about 3 times faster.

+   11.58%     0.00%  onebrc  onebrc  onebrc::i64_parser_new (inlined)

NB: When developing my new parsing code, I ran benchmarks on the isolated parsing function. 

The old version gave me those results:

test parser_bench_negative_double_digits ... bench:           2.79 ns/iter (+/- 0.57)
test parser_bench_negative_single_digits ... bench:           1.86 ns/iter (+/- 0.18)
test parser_bench_positive_double_digits ... bench:           3.34 ns/iter (+/- 0.44)
test parser_bench_positive_single_digits ... bench:           2.29 ns/iter (+/- 0.33)

While the new version gave me those results:

test parser_bench_negative_double_digits ... bench:           0.23 ns/iter (+/- 0.02)
test parser_bench_negative_single_digits ... bench:           0.23 ns/iter (+/- 0.02)
test parser_bench_positive_double_digits ... bench:           0.24 ns/iter (+/- 0.01)
test parser_bench_positive_single_digits ... bench:           0.23 ns/iter (+/- 0.04)

That's about a 10x speedup as opposed to the 3x we see in our code. My hypothesis at this point is that the benchmarks run each individual case repeatedly, which (rightfully) tricks the branch predictor (another mechanism the CPU uses to maintain instruction pipelining despite jmp instructions) into pipelining the instructions of the same branch each iteration. In practice, this doesn't work because we keep hitting different branches


time output: 
real    29.38s
user    27.35s
sys     1.82s
cpu     99%
