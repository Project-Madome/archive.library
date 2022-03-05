# beta, stable일 경우에만 릴리즈
# beta, stable 따로 관리할 수 있어야함

SVC=library

CURRENT_BRANCH="$(git branch --show-current)"
VERSION="$(cat Cargo.toml | grep 'version = ' | head -1 | sed -e 's/version = //' | sed -e 's/\"//g')"

if [ "$CURRENT_BRANCH" = "beta" ]; then
    TARGET="debug"
    VERSION="$VERSION-beta"

    cargo build --target=x86_64-unknown-linux-musl
elif [ "$CURRENT_BRANCH" = "stable" ]; then
    TARGET="release"

    cargo build --release --target=x86_64-unknown-linux-musl
else
    echo "can't release from master branch"
    exit 1
fi

if [ $? -ne 0 ]; then
    exit 1
fi

BIN=./target/x86_64-unknown-linux-musl/$TARGET/madome-$SVC

docker build --build-arg BINARY_FILE="$BIN" --tag "madome/$SVC:$VERSION" .

if [ $? -ne 0 ]; then
    exit 1
fi

docker push "madome/$SVC:$VERSION"
