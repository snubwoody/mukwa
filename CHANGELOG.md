## (unreleased)

- Windows: Added download progress bar.

## 0.3.1 - 2026-10-22

- Fixed transactions without categories not being shown in analytics.
- Fixed the app crashing when trying to import transactions from a csv file.

## 0.3.0 - 2026-10-20

- Added budget targets. Budget targets help you plan the amount of money you want to spend each month.
- Mukwa now automatically updates itself on Windows.

### Bug fixes

- Fixed the account balance not updating when creating or editing transactions.
- Fixed an issue where the account combobox was not working when creating transactions.
- Fixed the broken combobox in the create account dialog.
- Fixed the sidebar account balances not updating when creating or editing transactions.
- Fixed the buggy text in the payee column when creating a new transaction.
- Fixed a bug on the analytics page where the total spent was not updating when filtering categories.

## 0.2.4 - 2026-10-10

- Fixed an issue where combobox items were not being selected when clicked.

## 0.2.3 - 2026-10-10

- Fixed an issue where budgets were not being created when changing months on the spending page.
- Added checkboxes for selecting transactions.
    - Added button to delete selected transactions.

## 0.2.2 - 2026-10-07

- Fixed incorrect left to budget calculation. Transfers to credit accounts are no longer subtracted from the left to
  budget amount.

## 0.2.1 - 2026-10-04

- Added support for filtering categories on the analytics page.
- Fixed the left to budget amount not updating when editing budgets.

### Windows

- Fixed the title bar not maximising when double-clicked.
- Fixed minimise window button closing the window.

## 0.2.0 - 2026-09-21

- Added support for importing transactions from CSV files.
- Added a custom title bar on Windows.
- Added a "Credit payments" auto category group. Every credit account will get its own category in the "Credit payments"
  group for tracking payments.
- The spending page now displays how much money is available to assign.

## 0.1.2 - 2026-09-04

- Fixed category text field not working
- Added support for deleting accounts

## 0.1.1 - 2026-08-30

- Fixed transfers not showing on the receiving account.
- Fixed category alignment on spending page.
- Overspent budgets now show a red progress bar.
- Fixed incorrect total spent amount on analytics page.
- Improved budget helper text
    - Overspent categories will show "Overspent ... of ...".
    - Fully spent categories will show "Fully spent".
