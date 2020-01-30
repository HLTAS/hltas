#define CATCH_CONFIG_MAIN
#include "catch.hpp"

#include <array>
#include <string>
#include <utility>

#include "hltas.hpp"

const std::array<const char*, 19> parse_files = {
	"bhop_20fps.hltas",
	"bhop.hltas",
	"bkz_goldbhop.hltas",
	"blolly.hltas",
	"change.hltas",
	"cs_estate.hltas",
	"destructo-hops.hltas",
	"extra-letters.hltas",
	"goldbhop.hltas",
	"halflife.hltas",
	"kz_synergy_x.hltas",
	"mirror.hltas",
	"nuker.hltas",
	"rng.hltas",
	"tas-hazard course-1.32,669.hltas",
	"tas-kz_summercliff2-1.14.060.hltas",
	"triggertest.hltas",
	"tripminetest.hltas",
	"vectorial.hltas"
};

TEST_CASE("Parse") {
	HLTAS::Input input;

	for (const auto file : parse_files) {
		SECTION(file) {
			const auto path = std::string("../test-data/parse/") + file;
			const auto err = input.Open(path);
			CHECK(err.Code == HLTAS::ErrorCode::OK);
		}
	}
}

TEST_CASE("Parse, write, parse") {
	HLTAS::Input input;

	for (const auto file : parse_files) {
		SECTION(file) {
			auto path = std::string("../test-data/parse/") + file;
			REQUIRE(input.Open(path).Code == HLTAS::ErrorCode::OK);
			path = std::string("../test-data/write-output/") + file;
			REQUIRE(input.Save(path).Code == HLTAS::ErrorCode::OK);
			REQUIRE(input.Open(path).Code == HLTAS::ErrorCode::OK);
		}
	}
}

TEST_CASE("Error") {
	const std::array<std::pair<const char*, HLTAS::ErrorCode>, 13> files = {
		std::make_pair("does-not-exist.hltas", HLTAS::ErrorCode::FAILOPEN),
		std::make_pair("no-version.hltas", HLTAS::ErrorCode::FAILVER),
		std::make_pair("too-high-version.hltas", HLTAS::ErrorCode::NOTSUPPORTED),
		std::make_pair("no-save-name.hltas", HLTAS::ErrorCode::NOSAVENAME),
		std::make_pair("too-few-dashes-field-0.hltas", HLTAS::ErrorCode::FAILFRAME),
		std::make_pair("no-seed.hltas", HLTAS::ErrorCode::NOSEED),
		std::make_pair("no-yaw.hltas", HLTAS::ErrorCode::NOYAW),
		std::make_pair("no-buttons.hltas", HLTAS::ErrorCode::NOBUTTONS),
		std::make_pair("both-j-d.hltas", HLTAS::ErrorCode::BOTHAJDT),
		std::make_pair("no-lgagst-action.hltas", HLTAS::ErrorCode::NOLGAGSTACTION),
		std::make_pair("no-lgagst-min-speed.hltas", HLTAS::ErrorCode::NOLGAGSTMINSPEED),
		std::make_pair("lgagst-action-times.hltas", HLTAS::ErrorCode::LGAGSTACTIONTIMES),
		std::make_pair("no-reset-seed.hltas", HLTAS::ErrorCode::NORESETSEED),
	};

	HLTAS::Input input;

	for (const auto test : files) {
		const auto file = test.first;
		const auto code = test.second;

		SECTION(file) {
			const auto path = std::string("../test-data/error/") + file;
			const auto err = input.Open(path);
			CHECK(err.Code == code);
		}
	}
}

void validate(const HLTAS::Input& input) {
	CHECK(input.GetVersion() == 1);

	const auto& properties = input.GetProperties();
	CHECK(properties.size() == 3);
	CHECK(properties.at("demo") == "bhop");
	CHECK(properties.at("frametime0ms") == "0.0000001");
	CHECK(properties.at("hlstrafe_version") == "1");

	const auto& frames = input.GetFrames();
	REQUIRE(frames.size() == 7);

	SECTION("Frame 0") {
		const auto& frame = frames[0];
		CHECK(frame.Frametime == "0.001");
		CHECK(frame.GetRepeats() == 1);
		CHECK(frame.Commands == "sensitivity 0;bxt_timer_reset;bxt_taslog");
	}
	SECTION("Frame 1") {
		const auto& frame = frames[1];
		CHECK(frame.Frametime == "0.001");
		CHECK(frame.GetRepeats() == 5);
	}
	SECTION("Frame 2") {
		const auto& frame = frames[2];
		CHECK(frame.GetType() == HLTAS::StrafeType::MAXACCEL);
		CHECK(frame.GetDir() == HLTAS::StrafeDir::YAW);
		CHECK(frame.GetYaw() == 170);
		CHECK(frame.GetPitch() == 0);
		CHECK(frame.Frametime == "0.001");
		CHECK(frame.GetRepeats() == 400);
	}
	SECTION("Frame 3") {
		const auto& frame = frames[3];
		CHECK(frame.Frametime == "0.001");
		CHECK(frame.GetRepeats() == 2951);
	}
	SECTION("Frame 4") {
		const auto& frame = frames[4];
		CHECK(frame.GetType() == HLTAS::StrafeType::MAXACCEL);
		CHECK(frame.GetDir() == HLTAS::StrafeDir::YAW);
		CHECK(frame.GetYaw() == 90);
		CHECK(frame.Frametime == "0.001");
		CHECK(frame.GetRepeats() == 1);
		CHECK(frame.Commands == "bxt_timer_start");
	}
	SECTION("Frame 5") {
		const auto& frame = frames[5];
		CHECK(frame.GetType() == HLTAS::StrafeType::MAXACCEL);
		CHECK(frame.GetDir() == HLTAS::StrafeDir::YAW);
		CHECK(frame.Lgagst == true);
		CHECK(frame.GetDucktap0ms() == true);
		CHECK(frame.GetYaw() == 90);
		CHECK(frame.Frametime == "0.001");
		CHECK(frame.GetRepeats() == 5315);
		CHECK(frame.Comments == " More frames because some of them get converted to 0ms\n");
	}
	SECTION("Frame 6") {
		const auto& frame = frames[6];
		CHECK(frame.Frametime == "0.001");
		CHECK(frame.GetRepeats() == 1);
		CHECK(frame.Commands == "stop;bxt_timer_stop;pause;sensitivity 1;_bxt_taslog 0;bxt_taslog;//condebug");
	}
}

TEST_CASE("Parse and validate") {
	HLTAS::Input input;
	REQUIRE(input.Open("../test-data/parse/bhop.hltas").Code == HLTAS::ErrorCode::OK);

	validate(input);
}

TEST_CASE("Parse, write, parse and validate") {
	HLTAS::Input input;
	REQUIRE(input.Open("../test-data/parse/bhop.hltas").Code == HLTAS::ErrorCode::OK);
	REQUIRE(input.Save("../test-data/write-output/bhop.hltas").Code == HLTAS::ErrorCode::OK);
	REQUIRE(input.Open("../test-data/write-output/bhop.hltas").Code == HLTAS::ErrorCode::OK);

	validate(input);
}
