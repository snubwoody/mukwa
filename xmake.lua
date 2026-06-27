add_rules("mode.debug", "mode.release")
set_languages("c++23")

add_requires("doctest")

target("app_lib")
    add_rules("qt.static")
    add_packages("doctest")
    add_headerfiles("src/*.h")
    add_frameworks("QtGui","QtQml")
    add_headerfiles("src/*.h")
    add_files("src/*.cpp")
    add_files("src/*.h") -- Add for Qt MOC

target("app")
    add_rules("qt.quickapp")
    add_frameworks("QtGui")
    add_deps("app_lib")
    add_files("src/bin/*.cpp")
    add_files("src/ui/qml.qrc")
    add_defines("DOCTEST_CONFIG_DISABLE") -- Remove testing code
    if is_mode("debug") then
        set_values("windows.subsystem", "console")
    end

target("test")
    set_kind("binary")
    add_deps("app_lib")
    add_packages("doctest")
    add_files("tests/*.cpp")
    add_tests("unit")
