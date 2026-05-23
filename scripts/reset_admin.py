#!/usr/bin/env python3
"""
Reset or create admin credentials in the local SQLite database.
Uses the project's compiled 'hgen' Rust tool to hash the password securely with Argon2id.
"""

import os
import sqlite3
import subprocess
import sys
import uuid

def main():
    print("👤 Resetting/creating admin credentials...")
    username = input("Enter admin username [default: admin]: ").strip() or "admin"
    password = input("Enter new password: ").strip()
    
    if not password:
        print("❌ Error: Password cannot be empty.")
        return
        
    if len(password) < 12:
        print("⚠️  Warning: Password is less than 12 characters.")
        print("The server policy (frontend/src/pages/admin/password_change.rs) enforces a 12-character minimum!")
        confirm = input("Continue anyway? (y/N): ").strip().lower()
        if confirm != "y":
            print("Aborted.")
            return

    # Run hgen to get the Argon2id hash
    print("⏳ Compiling and running hgen to hash your password securely...")
    try:
        # Use subprocess to run cargo run --bin hgen, passing password via stdin
        process = subprocess.Popen(
            ["cargo", "run", "--bin", "hgen", "--quiet"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        stdout, stderr = process.communicate(input=password)
        if process.returncode != 0:
            print(f"❌ hgen failed to run: {stderr}")
            return
        
        # Get hash (last line of stdout)
        password_hash = stdout.strip().split("\n")[-1]
        if not password_hash.startswith("$argon2"):
            print(f"❌ Unexpected output from hgen: {stdout}")
            return
    except Exception as e:
        print(f"❌ Failed to execute cargo/hgen: {e}")
        return

    # Write to sqlite.db
    db_path = "sqlite.db"
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Ensure users table exists
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    """)
    
    # Generate new UUID if user doesn't exist
    user_id = str(uuid.uuid4())
    
    cursor.execute("""
        INSERT INTO users (id, username, password_hash)
        VALUES (?, ?, ?)
        ON CONFLICT(username) DO UPDATE SET
            password_hash = excluded.password_hash
    """, (user_id, username, password_hash))
    
    conn.commit()
    conn.close()
    
    print(f"🎉 Success! Admin user '{username}' has been configured with your new password in {db_path}!")

if __name__ == "__main__":
    main()
