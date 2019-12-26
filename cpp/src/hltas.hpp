#pragma once
#include <ctime>
#include <future>
#include <shared_mutex>
#include <string>
#include <unordered_map>
#include <vector>

extern "C" {
	struct hltas_frame;

	void hltas_input_set_property(void* input, const char* property, const char* value);
	const char* hltas_input_get_property(const void* input, const char* property);
	void hltas_input_push_frame(void* input, const hltas_frame* frame);
	int hltas_input_get_frame(const void* input, size_t index, hltas_frame* frame);
	void hltas_input_set_error_message(void* input, const char* message);
}

namespace HLTAS
{
	const int MAX_SUPPORTED_VERSION = 1;

	enum class ErrorCode {
		OK = 0,
		FAILOPEN,
		FAILVER,
		NOTSUPPORTED,
		FAILLINE,
		NOSAVENAME,
		FAILFRAME,
		FAILWRITE,
		NOSEED,
		NOYAW,
		NOBUTTONS,
		BOTHAJDT,
		NOLGAGSTACTION,
		NOLGAGSTMINSPEED,
		LGAGSTACTIONTIMES,
		NORESETSEED
	};

	struct ErrorDescription {
		ErrorCode Code;
		unsigned LineNumber;
	};

	const std::string& GetErrorMessage(ErrorDescription error);

	enum class StrafeType : unsigned char {
		MAXACCEL = 0,
		MAXANGLE,
		MAXDECCEL,
		CONSTSPEED
	};

	enum class StrafeDir : unsigned char {
		LEFT = 0,
		RIGHT,
		BEST,
		YAW,
		POINT,
		LINE
	};

	enum class ButtonState : unsigned char {
		NOTHING = 0,
		SET,
		CLEAR
	};

	enum class Button : unsigned char {
		FORWARD = 0,
		FORWARD_LEFT,
		LEFT,
		BACK_LEFT,
		BACK,
		BACK_RIGHT,
		RIGHT,
		FORWARD_RIGHT
	};

	struct StrafeButtons {
		StrafeButtons() :
			AirLeft(Button::FORWARD),
			AirRight(Button::FORWARD),
			GroundLeft(Button::FORWARD),
			GroundRight(Button::FORWARD) {}

		Button AirLeft;
		Button AirRight;
		Button GroundLeft;
		Button GroundRight;
	};

	struct Frame {
		// We know what we're doing, so save us from a lot of hassle.
		friend class Input;
		friend void ::hltas_input_push_frame(void* input, const hltas_frame* frame);
		friend int ::hltas_input_get_frame(const void* input, size_t index, hltas_frame* frame);

		Frame() :
			Strafe(false),
			Lgagst(false),
			Autojump(false),
			Ducktap(false),
			Jumpbug(false),
			Dbc(false),
			Dbg(false),
			Dwj(false),
			Type(StrafeType::MAXACCEL),
			Dir(StrafeDir::LEFT),
			LgagstFullMaxspeed(false),
			LgagstTimes(0),
			AutojumpTimes(0),
			Ducktap0ms(false),
			DucktapTimes(0),
			JumpbugTimes(0),
			DbcCeilings(false),
			DbcTimes(0),
			DbgTimes(0),
			DwjTimes(0),
			Forward(false),
			Left(false),
			Right(false),
			Back(false),
			Up(false),
			Down(false),
			Jump(false),
			Duck(false),
			Use(false),
			Attack1(false),
			Attack2(false),
			Reload(false),
			PitchPresent(false),
			YawPresent(false),
			X(0),
			Y(0),
			Pitch(0),
			Repeats(0),
			SeedPresent(0),
			Seed(0),
			BtnState(ButtonState::NOTHING),
			LgagstMinSpeedPresent(false),
			LgagstMinSpeed(0.0f),
			ResetFrame(false),
			ResetNonSharedRNGSeed(0) {};

		// If we have a framebulk with an autofunc with times, we want to reset it after first execution so the times don't get set every time.
		void ResetAutofuncs();

		bool Strafe;
		bool Lgagst;
		bool Autojump;
		bool Ducktap;
		bool Jumpbug;
		bool Dbc;
		bool Dbg;
		bool Dwj;

	protected:
		StrafeType Type;
		StrafeDir Dir;
		bool LgagstFullMaxspeed;
		unsigned LgagstTimes;
		unsigned AutojumpTimes;
		bool Ducktap0ms;
		unsigned DucktapTimes;
		unsigned JumpbugTimes;
		bool DbcCeilings;
		unsigned DbcTimes;
		unsigned DbgTimes;
		unsigned DwjTimes;

