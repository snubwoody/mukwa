add_rules("mode.debug", "mode.release")
set_languages("c++23")

add_requires("doctest","spdlog 1.17.0")
add_requires("sqlite3 3.53.0+0")
add_requires("sqlitecpp")

target("app_lib")
    add_rules("qt.static")
    add_packages("doctest","spdlog","sqlite3","sqlitecpp")
    add_headerfiles("src/*.h")
    add_frameworks("QtGui","QtQml")
    add_headerfiles("src/*.h")
    add_files("src/*.cpp")
    add_files("src/*.h") -- Add for Qt MOC

target("app")
    add_rules("qt.quickapp")
    add_frameworks("QtGui")
    add_deps("app_lib")
    add_packages("spdlog")
    add_files("src/bin/*.cpp")
    add_files("src/ui/qml.qrc")
    add_defines("DOCTEST_CONFIG_DISABLE") -- Remove testing code
    if is_mode("debug") then
        set_values("windows.subsystem", "console")
    end

target("doctest")
    set_kind("binary")
    add_deps("app_lib")
    add_packages("doctest","sqlitecpp")
    add_files("tests/doctest/*.cpp")
    add_tests("unit")

for _, file in ipairs(os.files("tests/qt/*.cpp")) do
    local name = path.basename(file)
    target("qt_test_"..name)
        set_kind("binary")
        add_rules("qt.console")
        add_packages("sqlitecpp")
        set_default(false)
        add_frameworks("QtTest","QtQml")
        add_deps("app_lib")
        add_files(file,{rules = "qt.moc"})
        add_tests("default")
end