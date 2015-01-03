#include <algorithm>
#include <cassert>
#include <cstdlib>
#include <ctime>
#include <fstream>
#include <future>
#include <iostream>
#include <locale>
#include <string>
#include <sstream>
#include <utility>
#include <boost/algorithm/string/trim.hpp>
#include <boost/format.hpp>
#include <boost/thread/lock_types.hpp>
#include <boost/thread/shared_mutex.hpp>
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
		"Failed parsing the frame data.",
		"Failed to write data to the file.",
		"Seeds are required.",
		"The yaw field needs a value on this frame."
	};

	const std::string& GetErrorMessage(ErrorDescription error)
	{
		assert(error.Code > 0);
		return ErrorMessages[error.Code - 1];
	}

	void Frame::SetType(StrafeType value)
	{
		Strafe = true;
		Type = value;
	}

	void Frame::SetDir(StrafeDir value)
	{
		Strafe = true;
		Dir = value;
	}

	void Frame::SetLgagstFullMaxspeed(bool value)
	{
		Lgagst = true;
		LgagstFullMaxspeed = value;
	}

	void Frame::SetLgagstTimes(unsigned value)
	{
		Lgagst = true;
		LgagstTimes = value;
	}

	void Frame::SetAutojumpTimes(unsigned value)
	{
		Autojump = true;
		AutojumpTimes = value;
	}

	void Frame::SetDucktapTimes(unsigned value)
	{
		Ducktap = true;
		DucktapTimes = value;
	}

	void Frame::SetJumpbugTimes(unsigned value)
	{
		Jumpbug = true;
		JumpbugTimes = value;
	}

	void Frame::SetDbcCeilings(bool value)
	{
		Dbc = true;
		DbcCeilings = value;
	}

	void Frame::SetDbcTimes(unsigned value)
	{
		Dbc = true;
		DbcTimes = value;
	}

	void Frame::SetDbgTimes(unsigned value)
	{
		Dbg = true;
		DbgTimes = value;
	}

	void Frame::SetDwjTimes(unsigned value)
	{
		Dwj = true;
		DwjTimes = value;
	}

	double Frame::GetYaw() const
	{
		assert(YawPresent);
		assert(!Strafe || (Dir != StrafeDir::LEFT && Dir != StrafeDir::RIGHT && Dir != StrafeDir::POINT));
		return Yaw;
	}

	double Frame::GetX() const
	{
		assert(YawPresent);
		assert(Strafe && Dir == StrafeDir::POINT);
		return X;
	}

	double Frame::GetY() const
	{
		assert(YawPresent);
		assert(Strafe && Dir == StrafeDir::POINT);
		return Y;
	}

	double Frame::GetPitch() const
	{
		assert(PitchPresent);
		return Pitch;
	}

	void Frame::SetYawPresent(bool value)
	{
		assert(!value || !Strafe || (Dir != StrafeDir::LEFT && Dir != StrafeDir::RIGHT));
		YawPresent = value;
	}

	void Frame::SetYaw(double value)
	{
		assert(!Strafe || (Dir != StrafeDir::LEFT && Dir != StrafeDir::RIGHT && Dir != StrafeDir::POINT));
		YawPresent = true;
		Yaw = value;
	}

	void Frame::SetX(double value)
	{
		assert(Strafe && Dir == StrafeDir::POINT);
		YawPresent = true;
		X = value;
	}

	void Frame::SetY(double value)
	{
		assert(Strafe && Dir == StrafeDir::POINT);
		YawPresent = true;
		Y = value;
	}

	void Frame::SetPitch(double value)
	{
		PitchPresent = true;
		Pitch = value;
	}

	void Frame::SetRepeats(unsigned value)
	{
		assert(value > 0);
		Repeats = value;
	}

	unsigned Frame::GetSharedRNGSeed() const
	{
		assert(SeedsPresent);
		return SharedRNGSeed;
	}

	std::time_t Frame::GetNonSharedRNGSeed() const
	{
		assert(SeedsPresent);
		return NonSharedRNGSeed;
	}

	void Frame::SetSharedRNGSeed(unsigned value)
	{
		SeedsPresent = true;
		SharedRNGSeed = value;
	}

	void Frame::SetNonSharedRNGSeed(std::time_t value)
	{
		SeedsPresent = true;
		NonSharedRNGSeed = value;
	}

	static std::pair<std::string, std::string> SplitProperty(const std::string& line)
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
		boost::unique_lock<boost::shared_mutex> lock(Mutex);
		Properties.clear();
		Frames.clear();
	}

	std::future<ErrorDescription> Input::Open(const std::string& filename)
	{
		Clear();
		return std::async(&Input::OpenInternal, this, filename);
	}

	std::future<ErrorDescription> Input::Save(const std::string& filename, int version)
	{
		return std::async(&Input::SaveInternal, this, filename, version);
	}

	ErrorDescription Input::Error(ErrorCode code)
	{
		return ErrorDescription { code, CurrentLineNumber };
	}

	ErrorDescription Input::OpenInternal(const std::string& filename)
	{
		boost::unique_lock<boost::shared_mutex> lock(Mutex);

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
		std::string line;
		while (std::getline(file, line)) {
			CurrentLineNumber++;
			
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
		bool yawIsRequired = false; // For viewangles checking.
		int strafeDir = -1; // For viewangles checking.
		std::string line;
		while (std::getline(file, line)) {
			CurrentLineNumber++;
			boost::trim_left(line);
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
				Frame f;
				f.Comments = commentString;
				f.SaveName = line.substr(5, std::string::npos);
				Frames.push_back(f);
				commentString.clear();
				continue;
			}
			if (!line.compare(0, 5, "seed ")) {
				if (line.length() < 6)
					throw NOSEED;
				boost::trim_right(line);
				Frame f;
				f.Comments = commentString;
				f.SeedsPresent = true;
				auto s = line.c_str() + 5;
				char *s2;
				f.SharedRNGSeed = std::strtoul(s, &s2, 0);
				if (!(*s2))
					throw NOSEED;
				f.NonSharedRNGSeed = std::strtoll(s2 + 1, nullptr, 0);
				Frames.push_back(f);
				commentString.clear();
				continue;
			}

			Frame f;
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
						if (strafeDir != f.Dir && f.Dir != StrafeDir::LEFT && f.Dir != StrafeDir::RIGHT)
							yawIsRequired = true;
						else
							yawIsRequired = false;
						strafeDir = f.Dir;
					} else if (str[0] != '-' || str[1] != '-' || str[2] != '-')
						throw FAILFRAME;
					if (!f.Strafe) {
						strafeDir = -1;
						yawIsRequired = false;
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
					if (!std::isdigit(str[0]) && str[0] != '-')
						throw FAILFRAME;

					std::move(str.begin(), str.end(), std::back_inserter(f.Frametime));
				}
					break;

				case 4:
				{
					if (l == 0)
						throw FAILFRAME;
					if (!std::isdigit(str[0]) && str[0] != '-')
						throw FAILFRAME;

					if (str == "-") {
						if (yawIsRequired)
							throw NOYAW;
						else
							break;
					} else if (f.Strafe && (f.Dir == StrafeDir::LEFT || f.Dir == StrafeDir::RIGHT))
						throw FAILFRAME;

					f.YawPresent = true;
					auto s = str.c_str();
					if (f.Dir == StrafeDir::POINT) {
						char *s2;
						f.X = std::strtod(s, &s2);
						f.Y = std::strtod(s2 + 1, nullptr);
					} else {
						f.Yaw = std::atof(s);
					}
				}
					break;

				case 5:
				{
					if (l == 0)
						throw FAILFRAME;
					if (!std::isdigit(str[0]) && str[0] != '-')
						throw FAILFRAME;

					if (str == "-")
						break;

					f.PitchPresent = true;
					f.Pitch = std::atof(str.c_str());
				}
					break;

				case 6:
				{
					if (l == 0)
						throw FAILFRAME;
					if (!std::isdigit(str[0]) && str[0] != '-')
						throw FAILFRAME;

					f.Repeats = ReadNumber(str.c_str(), nullptr);
				}
					break;
				}
			}

			if (!f.YawPresent && yawIsRequired)
				throw NOYAW;

			if (f.Repeats == 0)
				f.Repeats = 1;

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

	ErrorDescription Input::SaveInternal(const std::string& filename, int version)
	{
		boost::shared_lock<boost::shared_mutex> lock(Mutex);

		CurrentLineNumber = 1;
		std::ofstream file(filename);
		if (!file)
			return Error(FAILOPEN);

		file << "version " << version << '\n';
		if (file.fail())
			return Error(FAILWRITE);

		for (auto prop : Properties) {
			CurrentLineNumber++;
			file << prop.first;
			if (!prop.second.empty())
				file << ' ' << prop.second;
			file << '\n';
			if (file.fail())
				return Error(FAILWRITE);
		}

		file << "frames\n";
		for (auto frame : Frames) {
			if (!frame.Comments.empty()) {
				std::istringstream s(frame.Comments);
				std::string line;
				while (std::getline(s, line)) {
					CurrentLineNumber++;
					file << "//" << line << '\n';
					if (file.fail())
						return Error(FAILWRITE);
				}
			}
			CurrentLineNumber++;

			if (!frame.SaveName.empty()) {
				file << "save " << frame.SaveName << '\n';
				if (file.fail())
					throw FAILWRITE;
				continue;
			}
			if (frame.SeedsPresent) {
				file << "seed " << frame.SharedRNGSeed << ' ' << frame.NonSharedRNGSeed << '\n';
				if (file.fail())
					throw FAILWRITE;
				continue;
			}

			if (frame.Strafe)
				file << 's' << frame.Type << frame.Dir;
			else
				file << "---";

			if (frame.Lgagst) {
				if (frame.LgagstFullMaxspeed)
					file << 'L';
				else
					file << 'l';
				if (frame.LgagstTimes)
					file << frame.LgagstTimes;
			} else
				file << '-';

			#define WRITE(c, field) \
				if (frame.field) { \
					file << c; \
					if (frame.field##Times) \
						file << frame.field##Times; \
				} else \
					file << '-'; \

			WRITE('j', Autojump)
			WRITE('d', Ducktap)
			WRITE('b', Jumpbug)
			if (frame.Dbc) {
				if (frame.DbcCeilings)
					file << 'C';
				else
					file << 'c';
				if (frame.DbcTimes)
					file << frame.DbcTimes;
			} else
				file << '-';
			WRITE('g', Dbg)
			WRITE('w', Dwj)
			file << '|';

			#undef WRITE
			#define WRITE(c, field) \
				if (frame.field) \
					file << c; \
				else \
					file << '-';

			WRITE('f', Forward)
			WRITE('l', Left)
			WRITE('r', Right)
			WRITE('b', Back)
			WRITE('u', Up)
			WRITE('d', Down)
			file << '|';
			WRITE('j', Jump)
			WRITE('d', Duck)
			WRITE('u', Use)
			WRITE('1', Attack1)
			WRITE('2', Attack2)
			WRITE('r', Reload)
			file << '|';

			#undef WRITE

			file << frame.Frametime << '|';

			if (frame.YawPresent) {
				if (frame.Dir == StrafeDir::POINT)
					file << boost::format("%.10g %.10g") % frame.X % frame.Y;
				else
					file << boost::format("%.10g") % frame.Yaw;
			} else
				file << '-';
			file << '|';

			if (frame.PitchPresent)
				file << boost::format("%.10g") % frame.Pitch;
			else
				file << '-';
			file << '|';

			file << frame.Repeats << '|';
			file << frame.Commands;
			file << '\n';
			if (file.fail())
				throw FAILWRITE;
		}

		return Error(OK);
	}

	int Input::GetVersion() const
	{
		boost::shared_lock<boost::shared_mutex> lock(Mutex);
		return Version;
	}

	const std::unordered_map<std::string, std::string>& Input::GetProperties() const
	{
		boost::shared_lock<boost::shared_mutex> lock(Mutex);
		return Properties;
	}

	const std::vector<Frame>& Input::GetFrames() const
	{
		boost::shared_lock<boost::shared_mutex> lock(Mutex);
		return Frames;
	}

	void Input::SetProperty(const std::string& property, const std::string& value)
	{
		boost::unique_lock<boost::shared_mutex> lock(Mutex);
		Properties[property] = value;
	}

	void Input::RemoveProperty(const std::string& property)
	{
		boost::unique_lock<boost::shared_mutex> lock(Mutex);
		Properties.erase(property);
	}

	void Input::ClearProperties()
	{
		boost::unique_lock<boost::shared_mutex> lock(Mutex);
		Properties.clear();
	}

	void Input::InsertFrame(std::size_t n, const Frame& frame)
	{
		boost::unique_lock<boost::shared_mutex> lock(Mutex);
		Frames.insert(Frames.begin() + n, frame);
	}

	void Input::RemoveFrame(std::size_t n)
	{
		boost::unique_lock<boost::shared_mutex> lock(Mutex);
		Frames.erase(Frames.begin() + n);
	}

	void Input::ClearFrames()
	{
		boost::unique_lock<boost::shared_mutex> lock(Mutex);
		Frames.clear();
	}

	Frame& Input::GetFrame(std::size_t n)
	{
		boost::shared_lock<boost::shared_mutex> lock(Mutex);
		return Frames[n];
	}
}
