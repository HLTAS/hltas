#include <future>
#include <string>
#include <unordered_map>
#include <vector>

namespace HLTAS
{
	const int MAX_SUPPORTED_VERSION = 1;

	enum ErrorCode {
		OK = 0,
		FAILOPEN,
		FAILVER,
		NOTSUPPORTED,
		FAILLINE,
		NOSAVENAME,
		FAILFRAME,
		FAILWRITE
	};

	struct ErrorDescription {
		ErrorCode Code;
		unsigned LineNumber;
	};

	const std::string& GetErrorMessage(ErrorDescription error);

	enum StrafeType : unsigned char {
		MAXACCEL = 0,
		MAXANGLE,
		MAXDECCEL,
		CONSTSPEED
	};

	enum StrafeDir : unsigned char {
		LEFT = 0,
		RIGHT,
		YAW,
		POINT,
		LINE
	};

	struct Frame {
		// We know what we're doing, so save us from a lot of hassle.
		friend class Input;

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
		inline StrafeType GetType()         { return Type; }
		inline StrafeDir  GetDir()          { return Dir; }
		inline bool GetLgagstFullMaxspeed() { return LgagstFullMaxspeed; }
		inline unsigned GetLgagstTimes()    { return LgagstTimes; }
		inline unsigned GetAutojumpTimes()  { return AutojumpTimes; }
		inline unsigned GetDucktapTimes()   { return DucktapTimes; }
		inline unsigned GetJumpbugTimes()   { return JumpbugTimes; }
		inline bool     GetDbcCeilings()    { return DbcCeilings; }
		inline unsigned GetDbcTimes()       { return DbcTimes; }
		inline unsigned GetDbgTimes()       { return DbgTimes; }
		inline unsigned GetDwjTimes()       { return DwjTimes; }
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

	public:
		bool GetYawPresent() { return YawPresent; }
		double GetYaw();
		double GetX();
		double GetY();
		double GetPitch();
		void SetYawPresent(bool value);
		void SetYaw(double value);
		void SetX(double value);
		void SetY(double value);
		void SetPitch(double value);

		unsigned Repeats;
		std::string Commands;
		std::string Comments;

		std::string SaveName;
	};

	class Input
	{
	public:
		std::shared_future<ErrorDescription> Open(const std::string& filename);
		std::shared_future<ErrorDescription> Save(const std::string& filename, int version);
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
		std::shared_future<ErrorDescription> FinishedOperation;
		unsigned CurrentLineNumber;

		int Version;
		std::unordered_map<std::string, std::string> Properties;
		std::vector<Frame> Frames;
	};
}
