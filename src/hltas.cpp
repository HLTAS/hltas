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
#include <boost/lexical_cast.hpp>
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
		"The yaw field needs a value on this frame.",
		"Buttons are required.",
		"Cannot have both Autojump and Ducktap enabled on the same frame.",
		"Lgagst requires either Autojump or Ducktap.",
		"Lgagst min speed is required.",
		"You cannot specify the Autojump or Ducktap times if you have Lgagst enabled.",
		"RNG seed is required."
	};

	const std::string& GetErrorMessage(ErrorDescription error)
	{
		assert(error.Code > ErrorCode::OK);
		return ErrorMessages[static_cast<int>(error.Code) - 1];
	}

	void Frame::ResetAutofuncs()
	{
		if (Lgagst && LgagstTimes) {
			Lgagst = false;
			Autojump = false;
			Ducktap = false;
		}
		if (Autojump && AutojumpTimes)
			Autojump = false;
		if (Ducktap && DucktapTimes)
			Ducktap = false;
		if (Jumpbug && JumpbugTimes)
			Jumpbug = false;
		if (Dbc && DbcTimes)
			Dbc = false;
		if (Dbg && DbgTimes)
			Dbg = false;
		if (Dwj && DwjTimes)
			Dwj = false;
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

	void Frame::SetDucktap0ms(bool value)
	{
		Ducktap = true;
		Ducktap0ms = value;
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
		assert(!Strafe || (Dir != StrafeDir::LEFT && Dir != StrafeDir::RIGHT && Dir != StrafeDir::BEST && Dir != StrafeDir::POINT));
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
		assert(!value || !Strafe || (Dir != StrafeDir::LEFT && Dir != StrafeDir::RIGHT && Dir != StrafeDir::BEST));
		YawPresent = value;
	}

	void Frame::SetYaw(double value)
	{
		assert(!Strafe || (Dir != StrafeDir::LEFT && Dir != StrafeDir::RIGHT && Dir != StrafeDir::BEST && Dir != StrafeDir::POINT));
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

	unsigned Frame::GetSeed() const
	{
		assert(SeedPresent);
		return Seed;
	}

	void Frame::SetSeed(unsigned value)
	{
		SeedPresent = true;
		Seed = value;
	}

	const StrafeButtons& Frame::GetButtons() const
	{
		assert(BtnState == ButtonState::SET);
		return Buttons;
	}

	void Frame::SetButtons(const StrafeButtons& buttons)
	{
		BtnState = ButtonState::SET;
		Buttons = buttons;
	}

	float Frame::GetLgagstMinSpeed() const
	{
		assert(LgagstMinSpeedPresent);
		return LgagstMinSpeed;
	}

	void Frame::SetLgagstMinSpeed(float value)
	{
		LgagstMinSpeedPresent = true;
		LgagstMinSpeed = value;
	}

	int64_t Frame::GetResetNonSharedRNGSeed() const
	{
		assert(ResetFrame);
		return ResetNonSharedRNGSeed;
	}

	void Frame::SetResetNonSharedRNGSeed(int64_t value)
	{
		ResetFrame = true;
		ResetNonSharedRNGSeed = value;
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
		std::unique_lock<std::shared_timed_mutex> lock(Mutex);
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
		std::unique_lock<std::shared_timed_mutex> lock(Mutex);

		CurrentLineNumber = 1;
		std::ifstream file(filename);
		if (!file)
			return Error(ErrorCode::FAILOPEN);

		// Read and check the version.
		std::string temp;
		std::getline(file, temp, ' ');
		if (file.fail() || temp != "version")
			return Error(ErrorCode::FAILVER);
		std::getline(file, temp);
		if (file.fail() || temp.empty())
			return Error(ErrorCode::FAILVER);
		try {
			Version = std::stoi(temp);
		} catch (...) {
			return Error(ErrorCode::FAILVER);
		}
		if (Version <= 0)
			return Error(ErrorCode::FAILVER);
		if (Version > MAX_SUPPORTED_VERSION)
			return Error(ErrorCode::NOTSUPPORTED);

		try {
			ReadProperties(file);
			ReadFrames(file);
		} catch (ErrorCode error) {
			return Error(error);
		}

		return Error(ErrorCode::OK);
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
					throw ErrorCode::NOSAVENAME;
				Frame f;
				f.Comments = commentString;
				f.SaveName = line.substr(5, std::string::npos);
				Frames.push_back(f);
				commentString.clear();
				continue;
			}
			if (!line.compare(0, 5, "seed ")) {
				if (line.length() < 6)
					throw ErrorCode::NOSEED;
				boost::trim_right(line);
				Frame f;
				f.Comments = commentString;
				f.SeedPresent = true;
				auto s = line.c_str() + 5;
				f.Seed = std::strtoul(s, nullptr, 0);
				Frames.push_back(f);
				commentString.clear();
				continue;
			}
			if (!line.compare(0, 7, "buttons")) {
				boost::trim_right(line);
				Frame f;
				f.Comments = commentString;
				if (line.length() == 7)
					f.BtnState = ButtonState::CLEAR;
				else if (line.length() == 15) {
					f.BtnState = ButtonState::SET;
					f.Buttons.AirLeft = static_cast<Button>(line[8] - '0');
					f.Buttons.AirRight = static_cast<Button>(line[10] - '0');
					f.Buttons.GroundLeft = static_cast<Button>(line[12] - '0');
					f.Buttons.GroundRight = static_cast<Button>(line[14] - '0');
				} else
					throw ErrorCode::NOBUTTONS;
				Frames.push_back(f);
				commentString.clear();
				continue;
			}
			if (!line.compare(0, 15, "lgagstminspeed ")) {
				if (line.length() < 16)
					throw ErrorCode::NOLGAGSTMINSPEED;
				Frame f;
				f.Comments = commentString;
				f.LgagstMinSpeedPresent = true;
				auto s = line.c_str() + 15;
				f.LgagstMinSpeed = boost::lexical_cast<float>(s);
				Frames.push_back(f);
				commentString.clear();
				continue;
			}
			if (!line.compare(0, 6, "reset ")) {
				if (line.length() < 7)
					throw ErrorCode::NORESETSEED;
				Frame f;
				f.Comments = commentString;
				f.ResetFrame = true;
				auto s = line.c_str() + 6;
				f.ResetNonSharedRNGSeed = boost::lexical_cast<int64_t>(s);
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
						throw ErrorCode::FAILFRAME;

					if (str[0] == 's' && std::isdigit(str[1]) && std::isdigit(str[2])) {
						f.Strafe = true;
						f.Type = static_cast<StrafeType>(str[1] - '0');
						f.Dir = static_cast<StrafeDir>(str[2] - '0');
						if (strafeDir != static_cast<int>(f.Dir) && f.Dir != StrafeDir::LEFT && f.Dir != StrafeDir::RIGHT && f.Dir != StrafeDir::BEST)
							yawIsRequired = true;
						else
							yawIsRequired = false;
						strafeDir =static_cast<int>(f.Dir);
					} else if (str[0] != '-' || str[1] != '-' || str[2] != '-')
						throw ErrorCode::FAILFRAME;
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
						throw ErrorCode::FAILFRAME;

					#define READ(c, field) \
						pos++; \
						if (l <= pos) \
							throw ErrorCode::FAILFRAME; \
						if (str[pos] == c) { \
							f.field = true; \
							f.field##Times = ReadNumber(str.c_str() + pos + 1, &pos); \
						} else if (str[pos] != '-') \
							throw ErrorCode::FAILFRAME;

					READ('j', Autojump)

					pos++;
					if (l <= pos)
						throw ErrorCode::FAILFRAME;
					if (str[pos] == 'd' || str[pos] == 'D') {
						f.Ducktap = true;
						f.Ducktap0ms = (str[pos] == 'D');
						f.DucktapTimes = ReadNumber(str.c_str() + pos + 1, &pos);
					} else if (str[pos] != '-')
						throw ErrorCode::FAILFRAME;

					READ('b', Jumpbug)

					if (f.Autojump && f.Ducktap)
						throw ErrorCode::BOTHAJDT;
					if (f.Lgagst && !(f.Autojump || f.Ducktap))
						throw ErrorCode::NOLGAGSTACTION;
					if (f.Lgagst && (f.AutojumpTimes || f.DucktapTimes))
						throw ErrorCode::LGAGSTACTIONTIMES;

					pos++;
					if (l <= pos)
						throw ErrorCode::FAILFRAME;
					if (str[pos] == 'c' || str[pos] == 'C') {
						f.Dbc = true;
						f.DbcCeilings = (str[pos] == 'C');
						f.DbcTimes = ReadNumber(str.c_str() + pos + 1, &pos);
					} else if (str[pos] != '-')
						throw ErrorCode::FAILFRAME;

					READ('g', Dbg)
					READ('w', Dwj)

					#undef READ
				}
					break;

				case 1:
				{
					if (l != 6)
						throw ErrorCode::FAILFRAME;

					std::size_t pos = 0;
					#define READ(c, field) \
						if (str[pos] == c) \
							f.field = true; \
						else if (str[pos] != '-') \
							throw ErrorCode::FAILFRAME; \
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
						throw ErrorCode::FAILFRAME;

					std::size_t pos = 0;
					#define READ(c, field) \
						if (str[pos] == c) \
							f.field = true; \
						else if (str[pos] != '-') \
							throw ErrorCode::FAILFRAME; \
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
						throw ErrorCode::FAILFRAME;
					if (!std::isdigit(str[0]) && str[0] != '-')
						throw ErrorCode::FAILFRAME;

					std::move(str.begin(), str.end(), std::back_inserter(f.Frametime));
				}
					break;

				case 4:
				{
					if (l == 0)
						throw ErrorCode::FAILFRAME;
					if (!std::isdigit(str[0]) && str[0] != '-')
						throw ErrorCode::FAILFRAME;

					if (str == "-") {
						if (yawIsRequired)
							throw ErrorCode::NOYAW;
						else
							break;
					} else if (f.Strafe && (f.Dir == StrafeDir::LEFT || f.Dir == StrafeDir::RIGHT || f.Dir == StrafeDir::BEST))
						throw ErrorCode::FAILFRAME;

					f.YawPresent = true;
					if (f.Dir == StrafeDir::POINT) {
						auto s = str.c_str();
						char *s2;
						f.X = std::strtod(s, &s2); // TODO: replace this with lexical_cast probably.
						f.Y = std::strtod(s2 + 1, nullptr);
					} else {
						try {
							f.Yaw = boost::lexical_cast<double>(str);
						} catch (const boost::bad_lexical_cast&) {
							throw ErrorCode::FAILFRAME;
						}
					}
				}
					break;

				case 5:
				{
					if (l == 0)
						throw ErrorCode::FAILFRAME;
					if (!std::isdigit(str[0]) && str[0] != '-')
						throw ErrorCode::FAILFRAME;

					if (str == "-")
						break;

					f.PitchPresent = true;
					try {
						f.Pitch = boost::lexical_cast<float>(str);
					} catch (const boost::bad_lexical_cast&) {
						throw ErrorCode::FAILFRAME;
					}
				}
					break;

				case 6:
				{
					if (l == 0)
						throw ErrorCode::FAILFRAME;
					if (!std::isdigit(str[0]) && str[0] != '-')
						throw ErrorCode::FAILFRAME;

					f.Repeats = ReadNumber(str.c_str(), nullptr);
				}
					break;
				}
			}

			if (!f.YawPresent && yawIsRequired)
				throw ErrorCode::NOYAW;

			if (f.Repeats == 0)
				f.Repeats = 1;

			if (fieldCounter > 7) {
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
		std::shared_lock<std::shared_timed_mutex> lock(Mutex);

		CurrentLineNumber = 1;
		std::ofstream file(filename);
		if (!file)
			return Error(ErrorCode::FAILOPEN);

		file << "version " << version << '\n';
		if (file.fail())
			return Error(ErrorCode::FAILWRITE);

		for (auto prop : Properties) {
			CurrentLineNumber++;
			file << prop.first;
			if (!prop.second.empty())
				file << ' ' << prop.second;
			file << '\n';
			if (file.fail())
				return Error(ErrorCode::FAILWRITE);
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
						return Error(ErrorCode::FAILWRITE);
				}
			}
			CurrentLineNumber++;

			if (!frame.SaveName.empty()) {
				file << "save " << frame.SaveName << '\n';
				if (file.fail())
					throw ErrorCode::FAILWRITE;
				continue;
			}
			if (frame.SeedPresent) {
				file << "seed " << frame.Seed << '\n';
				if (file.fail())
					throw ErrorCode::FAILWRITE;
				continue;
			}
			if (frame.BtnState != ButtonState::NOTHING) {
				file << "buttons";
				if (frame.BtnState == ButtonState::SET)
					file << ' ' << static_cast<unsigned>(frame.Buttons.AirLeft)
						<< ' ' << static_cast<unsigned>(frame.Buttons.AirRight)
						<< ' ' << static_cast<unsigned>(frame.Buttons.GroundLeft)
						<< ' ' << static_cast<unsigned>(frame.Buttons.GroundRight);
				file << '\n';
				if (file.fail())
					throw ErrorCode::FAILWRITE;
				continue;
			}
			if (frame.LgagstMinSpeedPresent) {
				file << "lgagstminspeed " << frame.LgagstMinSpeed << '\n';
				if (file.fail())
					throw ErrorCode::FAILWRITE;
				continue;
			}
			if (frame.ResetFrame) {
				file << "reset " << frame.ResetNonSharedRNGSeed << '\n';
				if (file.fail())
					throw ErrorCode::FAILWRITE;
				continue;
			}

			if (frame.Strafe)
				file << 's' << static_cast<unsigned>(frame.Type) << static_cast<unsigned>(frame.Dir);
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

			if (frame.Ducktap) {
				if (frame.Ducktap0ms)
					file << 'D';
				else
					file << 'd';
				if (frame.DucktapTimes)
					file << frame.DucktapTimes;
			} else
				file << '-';

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
				throw ErrorCode::FAILWRITE;
		}

		return Error(ErrorCode::OK);
	}

	int Input::GetVersion() const
	{
		std::shared_lock<std::shared_timed_mutex> lock(Mutex);
		return Version;
	}

	const std::unordered_map<std::string, std::string>& Input::GetProperties() const
	{
		std::shared_lock<std::shared_timed_mutex> lock(Mutex);
		return Properties;
	}

	const std::vector<Frame>& Input::GetFrames() const
	{
		std::shared_lock<std::shared_timed_mutex> lock(Mutex);
		return Frames;
	}

	void Input::SetProperty(const std::string& property, const std::string& value)
	{
		std::unique_lock<std::shared_timed_mutex> lock(Mutex);
		Properties[property] = value;
	}

	void Input::RemoveProperty(const std::string& property)
	{
		std::unique_lock<std::shared_timed_mutex> lock(Mutex);
		Properties.erase(property);
	}

	void Input::ClearProperties()
	{
		std::unique_lock<std::shared_timed_mutex> lock(Mutex);
		Properties.clear();
	}

	void Input::PushFrame(const Frame& frame)
	{
		std::unique_lock<std::shared_timed_mutex> lock(Mutex);
		Frames.push_back(frame);
	}

	void Input::InsertFrame(std::size_t n, const Frame& frame)
	{
		std::unique_lock<std::shared_timed_mutex> lock(Mutex);
		Frames.insert(Frames.begin() + n, frame);
	}

	void Input::RemoveFrame(std::size_t n)
	{
		std::unique_lock<std::shared_timed_mutex> lock(Mutex);
		Frames.erase(Frames.begin() + n);
	}

	void Input::ClearFrames()
	{
		std::unique_lock<std::shared_timed_mutex> lock(Mutex);
		Frames.clear();
	}

	Frame& Input::GetFrame(std::size_t n)
	{
		std::shared_lock<std::shared_timed_mutex> lock(Mutex);
		return Frames[n];
	}
}

