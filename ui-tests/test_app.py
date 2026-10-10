import slint_testing
from slint_testing import AccessibleRole, PointerEventButton


def find(window: slint_testing.Window, id: str) -> slint_testing.Element:
    return window.query_descendants().match_id(f"AppWindow::{id}").find_one()


def todo_items(window: slint_testing.Window) -> list[slint_testing.Element]:
    return window.query_descendants().match_id("AppWindow::todo-checkbox").find_all()


def test_initial_state(window: slint_testing.Window):
    items = todo_items(window)
    assert [item.accessible_label for item in items] == [
        "Install Slint",
        "Write a UI test",
    ]
    assert [item.accessible_checked for item in items] == [True, False]
    assert all(item.accessible_role == AccessibleRole.Checkbox for item in items)

    assert find(window, "remaining-label").accessible_label == "1 item left"
    assert find(window, "new-todo-input").accessible_placeholder_text == (
        "What needs to be done?"
    )


def test_add_button_requires_text(window: slint_testing.Window):
    new_todo_input = find(window, "new-todo-input")
    add_button = find(window, "add-button")

    assert not add_button.accessible_enabled
    new_todo_input.accessible_value = "Buy milk"
    assert add_button.accessible_enabled
    new_todo_input.accessible_value = ""
    assert not add_button.accessible_enabled


def test_add_todo(window: slint_testing.Window):
    new_todo_input = find(window, "new-todo-input")
    new_todo_input.accessible_value = "Buy milk"
    find(window, "add-button").invoke_accessible_default_action()

    # The new item appears at the end of the list and the input is ready for the next one.
    items = todo_items(window)
    assert len(items) == 3
    assert items[-1].accessible_label == "Buy milk"
    assert not items[-1].accessible_checked
    assert new_todo_input.accessible_value == ""
    assert find(window, "remaining-label").accessible_label == "2 items left"


def test_toggle_todo_with_mouse(window: slint_testing.Window):
    remaining_label = find(window, "remaining-label")
    # Look the item up by its label rather than by its position in the list.
    checkboxes = (
        window.query_descendants()
        .match_accessible_role(AccessibleRole.Checkbox)
        .find_all()
    )
    [item] = [item for item in checkboxes if item.accessible_label == "Write a UI test"]

    item.single_click(PointerEventButton.Left)
    assert item.accessible_checked
    assert remaining_label.accessible_label == "0 items left"

    item.single_click(PointerEventButton.Left)
    assert not item.accessible_checked
    assert remaining_label.accessible_label == "1 item left"


def test_clear_completed(window: slint_testing.Window):
    clear_button = find(window, "clear-button")
    assert clear_button.accessible_enabled

    clear_button.invoke_accessible_default_action()
    assert [item.accessible_label for item in todo_items(window)] == ["Write a UI test"]
    assert not clear_button.accessible_enabled

    # A tracking element follows the list as its rows are recreated.
    first_item = (
        window.query_descendants().match_id("AppWindow::todo-checkbox").tracking()
    ).find_first()
    assert first_item is not None
    first_item.invoke_accessible_default_action()
    assert first_item.accessible_checked
    assert clear_button.accessible_enabled

    clear_button.invoke_accessible_default_action()
    assert todo_items(window) == []
    assert not first_item.is_valid
    assert find(window, "remaining-label").accessible_label == "0 items left"
