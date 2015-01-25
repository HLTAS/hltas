#pragma once
#include <ctime>
#include <future>
#include <string>
#include <unordered_map>
#include <vector>
#include <boost/thread/shared_mutex.hpp>

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
		NOBUTTONS
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

	struct Frame {
		// We know what we're doing, so save us from a lot of hassle.
		friend class Input;

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
			Buttons(ButtonState::NOTHING),
			AirLeftBtn(Button::FORWARD),
			AirRightBtn(Button::FORWARD),
			GroundLeftBtn(Button::FORWARD),
			GroundRightBtn(Button::FORWARD) {};

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

		ButtonState Buttons;

	protected:
		Button AirLeftBtn,
			AirRightBtn,
			GroundLeftBtn,
			GroundRightBtn;

	public:
		Button GetAirLeftBtn();
		Button GetAirRightBtn();
		Button GetGroundLeftBtn();
		Button GetGroundRightBtn();
		void SetAirLeftBtn(Button value);
		void SetAirRightBtn(Button value);
		void SetGroundLeftBtn(Button value);
		void SetGroundRightBtn(Button value);
	};

	class Input
	{
	public:
		std::future<ErrorDescription> Open(const std::string& filename);
		std::future<ErrorDescription> Save(const std::string& filename, int version = MAX_SUPPORTED_VERSION);
		void Clear();

		int GetVersion() const;
		const std::unordered_map<std::string, std::string>& GetProperties() const;
		const std::vector<Frame>& GetFrames() const;

		void SetProperty(const std::string& property, const std::string& value);
		void RemoveProperty(const std::string& property);
		void ClearProperties();

		void InsertFrame(std::size_t n, const Frame& frame);
		void RemoveFrame(std::size_t n);
		void ClearFrames();
		Frame& GetFrame(std::size_t n);

	protected:
		ErrorDescription Error(ErrorCode code);
		ErrorDescription OpenInternal(const std::string& filename);
		ErrorDescription SaveInternal(const std::string& filename, int version);
		void ReadProperties(std::ifstream& file);
		void ReadFrames(std::ifstream& file);
		mutable boost::shared_mutex Mutex;
		unsigned CurrentLineNumber;

		int Version;
		std::unordered_map<std::string, std::string> Properties;
		std::vector<Frame> Frames;
	};
}
