-- Your SQL goes here

CREATE TABLE `articles` (
    `id` INTEGER NOT NULL UNIQUE,
    `title` VARCHAR NULL,
    `is_archived` BOOLEAN DEFAULT 0,
    `is_public` BOOLEAN DEFAULT 0,
    `is_starred` BOOLEAN DEFAULT 0,
    `mimetype` VARCHAR DEFAULT `text/html`,
    `language` VARCHAR DEFAULT `en_US`,
    `preview_picture` TEXT NOT NULL,
    `content` LONG_TEXT NOT NULL,
    `published_by` VARCHAR NULL,
    `published_at` TIMESTAMP NULL,
    `reading_time` INTEGER DEFAULT 0,
    `base_url` VARCHAR NULL,
    `url` VARCHAR NULL
)

