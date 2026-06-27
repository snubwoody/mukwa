set windows-shell := ["nu","-c"]

# Generate a CMakeLists.txt file for IDE support
gen-cmake:
    xmake project -k cmakelists

# Generate a compile_commands.json file for IDE support
gen-compile-commands:
    xmake project -k compile_commands
# Run the app in dev mode
dev:
    xmake run folio
# Run all the tests
test:
    xmake run test

format:
    clang-format ./src/*.cpp ./src/*.h ./src/bin/*.cpp ./tests/*.cpp -i

check-format:
    clang-format ./src/*.cpp ./src/*.h ./src/bin/*.cpp ./tests/*.cpp --Werror --dry-run
