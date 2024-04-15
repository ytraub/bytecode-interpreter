cargo build --release
scp -C -r target/armv5te-unknown-linux-musleabi/release/runtime robot@ev3dev:/home/robot/mindfactory-code-test/runtime