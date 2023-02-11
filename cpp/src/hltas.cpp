#include <cassert>
#include <cstdlib>
#include <ctime>
#include <string>

#include "hltas.hpp"

extern "C" HLTAS::ErrorDescription hltas_rs_read(void* input, const char* filename);
extern "C" HLTAS::ErrorDescription hltas_rs_write(const void* input, const char* filename);
extern "C" HLTAS::ErrorDescription hltas_rs_from_string(void* input, const char* script);
extern "C" HLTAS::ErrorDescription hltas_rs_to_string(const void* input, char* script, unsigned long size);

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
		"RNG seed is required.",
		"Invalid strafing algorithm (only \"yaw\" and \"vectorial\" allowed).",
		"Missing constraints.",
		"Missing tolerance.",
		"Constraints should start with +- (e.g. +-0.5).",
		"Missing from and to yaw parameters.",
		"Missing \"to\"."
	};

	const std::string& GetErrorMessage(ErrorDescription error)
	{
		assert(error.Code > ErrorCode::OK);
		return ErrorMessages[static_cast<int>(error.Code) - 1];
	}

	Frame::Frame(const hltas_frame& c_frame) {
		Strafe = c_frame.Strafe;
		Lgagst = c_frame.Lgagst;
		Autojump = c_frame.Autojump;
		Ducktap = c_frame.Ducktap;
		Jumpbug = c_frame.Jumpbug;
		Dbc = c_frame.Dbc;
		Dbg = c_frame.Dbg;
		Dwj = c_frame.Dwj;
		Type = c_frame.Type;
		Dir = c_frame.Dir;
		LgagstFullMaxspeed = c_frame.LgagstFullMaxspeed;
		LgagstTimes = c_frame.LgagstTimes;
		AutojumpTimes = c_frame.AutojumpTimes;
		Ducktap0ms = c_frame.Ducktap0ms;
		DucktapTimes = c_frame.DucktapTimes;
		JumpbugTimes = c_frame.JumpbugTimes;
		DbcCeilings = c_frame.DbcCeilings;
		DbcTimes = c_frame.DbcTimes;
		DbgTimes = c_frame.DbgTimes;
		DwjTimes = c_frame.DwjTimes;
		Forward = c_frame.Forward;
		Left = c_frame.Left;
		Right = c_frame.Right;
		Back = c_frame.Back;
		Up = c_frame.Up;
		Down = c_frame.Down;
		Jump = c_frame.Jump;
		Duck = c_frame.Duck;
		Use = c_frame.Use;
		Attack1 = c_frame.Attack1;
		Attack1Times = c_frame.Attack1Times;
		Attack2 = c_frame.Attack2;
		Attack2Times = c_frame.Attack2Times;
		Reload = c_frame.Reload;
		if (c_frame.Frametime)
			Frametime = c_frame.Frametime;
		PitchPresent = c_frame.PitchPresent;
		YawPresent = c_frame.YawPresent;
		if (c_frame.Dir == HLTAS::StrafeDir::POINT) {
			X = c_frame.X;
			Y = c_frame.Y;
		} else if (c_frame.Dir == HLTAS::StrafeDir::LEFT_RIGHT || c_frame.Dir == HLTAS::StrafeDir::RIGHT_LEFT) {
			Count = c_frame.Count;
		} else {
			Yaw = c_frame.Yaw;
		}
		Pitch = c_frame.Pitch;
		Repeats = c_frame.Repeats;
		if (c_frame.Commands)
			Commands = c_frame.Commands;
		if (c_frame.Comments)
			Comments = c_frame.Comments;
		if (c_frame.SaveName)
			SaveName = c_frame.SaveName;
		SeedPresent = c_frame.SeedPresent;
		Seed = c_frame.Seed;
		BtnState = c_frame.BtnState;
		Buttons = c_frame.Buttons;
		LgagstMinSpeedPresent = c_frame.LgagstMinSpeedPresent;
		LgagstMinSpeed = c_frame.LgagstMinSpeed;
		ResetFrame = c_frame.ResetFrame;
		ResetNonSharedRNGSeed = c_frame.ResetNonSharedRNGSeed;
		StrafingAlgorithmPresent = c_frame.StrafingAlgorithmPresent;
		Algorithm = c_frame.Algorithm;
		AlgorithmParametersPresent = c_frame.AlgorithmParametersPresent;
		Parameters = c_frame.Parameters;
		ChangePresent = c_frame.ChangePresent;
		Target = c_frame.Target;
		ChangeFinalValue = c_frame.ChangeFinalValue;
		ChangeOver = c_frame.ChangeOver;
		TargetYawOverride = std::vector<float>(c_frame.TargetYawOverride, c_frame.TargetYawOverride + c_frame.TargetYawOverrideCount);
	}

	bool Frame::IsMovement() const {
		return SaveName.empty()
			&& !SeedPresent
			&& BtnState == HLTAS::ButtonState::NOTHING
			&& !LgagstMinSpeedPresent
			&& !ResetFrame
			&& !StrafingAlgorithmPresent
			&& !AlgorithmParametersPresent
			&& !ChangePresent
			&& TargetYawOverride.empty();
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
		if (Attack1 && Attack1Times)
			Attack1 = false;
		if (Attack2 && Attack2Times)
			Attack2 = false;
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

	void Frame::SetAttack1Times(unsigned value)
	{
		Attack1 = true;
		Attack1Times = value;
	}

	void Frame::SetAttack2Times(unsigned value)
	{
		Attack2 = true;
		Attack2Times = value;
	}

	double Frame::GetYaw() const
	{
		assert(HasYaw());
		return Yaw;
	}

	double Frame::GetX() const
	{
		assert(HasXY());
		return X;
	}

	double Frame::GetY() const
	{
		assert(HasXY());
		return Y;
	}

	unsigned Frame::GetCount() const
	{
		assert(HasCount());
		return Count;
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
		assert(!Strafe || (Dir == StrafeDir::YAW || Dir == StrafeDir::LINE));
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

	void Frame::SetCount(unsigned value)
	{
		assert(!Strafe || (Dir == StrafeDir::LEFT_RIGHT || Dir == StrafeDir::RIGHT_LEFT));
		YawPresent = true;
		Count = value;
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

	StrafingAlgorithm Frame::GetAlgorithm() const
	{
		assert(StrafingAlgorithmPresent);
		return Algorithm;
	}

	void Frame::SetAlgorithm(StrafingAlgorithm value)
	{
		StrafingAlgorithmPresent = true;
		Algorithm = value;
	}

	AlgorithmParameters Frame::GetAlgorithmParameters() const
	{
		assert(AlgorithmParametersPresent);
		return Parameters;
	}

	void Frame::SetAlgorithmParameters(AlgorithmParameters value)
	{
		AlgorithmParametersPresent = true;
		Parameters = value;
	}

	ChangeTarget Frame::GetChangeTarget() const
	{
		assert(ChangePresent);
		return Target;
	}

	float Frame::GetChangeFinalValue() const
	{
		assert(ChangePresent);
		return ChangeFinalValue;
	}

	float Frame::GetChangeOver() const
	{
		assert(ChangePresent);
		return ChangeOver;
	}

	void Frame::SetChangeTarget(ChangeTarget value)
	{
		Target = value;
	}

	void Frame::SetChangeFinalValue(float value)
	{
		ChangeFinalValue = value;
	}

	void Frame::SetChangeOver(float value)
	{
		ChangeOver = value;
	}

	bool Frame::IsEqualToMovementFrame(const Frame& rhs) const {
		return IsMovement() && rhs.IsMovement() &&
		       Strafe == rhs.Strafe &&
		       Lgagst == rhs.Lgagst &&
		       Autojump == rhs.Autojump &&
		       Ducktap == rhs.Ducktap &&
		       Jumpbug == rhs.Jumpbug &&
		       Dbc == rhs.Dbc &&
		       Dbg == rhs.Dbg &&
		       Dwj == rhs.Dwj &&
		       Type == rhs.Type &&
		       Dir == rhs.Dir &&
		       LgagstFullMaxspeed == rhs.LgagstFullMaxspeed &&
		       LgagstTimes == rhs.LgagstTimes &&
		       AutojumpTimes == rhs.AutojumpTimes &&
		       Ducktap0ms == rhs.Ducktap0ms &&
		       DucktapTimes == rhs.DucktapTimes &&
		       JumpbugTimes == rhs.JumpbugTimes &&
		       DbcCeilings == rhs.DbcCeilings &&
		       DbcTimes == rhs.DbcTimes &&
		       DbgTimes == rhs.DbgTimes &&
		       DwjTimes == rhs.DwjTimes &&
		       Forward == rhs.Forward &&
		       Left == rhs.Left &&
		       Right == rhs.Right &&
		       Back == rhs.Back &&
		       Up == rhs.Up &&
		       Down == rhs.Down &&
		       Jump == rhs.Jump &&
		       Duck == rhs.Duck &&
		       Use == rhs.Use &&
		       Attack1 == rhs.Attack1 &&
		       Attack2 == rhs.Attack2 &&
			   	Attack1Times == rhs.Attack1Times &&
			   Attack2Times == rhs.Attack2Times &&
		       Reload == rhs.Reload &&
		       Frametime == rhs.Frametime &&
		       PitchPresent == rhs.PitchPresent &&
		       YawPresent == rhs.YawPresent &&
		       Yaw == rhs.Yaw &&
		       X == rhs.X &&
		       Y == rhs.Y &&
			   Count == rhs.Count &&
		       Pitch == rhs.Pitch &&
		       Repeats == rhs.Repeats &&
		       Commands == rhs.Commands &&
		       Comments == rhs.Comments;
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

	ErrorDescription Input::FromString(const char* script)
	{
		Clear();

		auto error = hltas_rs_from_string(this, script);
		Version = 1;
		return error;
	}

	ErrorDescription Input::ToString(char* script, unsigned long size)
	{
		return hltas_rs_to_string(this, script, size);
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

	bool Input::SplitFrame(std::size_t bulk_idx, std::size_t repeat_idx)
	{
		auto bulk = GetFrame(bulk_idx);
		auto len = bulk.GetRepeats();

		if (repeat_idx >= len-1 || repeat_idx == 0)
			return false;

		auto after = bulk;
		bulk.SetRepeats(repeat_idx);
		after.SetRepeats(len - repeat_idx);

		RemoveFrame(bulk_idx);
		InsertFrame(bulk_idx, after);
		InsertFrame(bulk_idx, bulk);

		return true;
	}
}

extern "C" void hltas_input_set_property(void* input, const char* property, const char* value) {
	HLTAS::Input* hltas_input = static_cast<HLTAS::Input*>(input);
	hltas_input->SetProperty(property, value);
}

extern "C" void hltas_input_push_frame(void* input, const hltas_frame* c_frame) {
	HLTAS::Input* hltas_input = static_cast<HLTAS::Input*>(input);

	HLTAS::Frame frame(*c_frame);

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
	c_frame->Attack1Times = frame.Attack1Times;
	c_frame->Attack2Times = frame.Attack2Times;
	c_frame->Reload = frame.Reload;
	c_frame->Frametime = frame.Frametime.data();
	c_frame->PitchPresent = frame.PitchPresent;
	c_frame->YawPresent = frame.YawPresent;
	if (frame.Dir == HLTAS::StrafeDir::POINT) {
		c_frame->X = frame.X;
		c_frame->Y = frame.Y;
	} else if (frame.Dir == HLTAS::StrafeDir::LEFT_RIGHT || frame.Dir == HLTAS::StrafeDir::RIGHT_LEFT) {
		c_frame->Count = frame.Count;
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
	c_frame->StrafingAlgorithmPresent = frame.StrafingAlgorithmPresent;
	c_frame->Algorithm = frame.Algorithm;
	c_frame->AlgorithmParametersPresent = frame.AlgorithmParametersPresent;
	c_frame->Parameters = frame.Parameters;
	c_frame->ChangePresent = frame.ChangePresent;
	c_frame->Target = frame.Target;
	c_frame->ChangeFinalValue = frame.ChangeFinalValue;
	c_frame->ChangeOver = frame.ChangeOver;
	c_frame->TargetYawOverride = frame.TargetYawOverride.data();
	c_frame->TargetYawOverrideCount = frame.TargetYawOverride.size();

	return 0;
}

extern "C" void hltas_input_set_error_message(void* input, const char* message)
{
	HLTAS::Input* hltas_input = static_cast<HLTAS::Input*>(input);
	hltas_input->ErrorMessage = message;
}
