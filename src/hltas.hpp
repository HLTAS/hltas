#include <future>
#include <string>
#include <unordered_map>
#include <vector>

namespace HLTAS
{
	struct Frame {

	};

	class Input
	{
	public:
		std::shared_future<int> Open(const std::string& filename);
		void Clear();
		std::unordered_map<std::string, std::string>& GetProperties();
		std::vector<Frame>& GetFrames();

		static const std::string& GetErrorDescription(int errorCode);

	protected:
		int OpenInternal(const std::string& filename);
		std::shared_future<int> FinishedReading;

		int version;
		std::unordered_map<std::string, std::string> properties;
		std::vector<Frame> frames;
	};
}