	public:
		inline StrafeType GetType() const   { return Type; }
		inline StrafeDir  GetDir() const    { return Dir; }
		inline bool GetLgagstFullMaxspeed() const { return LgagstFullMaxspeed; }
		inline unsigned GetLgagstTimes() const    { return LgagstTimes; }
		inline unsigned GetAutojumpTimes() const  { return AutojumpTimes; }
		inline bool     GetDucktap0ms() const     { return Ducktap0ms; }
		inline unsigned GetDucktapTimes() const   { return DucktapTimes; }
		inline unsigned GetJumpbugTimes() const   { return JumpbugTimes; }
		inline bool     GetDbcCeilings() const    { return DbcCeilings; }
		inline unsigned GetDbcTimes() const { return DbcTimes; }
		inline unsigned GetDbgTimes() const { return DbgTimes; }
		inline unsigned GetDwjTimes() const { return DwjTimes; }
		void SetType(StrafeType value);
		void SetDir(StrafeDir value);
		void SetLgagstFullMaxspeed(bool value);
		void SetLgagstTimes(unsigned value);
		void SetAutojumpTimes(unsigned value);
		void SetDucktap0ms(bool value);
		void SetDucktapTimes(unsigned value);
		void SetJumpbugTimes(unsigned value);
		void SetDbcCeilings(bool value);
		void SetDbcTimes(unsigned value);
		void SetDbgTimes(unsigned value);
		void SetDwjTimes(unsigned value);

		bool Forward;
		bool Left;
		bool Right;
		bool Back;
		bool Up;
		bool Down;

		bool Jump;
		bool Duck;
		bool Use;
		bool Attack1;
		bool Attack2;
		bool Reload;

		std::string Frametime;

		bool PitchPresent;

	protected:
		bool YawPresent;
		union {
			double Yaw;
			struct {
				double X, Y;
			};
		};
		double Pitch;

		unsigned Repeats;

	public:
		inline bool GetYawPresent() const { return YawPresent; }
		double GetYaw() const;
		double GetX() const;
		double GetY() const;
		double GetPitch() const;
		inline unsigned GetRepeats() const { return Repeats; }
		void SetYawPresent(bool value);
		void SetYaw(double value);
		void SetX(double value);
		void SetY(double value);
		void SetPitch(double value);
		void SetRepeats(unsigned value);

		std::string Commands;
		std::string Comments;

		std::string SaveName;

		bool SeedPresent;

	protected:
		unsigned Seed;

	public:
		unsigned GetSeed() const;
		void SetSeed(unsigned value);

		ButtonState BtnState;

	protected:
		StrafeButtons Buttons;

	public:
		const StrafeButtons& GetButtons() const;
		void SetButtons(const StrafeButtons& buttons);

		bool LgagstMinSpeedPresent;

	protected:
		float LgagstMinSpeed;

	public:
		float GetLgagstMinSpeed() const;
		void SetLgagstMinSpeed(float value);

		bool ResetFrame;

	protected:
		int64_t ResetNonSharedRNGSeed;

	public:
		int64_t GetResetNonSharedRNGSeed() const;
		void SetResetNonSharedRNGSeed(int64_t value);
	};

	class Input
	{
		friend void ::hltas_input_set_error_message(void* input, const char* message);

	public:
		std::future<ErrorDescription> Open(const std::string& filename);
		std::future<ErrorDescription> Save(const std::string& filename);
		void Clear();

		int GetVersion() const;
		const std::unordered_map<std::string, std::string>& GetProperties() const;
		const std::vector<Frame>& GetFrames() const;
		const std::string& GetErrorMessage() const;

		void SetProperty(const std::string& property, const std::string& value);
		void RemoveProperty(const std::string& property);
		void ClearProperties();

		void PushFrame(const Frame& frame);
		void InsertFrame(std::size_t n, const Frame& frame);
		void RemoveFrame(std::size_t n);
		void ClearFrames();
		Frame& GetFrame(std::size_t n);

	protected:
		ErrorDescription OpenInternal(const std::string& filename);
		ErrorDescription SaveInternal(const std::string& filename);
		mutable std::shared_timed_mutex Mutex;

		int Version;
		std::unordered_map<std::string, std::string> Properties;
		std::vector<Frame> Frames;
		std::string ErrorMessage;
	};
}

extern "C" {
	struct hltas_frame {
		bool Strafe;
		bool Lgagst;
		bool Autojump;
		bool Ducktap;
		bool Jumpbug;
		bool Dbc;
		bool Dbg;
		bool Dwj;
		HLTAS::StrafeType Type;
		HLTAS::StrafeDir Dir;
		bool LgagstFullMaxspeed;
		uint32_t LgagstTimes;
		uint32_t AutojumpTimes;
		bool Ducktap0ms;
		uint32_t DucktapTimes;
		uint32_t JumpbugTimes;
		bool DbcCeilings;
		uint32_t DbcTimes;
		uint32_t DbgTimes;
		uint32_t DwjTimes;
		bool Forward;
		bool Left;
		bool Right;
		bool Back;
		bool Up;
		bool Down;
		bool Jump;
		bool Duck;
		bool Use;
		bool Attack1;
		bool Attack2;
		bool Reload;
		const char* Frametime;
		bool PitchPresent;
		bool YawPresent;
		double Yaw;
		double X;
		double Y;
		double Pitch;
		uint32_t Repeats;
		const char* Commands;
		const char* Comments;
		const char* SaveName;
		bool SeedPresent;
		uint32_t Seed;
		HLTAS::ButtonState BtnState;
		HLTAS::StrafeButtons Buttons;
		bool LgagstMinSpeedPresent;
		float LgagstMinSpeed;
		bool ResetFrame;
		int64_t ResetNonSharedRNGSeed;
	};
}
