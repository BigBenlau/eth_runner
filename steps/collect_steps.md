# Start On-CPU and Off-CPU Scanning at the same time
1.
./target/debug/rust_runner > rust.log & \
kill -STOP `pgrep -nx rust_runner`

## Off-CPU
2.1. offcputime-bpfcc -df -p `pgrep -nx rust_runner` > out.stacks  (-p process)
     offcputime-bpfcc -df -t `pgrep -nx rust_runner` > out.stacks  (-t thread)
        local: offcputime-bpfcc => /usr/share/bcc/tools/offcputime
     /usr/share/bcc/tools/offcputime -df -t `pgrep -nx rust_runner` 100 > out.stacks


## On-CPU
3. perf record -F 99 -g -p `pgrep -nx rust_runner` -o perf_on.data &

# CPU Time
4. perf stat -e task-clock -e cpu-clock -e cycles -e cpu-cycles -p `pgrep -nx rust_runner` -o stat_info.log &


5. kill -CONT `pgrep -nx rust_runner`


# Off-CPU
../FlameGraph/flamegraph.pl --color=io --title="Off-CPU Time Flame Graph" --countname=us < out.stacks > off_cpu.svg
../FlameGraph/flamegraph.pl --color=io --title="Off-CPU Time Icicle Graph" --countname=us --reverse --inverted < out.stacks > off_cpu_rev.svg

# On-CPU
perf script -i perf_on.data &> perf.unfold
../FlameGraph/stackcollapse-perf.pl perf.unfold &> perf.folded
../FlameGraph/flamegraph.pl perf.folded > perf.svg
../FlameGraph/flamegraph.pl perf.folded --reverse --inverted > perf_rev.svg







On-CPU flamegraph

Use root account, install perf
1. cd rust_runner; cargo build
2. perf record --call-graph=dwarf ./target/debug/rust_runner
3. perf script -i perf.data &> perf.unfold (need several hours or days)
4. git clone https://github.com/brendangregg/FlameGraph
5. FlameGraph/stackcollapse-perf.pl perf.unfold &> perf.folded
6. FlameGraph/flamegraph.pl perf.folded > perf.svg


# Off-CPU flamegraph
Use root account, install offcputime-bpfcc

cd /usr/share/bcc/tools
offcputime


1.1 ./target/debug/rust_runner & \
1.2 (immediately after 1.1)     offcputime-bpfcc -df -p `pgrep -nx rust_runner` > out.stacks  (-p process)
                                offcputime-bpfcc -df -t `pgrep -nx rust_runner` > out.stacks  (-t thread)
        local: offcputime-bpfcc => /usr/share/bcc/tools/offcputime \
2. ./flamegraph.pl --color=io --title="Off-CPU Time Flame Graph" --countname=us < out.stacks > out.svg




Memory flamegraph
Use root account, install heaptrack
1. heaptrack target/debug/rust_runner (output: heaptrack.rust_runner.3154.zst)
2. heaptrack_print heaptrack.rust_runner.3154.zst -F mem_1000.folded
3. FlameGraph/flamegraph.pl mem_1000.folded > mem_1000.svg



Reference:
1. https://rustmagazine.github.io/rust_magazine_2021/chapter_11/rust-profiling.html
2. https://www.cnblogs.com/conscience-remain/p/16142279.html
3. https://gist.github.com/HenningTimm/ab1e5c05867e9c528b38599693d70b35
4. https://www.aisoftcloud.cn/blog/article/1684117709501802?session=
5. https://blog.yanjingang.com/





## gdb
1. attach {PID}


2. thread apply all bt
show all thread record




# How do I logically turn off (offline) cpu#6 ?
Warning: It is not possible to disable CPU0 on Linux systems i.e do not try to take cpu0 offline. Some architectures may have some special dependency on a certain CPU. For e.g in IA64 platforms we have ability to sent platform interrupts to the OS. a.k.a Corrected Platform Error Interrupts (CPEI). In current ACPI specifications, we didn’t have a way to change the target CPU. Hence if the current ACPI version doesn’t support such re-direction, we disable that CPU by making it not-removable. In such cases you will also notice that the online file is missing under cpu0.

Type the following command at linux:
```
echo 0 > /sys/devices/system/cpu/cpu6/online
grep "processor" /proc/cpuinfo
```

# How do I logically turn on (online) cpu#6 ?
Type the following command at linux:
```
echo 1 > /sys/devices/system/cpu/cpu6/online
grep "processor" /proc/cpuinfo
```


