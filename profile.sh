cargo test --release --no-run 

sudo dtrace -c './target/release/thread_pool' -o out.stacks -n 'profile-997 /execname == "thread_pool"/ { @[ustack(100)] = count(); }'

# run only tests, only a single one
sudo dtrace -c './target/release/deps/thread_pool-824394ceab69d34e test_submit' -o out.stacks -n 'profile-997 /execname == "thread_pool-824394ceab69d34e"/ { @[ustack(100)] = count(); }'


../../FlameGraph/stackcollapse.pl  out.stacks | ../../FlameGraph/flamegraph.pl > graphs/pretty-graph.svg