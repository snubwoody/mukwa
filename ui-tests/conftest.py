import os
import subprocess
import sys
from collections.abc import Iterator
from pathlib import Path

import pytest
import slint_testing

PROJECT_DIR = Path(__file__).resolve().parent.parent
SCREENSHOT_DIR = PROJECT_DIR / "screenshots"


@pytest.fixture(scope="session")
def app_binary() -> Path:
    """Builds the application with system testing once per session, and returns the binary's path."""
    env = os.environ.copy()
    # The Slint compiler must emit debug info for element ids to be visible to the tests.
    env["SLINT_EMIT_DEBUG_INFO"] = "1"
    subprocess.run(
        ["cargo", "build","-p","mukwa", "--features", "system-testing"],
        cwd=PROJECT_DIR,
        env=env,
        check=True,
    )
    exe = "mukwa.exe" if sys.platform == "win32" else "todo"
    return PROJECT_DIR / "target" / "debug" / exe


@pytest.hookimpl(wrapper=True)
def pytest_runtest_makereport(item: pytest.Item, call: pytest.CallInfo):
    """Records the outcome of each test, for the `window` fixture."""
    report = yield
    setattr(item, f"report_{report.when}", report)
    return report


@pytest.fixture
def app(app_binary: Path) -> Iterator[slint_testing.Application]:
    """Launches a fresh instance of the application for each test, and terminates it afterwards."""
    with slint_testing.Application([str(app_binary)]) as app:
        yield app


@pytest.fixture
def window(
    app: slint_testing.Application, request: pytest.FixtureRequest
) -> Iterator[slint_testing.Window]:
    """Provides the application's window, and saves a screenshot in `screenshots/` if the test fails."""
    window = app.first_window
    assert window is not None
    yield window

    # Keep a screenshot of the window when a test failed, to help with debugging.
    report = getattr(request.node, "report_call", None)
    if report is not None and report.failed:
        SCREENSHOT_DIR.mkdir(exist_ok=True)
        path = SCREENSHOT_DIR / f"{request.node.name}.png"
        try:
            path.write_bytes(window.grab_window_as_png())
        except slint_testing.RequestError as error:
            print(f"Could not grab a screenshot: {error}")