extern "C" void hltas_input_set_property(void* input, const char* property, const char* value) {
	HLTAS::Input* hltas_input = static_cast<HLTAS::Input*>(input);
	hltas_input->SetProperty(property, value);
}

extern "C" void hltas_input_push_frame(void* input, const hltas_frame* c_frame) {
	HLTAS::Input* hltas_input = static_cast<HLTAS::Input*>(input);

	HLTAS::Frame frame;
	frame.Strafe = c_frame->Strafe;
	frame.Lgagst = c_frame->Lgagst;
	frame.Autojump = c_frame->Autojump;
	frame.Ducktap = c_frame->Ducktap;
	frame.Jumpbug = c_frame->Jumpbug;
	frame.Dbc = c_frame->Dbc;
	frame.Dbg = c_frame->Dbg;
	frame.Dwj = c_frame->Dwj;
	frame.Type = c_frame->Type;
	frame.Dir = c_frame->Dir;
	frame.LgagstFullMaxspeed = c_frame->LgagstFullMaxspeed;
	frame.LgagstTimes = c_frame->LgagstTimes;
	frame.AutojumpTimes = c_frame->AutojumpTimes;
	frame.Ducktap0ms = c_frame->Ducktap0ms;
	frame.DucktapTimes = c_frame->DucktapTimes;
	frame.JumpbugTimes = c_frame->JumpbugTimes;
	frame.DbcCeilings = c_frame->DbcCeilings;
	frame.DbcTimes = c_frame->DbcTimes;
	frame.DbgTimes = c_frame->DbgTimes;
	frame.DwjTimes = c_frame->DwjTimes;
	frame.Forward = c_frame->Forward;
	frame.Left = c_frame->Left;
	frame.Right = c_frame->Right;
	frame.Back = c_frame->Back;
	frame.Up = c_frame->Up;
	frame.Down = c_frame->Down;
	frame.Jump = c_frame->Jump;
	frame.Duck = c_frame->Duck;
	frame.Use = c_frame->Use;
	frame.Attack1 = c_frame->Attack1;
	frame.Attack2 = c_frame->Attack2;
	frame.Reload = c_frame->Reload;
	if (c_frame->Frametime)
		frame.Frametime = c_frame->Frametime;
	frame.PitchPresent = c_frame->PitchPresent;
	frame.YawPresent = c_frame->YawPresent;
	if (c_frame->Dir == HLTAS::StrafeDir::POINT) {
		frame.X = c_frame->X;
		frame.Y = c_frame->Y;
	} else {
		frame.Yaw = c_frame->Yaw;
	}
	frame.Pitch = c_frame->Pitch;
	frame.Repeats = c_frame->Repeats;
	if (c_frame->Commands)
		frame.Commands = c_frame->Commands;
	if (c_frame->Comments)
		frame.Comments = c_frame->Comments;
	if (c_frame->SaveName)
		frame.SaveName = c_frame->SaveName;
	frame.SeedPresent = c_frame->SeedPresent;
	frame.Seed = c_frame->Seed;
	frame.BtnState = c_frame->BtnState;
	frame.Buttons = c_frame->Buttons;
	frame.LgagstMinSpeedPresent = c_frame->LgagstMinSpeedPresent;
	frame.LgagstMinSpeed = c_frame->LgagstMinSpeed;
	frame.ResetFrame = c_frame->ResetFrame;
	frame.ResetNonSharedRNGSeed = c_frame->ResetNonSharedRNGSeed;

	hltas_input->PushFrame(std::move(frame));
}
