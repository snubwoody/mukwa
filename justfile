set windows-shell := ["powershell","-c"]

add-migration name:
    cargo run --bin cli migrate new {{name}}

migrate:
    cargo run --bin cli migrate up

# Generate a CMakeLists.txt file for IDE support
gen-cmake:
    xmake project -k cmakelists

# Generate a compile_commands.json file for IDE support
gen-compile-commands:
    xmake project -k compile_commands

# Run the app
run:
    xmake run app

# Run all the tests
test:
    xmake test -y

format:
    clang-format ./src/*.cpp ./src/*.h ./src/bin/*.cpp ./tests/qt/*.cpp ./tests/doctest/*.cpp -i

check-format:
    clang-format ./src/*.cpp ./src/*.h ./src/bin/*.cpp ./tests/qt/*.cpp ./tests/doctest/*.cpp --Werror --dry-run

# Run qmllint on all the *.qml files
check-qml-lint:
    qmllint src/ui/*.qml --max-warnings 0
