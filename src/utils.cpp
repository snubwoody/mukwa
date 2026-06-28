#include "utils.h"

#include <generator>
#include <print>

namespace app {
    std::string randomString(size_t length) {
        const std::string characters =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        std::string result;
        std::srand(std::time(0));

        for (size_t i = 0; i < length; i++) {
            result += characters[std::rand() % characters.size()];
        }
        return result;
    }

    static std::generator<std::string_view> splitLines(const std::string_view& value) {
        std::string currentLine = "";

        for (int i = 0; i < value.length(); i++) {
            if (value[i] == '\n') {
                const auto line = currentLine;
                currentLine = "";
                co_yield line;
                continue;
            }
            currentLine += value[i];
        }

        if (!currentLine.empty()) {
            co_yield currentLine;
        }
    }

    std::tuple<std::string, std::string> parseMigration(const std::string_view& sql) {
        // TODO: run in a transaction
        bool inUpBlock = false;
        bool inDownBlock = false;
        std::string upBlock;
        std::string downBlock;

        for (auto line : splitLines(sql)) {
            if (line == "--migrate: up") {
                inUpBlock = true;
                inDownBlock = false;
                continue;
            }

            if (line == "--migrate: down") {
                inDownBlock = true;
                inUpBlock = false;
                continue;
            }

            if (inUpBlock) {
                upBlock += line;
                upBlock += "\n";
            }

            if (inDownBlock) {
                downBlock += line;
                downBlock += "\n";
            }
        }

        return std::make_tuple(upBlock, downBlock);
    }

    TempDir::TempDir(std::filesystem::path const& dir)
        : path(std::filesystem::temp_directory_path() / dir) {
        std::filesystem::create_directories(path);
    }

    TempDir::~TempDir() {
        std::filesystem::remove_all(path);
    }
} // namespace app
