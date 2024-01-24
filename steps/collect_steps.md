
On-CPU flamegraph

Use root account, install perf
1. cd rust_runner; cargo build
2. perf record --call-graph=dwarf ./target/debug/rust_runner
3. perf script -i perf.data &> perf.unfold (need several hours or days)
4. git clone https://github.com/brendangregg/FlameGraph
5. FlameGraph/stackcollapse-perf.pl perf.unfold &> perf.folded
6. FlameGraph/flamegraph.pl perf.folded > perf.svg


Off-CPU flamegraph
Use root account, install offcputime-bpfcc

cd /usr/share/bcc/tools
offcputime

1.
1.1 ./target/debug/rust_runner &
1.2 (immediately after 1.1) offcputime-bpfcc -df -p `pgrep -nx rust_runner` 100 > out.stacks
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