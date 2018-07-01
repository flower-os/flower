def archiveFlower() {
    if (BRANCH_NAME == "master" || BRANCH_NAME == "development") {
        echo "Archiving build artifacts"
        archiveArtifacts artifacts: "build/release/kernel.elf"
        archiveArtifacts artifacts: "build/release/flower.iso"
    } else {
        echo "Skipping build artifact archiving"
    }
}

pipeline {
    agent any
    options {
        timeout(time: 10, unit: "MINUTES") 
    }
    stages {
        stage("Build") {
            steps {
                sh """export PATH="/home/gegy1000/.cargo/bin:$PATH"
                    |export RUST_BACKTRACE=1
                    |export RUST_TARGET_PATH=\$(pwd)/kernel
                    |rustup override add nightly
                    |rustup component add rust-src
                    |make clean
                    |make iso""".stripMargin()
            }
            post {
                success {
                    archiveFlower()
                }
            }
        }
    }
}
