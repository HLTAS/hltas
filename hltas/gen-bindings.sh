#!/bin/sh

exec bindgen ../src/hltas.hpp \
	-o src/hltas_cpp.rs \
	--whitelist-function 'hltas_.*' \
	--whitelist-type 'HLTAS::ErrorDescription' \
	--rustified-enum 'HLTAS::StrafeType|StrafeDir|ButtonState|Button|ErrorCode' \
	--disable-name-namespacing \
	-- -std=c++14 -I/usr/bin/../lib64/gcc/x86_64-pc-linux-gnu/9.2.0/include
