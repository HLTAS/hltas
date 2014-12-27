#include <cassert>
#include <fstream>
#include <future>
#include <iostream>
#include <string>

#include "hltas.hpp"

namespace HLTAS
{
	enum ErrorCode {
		OK = 0,
		FAILOPEN,
		FAILVER,
		NOTSUPPORTED
	};

	static const std::string ErrorDescriptions[] =
	{
		"Failed to open the file.",
		"Failed to read the version.",
		"This version is not supported."
	};

	void Input::Clear()
	{
		// If we're reading some file, wait for it to finish.
		if (FinishedReading.valid())
			FinishedReading.wait();

		Properties.clear();
		Frames.clear();
	}

	std::shared_future<int> Input::Open(const std::string& filename)
	{
		Clear();

		FinishedReading = std::async(&Input::OpenInternal, this, filename);
		return FinishedReading;
	}

	const std::string& Input::GetErrorDescription(int errorCode)
	{
		assert(errorCode > 0);
		return ErrorDescriptions[errorCode - 1];
	}

	int Input::OpenInternal(const std::string& filename)
	{
		std::ifstream file(filename);
		if (!file)
			return ErrorCode::FAILOPEN;

		// Read and check the version.
		std::string temp;
		std::getline(file, temp, ' ');
		if (file.fail() || temp != "version")
			return ErrorCode::FAILVER;
		std::getline(file, temp);
		if (file.fail() || temp.empty())
			return ErrorCode::FAILVER;
		try {
			Version = std::stoi(temp);
		} catch (...) {
			return ErrorCode::FAILVER;
		}
		if (Version <= 0)
			return ErrorCode::FAILVER;
		if (Version > MAX_SUPPORTED_VERSION)
			return ErrorCode::NOTSUPPORTED;

		ReadProperties(file);
		ReadFrames(file);

		return ErrorCode::OK;
	}

	void Input::ReadProperties(std::ifstream& file)
	{

	}

	void Input::ReadFrames(std::ifstream& file)
	{
		
	}

	int Input::GetVersion()
	{
		if (FinishedReading.valid())
			FinishedReading.wait();

		return Version;
	}

	std::unordered_map<std::string, std::string>& Input::GetProperties()
	{
		if (FinishedReading.valid())
			FinishedReading.wait();

		return Properties;
	}

	std::vector<Frame>& Input::GetFrames()
	{
		if (FinishedReading.valid())
			FinishedReading.wait();

		return Frames;
	}
}
