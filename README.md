# Transaction Processing System

## System Components

The system consists of the following main components: **Data Provider**, **Service**, **Account**, and **Transactions**.

### 1. Data Provider
- The **Data Provider** is responsible for parsing the input, preparing transactions for execution, and sending them to the **Service**.
- It runs concurrently with the **Service**.
- Once all the data is parsed, the **Data Provider** sends a "Stop" message to the **Service** to signal the end of data input.

### 2. Service
- The **Service** manages account states and is responsible for creating account tasks as needed.
- When a transaction arrives, the **Service**:
  - Checks whether a task for the account already exists. If not, it creates a new task for that account.
  - Sends the transaction to the account for processing.
- The **Service** is also responsible for returning results once the transactions are processed.
- The **Service** is responsible for gracefully stop accounts tasks when "Stop" message arrives.

### 3. Account
- **Account** is responsible for validating and executing transactions.
- Each **Account** has a concurrent task that processes transactions one by one.
- Executed transactions are stored in a `HashMap` by their transaction ID, allowing for efficient handling of further actions related to the same transaction.
- The account only processes a transaction if it is deemed valid.

### 4. Transactions
- **Transactions** hold the transaction data and are responsible for verifying the "ancestor" transactions.
- The system uses `InputTransaction` to handle whitespace and formatting issues in the CSV input file.
- A **Transaction** is built from an `InputTransaction` after the input has been processed.



## Assumptions

- "Withdrawal" transactions are treated as "Deposit" transactions with a negative amount. This means that "Dispute", "Resolve", and "Chargeback" for withdrawals are processed with a negative amount.
- Transactions that fail to parse are ignored.
- Transactions that are parsed but deemed irrelevant are saved to the account data as failed and not processed.
- A transaction is considered irrelevant if the previous state of the transaction does not allow the current action (e.g., "Resolve" after "Deposit" without a preceding "Dispute").
- Duplicated "Deposit" or "Withdrawal" transactions are considered irrelevant.
- Input amounts cannot be negative.
- If an account is locked, no other transactions are applied to it.
- If the system receives only irrelevant transactions for an account, the system stores and outputs the account with default values (zeros).
- A "Dispute" can be initiated on a resolved transaction.

## Correctness

- Unit tests are provided to verify the core logic of the system.
- Integration tests are available that take predefined input in CSV format and check the correctness of account states at the end of processing.
- Additional handling for parsing and service errors should be added.
- files for manual test included in test/manual


## Ways to Improve & Unresolved Issues

- The data provider should be separated from the main logic and moved into a different module.
- Additional unit tests could be written to cover more edge cases.
- Consider avoiding the use of `InputTransaction` for CSV parsing as it's currently being used to handle whitespace.
- A wrapper should be created to manage senders of channels, grouping `Account` tasks with their respective accounts. The wrapper can manage the senders more efficiently.
- several TODO's are left in the code for improvement
- Additional logging should be added for error cases

## Unsafe Code / Security

- No unsafe code is used in this project.
- When an error occurs, the system skips the current transaction but does **not** cause the system to fail.

## Efficiency

- The tool uses `async_csv` to perform asynchronous, buffered reads of the input data.
- A `Service` is responsible for receiving messages from the reader and updating account states.
- Each account has its own task for processing transactions, allowing transactions for different accounts to be processed concurrently.
- The `Service` runs concurrently with the reader and uses an `mpsc` channel. Multiple readers can be attached to provide data to the service. (Note: If account state depends on transaction order, transactions for the same account should come from the same data provider, or a transaction ordering mechanism should be implemented.)
- Failed transactions can be easily removed from the account if needed.

