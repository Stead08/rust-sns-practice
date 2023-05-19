-- Add migration script here
CREATE TABLE IF NOT EXISTS users
(
    id        SERIAL PRIMARY KEY,
    username  VARCHAR UNIQUE NOT NULL,
    email     VARCHAR UNIQUE NOT NULL,
    password  VARCHAR        NOT NULL,
    createdAt TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sessions
(
    id         SERIAL PRIMARY KEY,
    session_id VARCHAR NOT NULL UNIQUE,
    user_id    INT     NOT NULL UNIQUE REFERENCES users (id)
);
CREATE TABLE IF NOT EXISTS follow_relations
(
    id          SERIAL PRIMARY KEY,
    followee_id BIGINT NOT NULL,                                       -- フォローされる側のID
    follower_id BIGINT NOT NULL,                                       -- フォローする側のID
    FOREIGN KEY (followee_id) REFERENCES users (id) ON DELETE CASCADE, -- フォロウィーユーザ削除時にフォロー関係削除
    FOREIGN KEY (follower_id) REFERENCES users (id) ON DELETE CASCADE  -- フォロワーユーザ削除時にフォロー関係削除
);

CREATE UNIQUE INDEX follow_relations__follower_id__followee_id ON follow_relations (follower_id, followee_id);

CREATE TABLE IF NOT EXISTS user_tweets
(
    id      SERIAL PRIMARY KEY ,
    user_id INT NOT NULL REFERENCES users (id), -- ユーザーID
    content VARCHAR(140),                       -- メモ内容
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);