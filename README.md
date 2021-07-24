# works-server

請求書管理システム

## 構成
- rust 1.53.0
- juniper https://github.com/graphql-rust/juniper
- diesel
- k8s
- firebase auth

## Misoca API

```
https://app.misoca.jp/oauth2/authorize?client_id=jGKRHV2hW_t4kn0w4Ma1Jxo_XkZxUA37rqFPRiYT61k&redirect_uri=https://works-prod.web.app&response_type=code&scope=write

curl --location --request POST 'https://app.misoca.jp/oauth2/token' \
--header 'Content-Type: application/json' \
--data '{
    "client_id": "jGKRHV2hW_t4kn0w4Ma1Jxo_XkZxUA37rqFPRiYT61k",
    "client_secret": "",
    "redirect_uri": "https://works-prod.web.app",
    "grant_type": "authorization_code",
    "code": ""
}'

curl --location --request POST 'https://app.misoca.jp/oauth2/token' \
--header 'Content-Type: application/json' \
--data '{
    "client_id": "jGKRHV2hW_t4kn0w4Ma1Jxo_XkZxUA37rqFPRiYT61k",
    "client_secret": "",
    "redirect_uri": "https://works-prod.web.app",
    "grant_type": "refresh_token",
    "refresh_token": "MGFqzUdlBRWl-WmyfevZcctHiSTkT-SAmlQty4EUBLs"
}'

curl --location --request GET 'https://app.misoca.jp/api/v3/invoices' \
--header 'Content-Type: application/json' \
--header 'Authorization: bearer '
```


