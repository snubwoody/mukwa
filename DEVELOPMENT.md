# Developing Mukwa

In debug mode the application state (logs, database, settings, etc.) are stored in a .mukwa at the root of the project.

User data is stored a Sqlite database, the migrations for this database are stored in the migrations folder. Think
carefully before creating a new migration. Consider it irreversible.

## Building

To build Mukwa you will need:

### Linux