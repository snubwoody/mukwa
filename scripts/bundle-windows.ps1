# Write-Output "Usage: test.ps1 [-Install] [-Help]"
# Write-Output "Build the installer for Windows.\n"
# Write-Output "Options:"
# Write-Output "  -Architecture, -a Which architecture to build (x86_64 or aarch64)"
# Write-Output "  -Install, -i      Run the installer after building."
# Write-Output "  -Help, -h         Show this help message."

$OSArchitecture = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)
{
    "X64" {
        "x86_64"
    }
    "Arm64" {
        "aarch64"
    }
    default {
        throw "Unsupported architecture"
    }
}

$Architecture = if ($Architecture)
{
    $Architecture
}
else
{
    $OSArchitecture
}

$AppVersion = "0.1.0-alpha.3"
$CargoTarget = "$Architecture-pc-windows-msvc"
$CargoBuildDir = "target/$CargoTarget/release"
$ResourceDir = "$env:TEMP\MukwaBundleDir";

cargo build -r --target $CargoTarget

mkdir $ResourceDir

Copy-Item -Path "${CargoBuildDir}/mukwa.exe" -Destination "$ResourceDir"
Copy-Item -Path "LICENSE" -Destination "$ResourceDir"

iscc resources/Mukwa.iss /DAppVersion=$AppVersion /DResourceDir=$ResourceDir /Obuild /FMukwa-$Architecture-Setup

Remove-Item -Path $ResourceDir -Recurse -Force
