#include <print>
#include <iostream>
#include "doctest.h"
#include <SQLiteCpp/SQLiteCpp.h>

TEST_CASE("AccountService") {
    try
    {
        // Open a database file
        SQLite::Database    db("data.sqlite");

        // Compile a SQL query, containing one parameter (index 1)
        SQLite::Statement   query(db, "SELECT * FROM test WHERE size > ?");

        // Bind the integer value 6 to the first parameter of the SQL query
        query.bind(1, 6);

        // Loop to execute the query step by step, to get rows of result
        while (query.executeStep())
        {
            // Demonstrate how to get some typed column value
            int         id      = query.getColumn(0);
            const char* value   = query.getColumn(1);
            int         size    = query.getColumn(2);

            std::cout << "row: " << id << ", " << value << ", " << size << std::endl;
        }
    }
    catch (std::exception& e)
    {
        std::cout << "exception: " << e.what() << std::endl;
    }
}