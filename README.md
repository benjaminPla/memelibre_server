# README

## db

postgres:17

```
CREATE TABLE IF NOT EXISTS memes (
    id SERIAL PRIMARY KEY,
    image_url TEXT NOT NULL
);

CREATE TABLE users (
    id VARCHAR(32) PRIMARY KEY,
    is_admin BOOLEAN DEFAULT FALSE,
    username VARCHAR(32) NOT NULL
);

CREATE TABLE likes (
    meme_id INTEGER NOT NULL,
    user_id VARCHAR(32) NOT NULL,
    PRIMARY KEY (user_id, meme_id)
);

CREATE TABLE saved (
    meme_id INTEGER NOT NULL,
    user_id VARCHAR(32) NOT NULL,
    PRIMARY KEY (user_id, meme_id)
);

CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    meme_id INTEGER NOT NULL,
    user_id VARCHAR(32) NOT NULL,
    content VARCHAR(128) NOT NULL,
    CONSTRAINT fk_comment_meme FOREIGN KEY (meme_id) REFERENCES memes(id) ON DELETE CASCADE,
    CONSTRAINT fk_comment_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_likes_meme_id ON likes(meme_id);

ALTER TABLE memes ADD COLUMN like_count INTEGER DEFAULT 0;

ALTER TABLE memes ALTER COLUMN like_count SET NOT NULL;

ALTER TABLE likes
ADD CONSTRAINT fk_meme FOREIGN KEY (meme_id) REFERENCES memes(id) ON DELETE CASCADE,
ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE users ADD CONSTRAINT unique_username UNIQUE (username);

ALTER TABLE memes
ADD COLUMN created_by VARCHAR(32),
ADD CONSTRAINT fk_created_by
	FOREIGN KEY (created_by)
	REFERENCES users(id)
	ON DELETE SET NULL;

UPDATE memes
SET created_by = <sudo_id>
WHERE created_by IS NULL;

ALTER TABLE memes
ALTER COLUMN created_by SET NOT NULL;

CREATE INDEX idx_saved_memes_user_id ON saved(user_id);
CREATE INDEX idx_saved_memes_meme_id ON saved(meme_id);

ALTER TABLE saved
ADD CONSTRAINT fk_saved_meme FOREIGN KEY (meme_id) REFERENCES memes(id) ON DELETE CASCADE,
ADD CONSTRAINT fk_saved_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
```

## docker postgres

```
docker run --name memelibre-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=memelibre \
  -p 5432:5432 \
  -d postgres
```

## k8

basics

```
kubectl apply -f .
kubectl delete -f .

kubectl rollout restart deployment/memelibre-server
```

switch in between kubectl context

```
kubectl config get-contexts
kubectl config use-context <minikube|do-sao1-memelibre>
```

logs

> since k8 do not merge logs from multiple pods into one single file you can get them all by

```
kubectl logs -l app=memelibre -n default
```

after changes on config or secret

```
kubectl rollout restart deployment memelibre

```

## my ip

```
curl ifconfig.me
curl https://api.ipify.org
```
