-- Add up migration script here
CREATE TABLE
    IF NOT EXISTS users (
      id CHAR(36) PRIMARY KEY NOT NULL,
    first_name VARCHAR(255) NOT NULL,
   second_name VARCHAR(255) NOT NULL,
   email VARCHAR(255) NOT NULL UNIQUE,
    active int NOT NULL DEFAULT 0,
    password VARCHAR(255) NOT NULL,
    dob VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    deleted_at TIMESTAMP
    );

    