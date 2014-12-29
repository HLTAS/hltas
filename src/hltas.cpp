#include <algorithm>
#include <cassert>
#include <cstdlib>
#include <fstream>
#include <future>
#include <iostream>
#include <locale>
#include <string>
#include <utility>
#include <boost/algorithm/string/trim.hpp>
#include <boost/tokenizer.hpp>

#include "hltas.hpp"

namespace HLTAS
{
	static const std::string ErrorMessages[] =
	{
		"Failed to open the file.",
		"Failed to read the version.",
		"This version is not supported.",
		"Failed to read line.",
		"Save name is required.",
		"Failed parsing the frame data."
	};

	static auto SplitProperty(const std::string& line)
	{
		auto commentPos = line.find("//");
		auto propertyLine = line.substr(0, commentPos);
		boost::trim(propertyLine);

		// Find the first whitespace character.
		auto space = std::find_if(propertyLine.begin(), propertyLine.end(),
			std::function<bool (const std::string::value_type&)>(
				[](const std::string::value_type& c) -> bool {
					return std::isspace(c, std::locale());
				}
			));

		std::string property, value;
		std::move(propertyLine.begin(), space, std::back_inserter(property));
		if (space != propertyLine.end())
		{
			std::move(++space, propertyLine.end(), std::back_inserter(value));
			boost::trim_left(value);
		}

		return std::make_pair(property, value);
	}

	static unsigned ReadNumber(const char* str, std::size_t* pos)
	{
		unsigned ret = 0;
		if (!str)
			return ret;
		while (std::isdigit(*str)) {
			ret *= 10;
			ret += *str - '0';
			str++;
			if (pos)
				(*pos)++;
		}
		return ret;
	}

	void Input::Clear()
	{
		// If we're reading some file, wait for it to finish.
		if (FinishedReading.valid())
			FinishedReading.wait();

		Properties.clear();
		Frames.clear();
	}

	std::shared_future<ErrorDescription> Input::Open(const std::string& filename)
	{
		Clear();

		FinishedReading = std::async(&Input::OpenInternal, this, filename);
		return FinishedReading;
	}

	const std::string& Input::GetErrorMessage(ErrorDescription error)
	{
		assert(error.Code > 0);
		return ErrorMessages[error.Code - 1];
	}

	ErrorDescription Input::Error(ErrorCode code)
	{
		return ErrorDescription { code, CurrentLineNumber };
	}

	ErrorDescription Input::OpenInternal(const std::string& filename)
	{
		CurrentLineNumber = 1;
		std::ifstream file(filename);
		if (!file)
			return Error(FAILOPEN);

		// Read and check the version.
		std::string temp;
		std::getline(file, temp, ' ');
		if (file.fail() || temp != "version")
			return Error(FAILVER);
		std::getline(file, temp);
		if (file.fail() || temp.empty())
			return Error(FAILVER);
		try {
			Version = std::stoi(temp);
		} catch (...) {
			return Error(FAILVER);
		}
		if (Version <= 0)
			return Error(FAILVER);
		if (Version > MAX_SUPPORTED_VERSION)
			return Error(NOTSUPPORTED);

		try {
			ReadProperties(file);
			ReadFrames(file);
		} catch (ErrorCode error) {
			return Error(error);
		}

		return Error(OK);
	}

	void Input::ReadProperties(std::ifstream& file)
	{
		while (file.good()) {
			CurrentLineNumber++;

			std::string line;
			std::getline(file, line);
			if (file.fail())
				throw FAILLINE;

			auto prop = SplitProperty(line);
			if (prop.first.empty())
				continue;
			if (prop.first == "frames")
				return;

			Properties[prop.first] = prop.second;
		}
	}

