#include <cassert>
#include <cstdlib>
#include <ctime>
#include <string>

#include "hltas.hpp"

extern "C" HLTAS::ErrorDescription hltas_rs_read(void* input, const char* filename);
extern "C" HLTAS::ErrorDescription hltas_rs_write(const void* input, const char* filename);

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
		Properties.clear();
		Frames.clear();
		ErrorMessage.clear();
	}

	ErrorDescription Input::Open(const std::string& filename)
	{
		Clear();

		auto error = hltas_rs_read(this, filename.data());
		Version = 1;
		return error;
	}

	ErrorDescription Input::Save(const std::string& filename)
	{
		return hltas_rs_write(this, filename.data());
	}

	int Input::GetVersion() const
	{
		return Version;
	}

	const std::unordered_map<std::string, std::string>& Input::GetProperties() const
	{
		return Properties;
	}

	const std::vector<Frame>& Input::GetFrames() const
	{
		return Frames;
	}

	const std::string& Input::GetErrorMessage() const
	{
		return ErrorMessage;
	}

	void Input::SetProperty(const std::string& property, const std::string& value)
	{
		Properties[property] = value;
	}

	void Input::RemoveProperty(const std::string& property)
	{
		Properties.erase(property);
	}

	void Input::ClearProperties()
	{
		Properties.clear();
	}

	void Input::PushFrame(const Frame& frame)
	{
		Frames.push_back(frame);
	}

	void Input::InsertFrame(std::size_t n, const Frame& frame)
	{
		Frames.insert(Frames.begin() + n, frame);
	}

	void Input::RemoveFrame(std::size_t n)
	{
		Frames.erase(Frames.begin() + n);
	}

	void Input::ClearFrames()
	{
		Frames.clear();
	}

	Frame& Input::GetFrame(std::size_t n)
	{
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

extern "C" const char* hltas_input_get_property(const void* input, const char* property) {
	const HLTAS::Input* hltas_input = static_cast<const HLTAS::Input*>(input);

	const auto& properties = hltas_input->GetProperties();
	if (properties.find(property) != properties.cend())
		return properties.at(property).data();

	return nullptr;
}

extern "C" int hltas_input_get_frame(const void* input, size_t index, hltas_frame* c_frame) {
	const HLTAS::Input* hltas_input = static_cast<const HLTAS::Input*>(input);

	const auto& frames = hltas_input->GetFrames();
	if (index >= frames.size())
		return 1;

	const auto& frame = frames[index];
	c_frame->Strafe = frame.Strafe;
	c_frame->Lgagst = frame.Lgagst;
	c_frame->Autojump = frame.Autojump;
	c_frame->Ducktap = frame.Ducktap;
	c_frame->Jumpbug = frame.Jumpbug;
	c_frame->Dbc = frame.Dbc;
	c_frame->Dbg = frame.Dbg;
	c_frame->Dwj = frame.Dwj;
	c_frame->Type = frame.Type;
	c_frame->Dir = frame.Dir;
	c_frame->LgagstFullMaxspeed = frame.LgagstFullMaxspeed;
	c_frame->LgagstTimes = frame.LgagstTimes;
	c_frame->AutojumpTimes = frame.AutojumpTimes;
	c_frame->Ducktap0ms = frame.Ducktap0ms;
	c_frame->DucktapTimes = frame.DucktapTimes;
	c_frame->JumpbugTimes = frame.JumpbugTimes;
	c_frame->DbcCeilings = frame.DbcCeilings;
	c_frame->DbcTimes = frame.DbcTimes;
	c_frame->DbgTimes = frame.DbgTimes;
	c_frame->DwjTimes = frame.DwjTimes;
	c_frame->Forward = frame.Forward;
	c_frame->Left = frame.Left;
	c_frame->Right = frame.Right;
	c_frame->Back = frame.Back;
	c_frame->Up = frame.Up;
	c_frame->Down = frame.Down;
	c_frame->Jump = frame.Jump;
	c_frame->Duck = frame.Duck;
	c_frame->Use = frame.Use;
	c_frame->Attack1 = frame.Attack1;
	c_frame->Attack2 = frame.Attack2;
	c_frame->Reload = frame.Reload;
	c_frame->Frametime = frame.Frametime.data();
	c_frame->PitchPresent = frame.PitchPresent;
	c_frame->YawPresent = frame.YawPresent;
	if (frame.Dir == HLTAS::StrafeDir::POINT) {
		c_frame->X = frame.X;
		c_frame->Y = frame.Y;
	} else {
		c_frame->Yaw = frame.Yaw;
	}
	c_frame->Pitch = frame.Pitch;
	c_frame->Repeats = frame.Repeats;
	if (!frame.Commands.empty())
		c_frame->Commands = frame.Commands.data();
	if (!frame.Comments.empty())
		c_frame->Comments = frame.Comments.data();
	if (!frame.SaveName.empty())
		c_frame->SaveName = frame.SaveName.data();
	c_frame->SeedPresent = frame.SeedPresent;
	c_frame->Seed = frame.Seed;
	c_frame->BtnState = frame.BtnState;
	c_frame->Buttons = frame.Buttons;
	c_frame->LgagstMinSpeedPresent = frame.LgagstMinSpeedPresent;
	c_frame->LgagstMinSpeed = frame.LgagstMinSpeed;
	c_frame->ResetFrame = frame.ResetFrame;
	c_frame->ResetNonSharedRNGSeed = frame.ResetNonSharedRNGSeed;

	return 0;
}

extern "C" void hltas_input_set_error_message(void* input, const char* message)
{
	HLTAS::Input* hltas_input = static_cast<HLTAS::Input*>(input);
	hltas_input->ErrorMessage = message;
}
