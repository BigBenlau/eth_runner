./target/debug/rust_runner > rust.log & \
kill -STOP `pgrep -nx rust_runner`
/usr/share/bcc/tools/offcputime -df -t `pgrep -nx rust_runner` 100 > out.stacks &
perf stat -e task-clock -e cpu-clock -e cycles -e cpu-cycles -p `pgrep -nx rust_runner` -o stat_info.log &
perf record -F 99 -g -p `pgrep -nx rust_runner` -o perf_on.data &
kill -CONT `pgrep -nx rust_runner`
echo `date`
