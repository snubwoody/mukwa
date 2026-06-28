#include "transaction.h"

#include <print>
#include <spdlog/spdlog.h>

namespace app {

    QVariant TransactionTableModel::data(const QModelIndex& index, int role) const {
        if (role != Qt::DisplayRole) {
            return QVariant();
        }

        const auto transaction = transactions[index.row()];
        const auto account = accountModel->getAccount(transaction.accountId);

        switch (index.column()) {
            case 0:
                return QString::fromStdString(transaction.date);
            case 1:
                if (account) {
                    return QString::fromStdString(account->name);
                }
                return QVariant();
            case 2:
                return QString("Note");
            case 3:
                if (!transaction.categoryId) {
                    return QVariant();
                }
                if (const auto category =
                        categoryModel->getCategory(transaction.categoryId.value())) {
                    return QString::fromStdString(category.value().title);
                }
                return QVariant();
            case 4:
                return QString("Outflow");
            case 5:
                return QString("Inflow");
            default:
                break;
        }

        return QVariant();
    }

    void TransactionTableModel::loadTransactions(std::span<Transaction> transactions) {
        for (const auto& transaction : transactions) {
            this->transactions.push_back(transaction);
        }
    }

    void TransactionTableModel::setAccount(QString transactionId, QString accountId) {
        for (auto& transaction : transactions) {
            if (transaction.id != transactionId.toStdString()) {
                continue;
            }
            transaction.accountId = accountId.toStdString();
        }
        spdlog::info(
            "Updated transaction ({}) account to {}",
            transactionId.toStdString(),
            accountId.toStdString()
        );
    }

    int TransactionTableModel::rowCount(const QModelIndex& index) const {
        return transactions.size();
    }

    int TransactionTableModel::columnCount(const QModelIndex& index) const {
        return 6;
    }

    QVariant
    TransactionTableModel::headerData(int section, Qt::Orientation orientation, int role) const {
        switch (section) {
            case 0:
                return QString("Date");
            case 1:
                return QString("Account");
            case 2:
                return QString("Note");
            case 3:
                return QString("Category");
            case 4:
                return QString("Outflow");
            case 5:
                return QString("Inflow");
            default:
                break;
        }

        return QVariant();
    }

    Qt::ItemFlags TransactionTableModel::flags(const QModelIndex& index) const {
        Q_UNUSED(index)
        return Qt::ItemIsSelectable | Qt::ItemIsEnabled | Qt::ItemIsEditable;
    }
} // namespace app
