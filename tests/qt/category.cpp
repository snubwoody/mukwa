#include "../../src/category.h"
#include <QTest>

class CategoryModelTest : public QObject {
    Q_OBJECT
  private slots:

    void getCategory() {
        auto model = app::CategoryModel{};
        std::vector categories{
            app::Category{
                .id = "C1",
                .title = "Groceries",
            },
            app::Category{
                .id = "C2",
                .title = "Taxes",
            },
        };
        model.loadCategories(categories);
        QVERIFY(model.getCategory("C1").value().title == "Groceries");
        QVERIFY(model.getCategory("C2").value().title == "Taxes");
        QVERIFY(!model.getCategory("does not exist").has_value());
    }
};

QTEST_MAIN(CategoryModelTest);
#include "category.moc"