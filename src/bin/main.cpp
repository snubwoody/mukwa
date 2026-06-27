#include "../category.h"
#include "../transaction.h"
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <ostream>
#include <print>
#include <qqml.h>
#include <string>
#include <vector>

#include "../account.h"

using namespace app;

int main(int argc, char* argv[]) {
    QCoreApplication::setApplicationName("Finance app");
    QCoreApplication::setApplicationVersion("0.1.0");
    QGuiApplication app(argc, argv);

    AccountModel accountModel{};
    CategoryModel categoryModel{};
    TransactionTableModel transactionModel{&categoryModel, &accountModel};

    std::vector transactions{
        Transaction{
            .id = "T1",
            .date = "01/01/2026",
            .accountId = "A1",
            .amount = 200,
        },
        Transaction{
            .id = "T2",
            .date = "01/01/2026",
            .categoryId = "C2",
            .accountId = "A1",
            .amount = 22400,
        },
        Transaction{
            .id = "T3",
            .date = "01/01/2026",
            .categoryId = "C1",
            .accountId = "A2",
            .amount = 2002,
        },
    };
    transactionModel.loadTransactions(transactions);

    std::vector categories{
        Category{.id = "C1", .title = "Groceries"},
        Category{.id = "C2", .title = "Transport"},
        Category{.id = "C3", .title = "Rent"},
        Category{.id = "C4", .title = "Entertainment"},
    };
    categoryModel.loadCategories(categories);

    std::vector accounts{
        Account{.id = "A1", .name = "RBC Credit Card"},
        Account{.id = "A2", .name = "Absa Chequing"},
        Account{.id = "A3", .name = "FNB Savings"},
    };
    accountModel.loadAccounts(accounts);

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("transactionTableModel", &transactionModel);

    engine.rootContext()->setContextProperty("categoryModel", &categoryModel);
    engine.rootContext()->setContextProperty("accountModel", &accountModel);
    engine.load(QUrl(QStringLiteral("qrc:/Main.qml")));

    if (engine.rootObjects().isEmpty()) return -1;

    return app.exec();
}