	void Input::ReadFrames(std::ifstream& file)
	{
		std::string commentString;
		bool firstFrameOfStrafing = false; // For viewangles checking.
		bool strafing = false; // For viewangles checking.
		while (file.good()) {
			CurrentLineNumber++;

			std::string line;
			std::getline(file, line);
			if (file.fail())
				throw FAILLINE;
			if (line.empty())
				continue;

			// TODO: Profile and check if std::move is faster.
			if (!line.compare(0, 2, "//")) {
				commentString += line.substr(2, std::string::npos) + '\n';
				continue;
			}
			if (!line.compare(0, 5, "save ")) {
				if (line.length() < 6)
					throw NOSAVENAME;
				Frame f = {};
				f.Comments = commentString;
				f.SaveName = line.substr(5, std::string::npos);
				Frames.push_back(f);
				commentString.clear();
				continue;
			}

			Frame f = {};
			unsigned fieldCounter = 0;
			boost::tokenizer< boost::char_separator<char> > tok(line, boost::char_separator<char>("|"));
			for (auto it = tok.begin(); it != tok.end(); ++it, ++fieldCounter) {
				auto str(std::move(*it));
				boost::trim(str);
				auto l = str.length();
				switch (fieldCounter) {
				case 0:
				{
					if (l < 10)
						throw FAILFRAME;

					if (str[0] == 's' && std::isdigit(str[1]) && std::isdigit(str[2])) {
						f.Strafe = true;
						f.Type = static_cast<StrafeType>(str[1] - '0');
						f.Dir = static_cast<StrafeDir>(str[2] - '0');
						if (!strafing)
							firstFrameOfStrafing = true;
						else
							firstFrameOfStrafing = false;
						strafing = true;
					} else if (str[0] != '-' || str[1] != '-' || str[2] != '-')
						throw FAILFRAME;
					if (!f.Strafe) {
						strafing = false;
						firstFrameOfStrafing = false;
					}

					std::size_t pos = 3;
					if (str[pos] == 'l' || str[pos] == 'L') {
						f.Lgagst = true;
						f.LgagstFullMaxspeed = (str[pos] == 'L');
						f.LgagstTimes = ReadNumber(str.c_str() + pos + 1, &pos);
					} else if (str[pos] != '-')
						throw FAILFRAME;

					#define READ(c, field) \
						pos++; \
						if (l <= pos) \
							throw FAILFRAME; \
						if (str[pos] == c) { \
							f.field = true; \
							f.field##Times = ReadNumber(str.c_str() + pos + 1, &pos); \
						} else if (str[pos] != '-') \
							throw FAILFRAME;

					READ('j', Autojump)
					READ('d', Ducktap)
					READ('b', Jumpbug)

					pos++;
					if (l <= pos)
						throw FAILFRAME;
					if (str[pos] == 'c' || str[pos] == 'C') {
						f.Dbc = true;
						f.DbcCeilings = (str[pos] == 'C');
						f.DbcTimes = ReadNumber(str.c_str() + pos + 1, &pos);
					} else if (str[pos] != '-')
						throw FAILFRAME;

					READ('g', Dbg)
					READ('w', Dwj)

					#undef READ
				}
					break;

				case 1:
				{
					if (l != 6)
						throw FAILFRAME;

					std::size_t pos = 0;
					#define READ(c, field) \
						if (str[pos] == c) \
							f.field = true; \
						else if (str[pos] != '-') \
							throw FAILFRAME; \
						pos++;

					READ('f', Forward)
					READ('l', Left)
					READ('r', Right)
					READ('b', Back)
					READ('u', Up)
					READ('d', Down)

					#undef READ
				}
					break;

				case 2:
				{
					if (l != 6)
						throw FAILFRAME;

					std::size_t pos = 0;
					#define READ(c, field) \
						if (str[pos] == c) \
							f.field = true; \
						else if (str[pos] != '-') \
							throw FAILFRAME; \
						pos++;

					READ('j', Jump)
					READ('d', Duck)
					READ('u', Use)
					READ('1', Attack1)
					READ('2', Attack2)
					READ('r', Reload)

					#undef READ
				}
					break;

				case 3:
				{
					if (l == 0)
						throw FAILFRAME;

					f.Frametime = std::strtof(str.c_str(), nullptr);
				}
					break;

				case 4:
				{
					if (l == 0)
						throw FAILFRAME;

					if (str[0] == '-') {
						if (firstFrameOfStrafing)
							throw FAILFRAME;
						else
							break;
					}

					f.YawPresent = true;
					auto s = str.c_str();
					if (f.Dir == StrafeDir::POINT) {
						char *s2;
						f.X = std::strtod(s, &s2);
						f.Y = std::strtod(s2, nullptr);
					} else {
						f.Yaw = std::atof(s);
					}
				}
					break;

				case 5:
				{
					if (l == 0)
						throw FAILFRAME;

					if (str[0] == '-')
						break;

					f.PitchPresent = true;
					f.Pitch = std::atof(str.c_str());
				}
					break;

				case 6:
				{
					if (l == 0)
						throw FAILFRAME;

					f.Frames = ReadNumber(str.c_str(), nullptr);
				}
					break;
				}
			}

			if (f.Frames == 0)
				f.Frames = 1;

			if (fieldCounter >= 7) {
				int sep = 0;
				std::size_t pos = 0;
				for (; sep != 7; ++pos)
					if (line[pos] == '|')
						sep++;

				f.Commands = line.substr(pos, std::string::npos);
			}

			std::move(commentString.begin(), commentString.end(), std::back_inserter(f.Comments));
			Frames.push_back(f);
			commentString.clear();
		}
	}

	int Input::GetVersion()
	{
		if (FinishedReading.valid())
			FinishedReading.wait();

		return Version;
	}

	std::unordered_map<std::string, std::string>& Input::GetProperties()
	{
		if (FinishedReading.valid())
			FinishedReading.wait();

		return Properties;
	}

	std::vector<Frame>& Input::GetFrames()
	{
		if (FinishedReading.valid())
			FinishedReading.wait();

		return Frames;
	}
}
