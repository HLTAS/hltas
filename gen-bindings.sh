#!/bin/sh

exec bindgen cpp/src/hltas.hpp \
	-o hltas-cpp-bridge/src/hltas_cpp_raw.rs \
	--whitelist-function 'hltas_.*' \
	--whitelist-type 'HLTAS::ErrorDescription' \
	--rustified-enum 'HLTAS::StrafeType|StrafeDir|ButtonState|Button|ErrorCode|StrafingAlgorithm|ConstraintsType' \
	--disable-name-namespacing \
	-- -std=c++14 -I/usr/bin/../lib64/gcc/x86_64-pc-linux-gnu/9.2.0/include --target=i686-unknown-linux-gnu
