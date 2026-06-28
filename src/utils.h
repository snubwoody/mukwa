#pragma once
#include <filesystem>

namespace app {
    /// @brief Generates a random alphanumeric string.
    ///
    /// This function is not cryptographically secure.
    ///
    /// @param length the length of the generated string.
    std::string randomString(size_t length = 10);

    class TempDir {
      public:
        const std::filesystem::path path;

        TempDir(std::filesystem::path const& dir = randomString());
        // Closes the
        ~TempDir();
    };

    /// @brief Parses up and down migrations from a SQL string.
    ///
    /// @returns A tuple containing the up (left) and down (right) migrations.
    std::tuple<std::string, std::string> parseMigration(const std::string_view& sql);
} // namespace app
