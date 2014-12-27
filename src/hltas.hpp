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
		FAILPROP
	};

	struct ErrorDescription {
		ErrorCode Code;
		unsigned LineNumber;
	};

	struct Frame {

	};

	class Input
	{
	public:
		std::shared_future<ErrorDescription> Open(const std::string& filename);
		void Clear();

		int GetVersion();
		std::unordered_map<std::string, std::string>& GetProperties();
		std::vector<Frame>& GetFrames();

		static const std::string& GetErrorMessage(ErrorDescription error);

	protected:
		ErrorDescription Error(ErrorCode code);
		ErrorDescription OpenInternal(const std::string& filename);
		void ReadProperties(std::ifstream& file);
		void ReadFrames(std::ifstream& file);
		std::shared_future<ErrorDescription> FinishedReading;
		unsigned CurrentLineNumber;

		int Version;
		std::unordered_map<std::string, std::string> Properties;
		std::vector<Frame> Frames;
	};
}
