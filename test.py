import asyncio
import os
import sqlite3
from typing import List, Dict, Any

from fust_orm import Database, QueryBuilder

DB_FILE = "demo.db"


def setup_database():
    """
    Uses the standard sqlite3 library to create and seed the database.
    This is done to prepare the data for the fust_orm demonstration.
    """
    # Remove old database file if it exists
    if os.path.exists(DB_FILE):
        os.remove(DB_FILE)

    con = sqlite3.connect(DB_FILE)
    cur = con.cursor()

    # Create a table
    cur.execute("""
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER
        )
    """)

    # Insert some data
    users_to_add = [
        (1, "Alice", 30),
        (2, "Bob", 25),
        (3, "Charlie", 30),
        (4, "David", 42),
    ]
    cur.executemany("INSERT INTO users VALUES(?, ?, ?)", users_to_add)

    con.commit()
    con.close()
    print(f"Database '{DB_FILE}' created and seeded successfully.")


async def main():
    """
    Main async function to demonstrate the fust_orm library.
    """
    try:
        # 1. Setup the database with initial data
        setup_database()

        # 2. Asynchronously connect to the database using our Rust library
        print("\nConnecting to the database with fust_orm...")
        db = await Database.connect(DB_FILE)
        print("Connection successful!")

        # 3. Build a query to find all users
        print("\nExecuting query to find all users...")
        all_users_query = QueryBuilder.select(
            table="users", columns=["id", "name", "age"]
        )
        all_users: List[Dict[str, Any]] = await db.execute(all_users_query)

        print("Query results for all users:")
        for user in all_users:
            print(f"  - {user}")

        # 4. Build a more complex query using method chaining to find users who are 30
        print("\nExecuting query to find users with age = 30...")
        users_aged_30_query = QueryBuilder.select(
            table="users", columns=["id", "name"]
        ).where_("age", "30")

        users_aged_30: List[Dict[str, Any]] = await db.execute(users_aged_30_query)

        print("Query results for users aged 30:")
        for user in users_aged_30:
            print(f"  - {user}")

    except Exception as e:
        print(f"\nAn error occurred: {e}")
    finally:
        # 5. Clean up the database file
        if os.path.exists(DB_FILE):
            os.remove(DB_FILE)
            print(f"\nCleaned up '{DB_FILE}'.")


if __name__ == "__main__":
    asyncio.run(main())
