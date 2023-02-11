#!/bin/sh
set -ex

# On F35 you need to install: clang-libs clang mingw32-gcc-c++ mingw64-gcc-c++

run_bindgen() {
	TARGET="$1"
	shift
	bindgen cpp/src/hltas.hpp \
		-o hltas-cpp-bridge/src/"$TARGET".rs \
		--allowlist-function 'hltas_.*' \
		--allowlist-type 'HLTAS::ErrorDescription' \
		--rustified-enum 'HLTAS::(StrafeType|StrafeDir|ButtonState|Button|ErrorCode|StrafingAlgorithm|ConstraintsType|ChangeTarget|LookAtAction)' \
		--disable-name-namespacing \
		-- -std=c++14 --target="$TARGET" "$@"
}

run_bindgen x86_64-unknown-linux-gnu
run_bindgen i686-unknown-linux-gnu
run_bindgen x86_64-pc-windows-gnu --sysroot=/usr/x86_64-w64-mingw32/sys-root/mingw
run_bindgen i686-pc-windows-gnu --sysroot=/usr/i686-w64-mingw32/sys-root/mingw