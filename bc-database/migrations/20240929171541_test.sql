-- Add migration script here
CREATE TABLE foo(
    id INT NOT NULL,
    bar TEXT NOT NULL
);

CREATE TABLE test(
    id INT NOT NULL,
    foo TEXT NOT NULL,
    bar INT NOT NULL,
    baz BYTEA NOT NULL
);

CREATE TABLE inner_test(
    foo TEXT NOT NULL,
    bar BYTEA NOT NULL,
    baz BOOLEAN NOT NULL
);
