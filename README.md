# README

## db

postgres:17

```
CREATE TABLE IF NOT EXISTS memes (
    id SERIAL PRIMARY KEY,
    image_url TEXT NOT NULL
);
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

## backblaze

- https://www.backblaze.com/apidocs/

## k8

basics

```
kubectl apply -f .
kubectl delete -f .
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
