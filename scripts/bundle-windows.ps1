$OSArchitecture = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
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

$Architecture = if ($Architecture) {
    $Architecture
} else {
    $OSArchitecture
}

$AppVersion = "0.1.2"
$CargoTarget = "$Architecture-pc-windows-msvc"
$CargoBuildDir = "target/$CargoTarget/release"
$ResourceDir = "$env:TEMP\MukwaBundleDir";
$MukwaDir = "crates/mukwa"

cargo build -p mukwa -r --target $CargoTarget

mkdir $ResourceDir

Copy-Item -Path "${CargoBuildDir}/mukwa.exe" -Destination "$ResourceDir"
Copy-Item -Path "LICENSE" -Destination "$ResourceDir"
Copy-Item -Path "${MukwaDir}/resources/icons/app-icon.ico" -Destination "$ResourceDir"

iscc ${MukwaDir}/resources/Mukwa.iss /DAppVersion=$AppVersion /DResourceDir=$ResourceDir /Obuild /FMukwa-$Architecture-Setup

Remove-Item -Path $ResourceDir -Recurse -Force
