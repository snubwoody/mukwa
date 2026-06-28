#include "../../src/utils.h"

#include <print>

#include "doctest.h"
using namespace app;

TEST_CASE("TempDir") {
    SUBCASE("creates directory in system TEMP") {
        const TempDir temp{};
        CHECK(std::filesystem::exists(temp.path));
    }

    SUBCASE("deletes directory when dropped") {
        const auto* temp = new TempDir;
        const auto path = temp->path;
        delete temp;
        CHECK_FALSE(std::filesystem::exists(path));
    }
}

TEST_CASE("Migrator") {
    SUBCASE("parse single up migration") {
        const std::string sql = "--migrate: up\nCREATE TABLE accounts(id INT PRIMARY KEY);";
        auto [up, _] = parseMigration(sql);
        CHECK_EQ(up, "CREATE TABLE accounts(id INT PRIMARY KEY);\n");
    }

    SUBCASE("parse single down migration") {
        const std::string sql = "--migrate: down\nDROP TABLE accounts;";
        auto [_, down] = parseMigration(sql);
        CHECK_EQ(down, "DROP TABLE accounts;\n");
    }
}