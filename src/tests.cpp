#define CATCH_CONFIG_MAIN
#include "catch.hpp"

TEST_CASE("Basic") {
	REQUIRE(1 == 1);
}

TEST_CASE("Basic not working") {
	REQUIRE(1 == 2);
}
