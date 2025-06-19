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

CREATE INDEX idx_likes_meme_id ON likes(meme_id);
ALTER TABLE memes ADD COLUMN like_count INTEGER DEFAULT 0;
ALTER TABLE likes
ADD CONSTRAINT fk_meme FOREIGN KEY (meme_id) REFERENCES memes(id) ON DELETE CASCADE,
ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
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
