#include <cassert>
#include <fstream>
#include <future>
#include <iostream>
#include <string>
#include <utility>

#include "hltas.hpp"

namespace HLTAS
{
	static const std::string ErrorMessages[] =
	{
		"Failed to open the file.",
		"Failed to read the version.",
		"This version is not supported.",
		"Failed to read property."
	};

	static auto SplitProperty(const std::string& line)
	{
		auto space = line.find(' ');
		if (space != std::string::npos)
			return std::make_pair(line.substr(0, space), line.substr(space + 1, std::string::npos));
		else
			return std::make_pair(line, std::string());
	}

	void Input::Clear()
	{
		// If we're reading some file, wait for it to finish.
		if (FinishedReading.valid())
			FinishedReading.wait();

		Properties.clear();
		Frames.clear();
	}

	std::shared_future<ErrorDescription> Input::Open(const std::string& filename)
	{
		Clear();

		FinishedReading = std::async(&Input::OpenInternal, this, filename);
		return FinishedReading;
	}

	const std::string& Input::GetErrorMessage(ErrorDescription error)
	{
		assert(error.Code > 0);
		return ErrorMessages[error.Code - 1];
	}

	ErrorDescription Input::Error(ErrorCode code)
	{
		return ErrorDescription { code, CurrentLineNumber };
	}

	ErrorDescription Input::OpenInternal(const std::string& filename)
	{
		CurrentLineNumber = 1;
		std::ifstream file(filename);
		if (!file)
			return Error(FAILOPEN);

		// Read and check the version.
		std::string temp;
		std::getline(file, temp, ' ');
		if (file.fail() || temp != "version")
			return Error(FAILVER);
		std::getline(file, temp);
		if (file.fail() || temp.empty())
			return Error(FAILVER);
		try {
			Version = std::stoi(temp);
		} catch (...) {
			return Error(FAILVER);
		}
		if (Version <= 0)
			return Error(FAILVER);
		if (Version > MAX_SUPPORTED_VERSION)
			return Error(NOTSUPPORTED);

		try {
			ReadProperties(file);
			ReadFrames(file);
		} catch (ErrorCode error) {
			return Error(error);
		}

		return Error(OK);
	}

	void Input::ReadProperties(std::ifstream& file)
	{
		while (file.good()) {
			CurrentLineNumber++;

			std::string line;
			std::getline(file, line);
			if (file.fail())
				throw FAILPROP;
			
			// Empty line.
			if (line.empty())
				continue;
			// Comments.
			if (!line.find("//"))
				continue;

			// Frame data ahead.
			if (line == "frames")
				return;

			auto prop = SplitProperty(line);
			Properties[prop.first] = prop.second;
		}
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
