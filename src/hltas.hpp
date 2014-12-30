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
		bool Strafe;
		StrafeType Type;
		StrafeDir Dir;
		bool Lgagst;
		bool LgagstFullMaxspeed;
		unsigned LgagstTimes;
		bool Autojump;
		unsigned AutojumpTimes;
		bool Ducktap;
		unsigned DucktapTimes;
		bool Jumpbug;
		unsigned JumpbugTimes;
		bool Dbc;
		bool DbcCeilings;
		unsigned DbcTimes;
		bool Dbg;
		unsigned DbgTimes;
		bool Dwj;
		unsigned DwjTimes;

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

		bool YawPresent;
		union {
			double Yaw;
			struct {
				double X, Y;
			};
		};
		bool PitchPresent;
		double Pitch;

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

		int GetVersion();
		std::unordered_map<std::string, std::string>& GetProperties();
		std::vector<Frame>& GetFrames();

		static const std::string& GetErrorMessage(ErrorDescription error);

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
