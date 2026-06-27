#include "../src/utils.h"

#include <print>

#include "doctest.h"

TEST_CASE("TempDir") {
    SUBCASE("creates directory in system TEMP") {
        const app::TempDir temp{};
        CHECK(std::filesystem::exists(temp.path));
    }

    SUBCASE("deletes directory when dropped") {
        const auto* temp = new app::TempDir;
        const auto path = temp->path;
        delete temp;
        CHECK_FALSE(std::filesystem::exists(path));
    }
}
