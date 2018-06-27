CREATE TABLE roles (
    id      UUID    PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id INTEGER NOT NULL,
    name    VARCHAR NOT NULL,
    data    JSONB   NOT NULL,

    CONSTRAINT role UNIQUE (user_id, name, data)
);
