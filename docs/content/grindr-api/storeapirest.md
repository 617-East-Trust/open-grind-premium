# StoreApiRest, WIP

> [!NOTE]
> This page is a work in progress. Endpoints below haven't been fully researched.

## Get store products store, WIP

Response type: `StoreResponse` (undocumented).

```
GET /v3/store/googleplay/products/store
```

## Submit consumables attribution token, WIP

Body type: `PaymentTokenRequest` (undocumented).

Response type: `PaymentTokenResponse` (undocumented).

```
POST /v1/consumables/attribution/tokens
```

## Get consumable products (deprecated), WIP

Response type: `ConsumableProductsResponse` (undocumented).

```
GET /v1/store/googleplay/products/consumables
```

## Get explore shop products, WIP

Response type: `StoreInProfileResponse` (undocumented).

```
GET /v1/shop/googleplay/products/explore
```

## Get products by one stop shop category, WIP

Response type: `StoreResponse` (undocumented).

```
GET /v2/store/googleplay/products/{oneStopShopCategory}
```

## Submit store attribution token, WIP

Body type: `PaymentTokenRequest` (undocumented).

Response type: `PaymentTokenResponse` (undocumented).

```
POST /v1/store/attribution/tokens
```

## Get store product by id, WIP

Response type: `Product` (undocumented).

```
GET /v4/store/products/{productId}
```

## Get profile drawer shop products, WIP

Response type: `StoreInProfileResponse` (undocumented).

```
GET /v1/shop/googleplay/products/profile_drawer
```

## Submit google play purchase, WIP

Body type: `PurchaseData` (undocumented).

```
POST /v3.1/store/googleplay/purchases
```

## Restore google play purchases, WIP

```
POST /v3.1/store/googleplay/purchases/restorations
```

Body:

## Submit consumable purchase token, WIP

Body type: `ConsumableTokenRequest` (undocumented).

```
POST /v1/store/googleplay/consumables/purchases
```

## Submit subscription purchase token, WIP

Body type: `SubscriptionTokenRequest` (undocumented).

```
POST /v1/store/googleplay/subscriptions/purchases
```

## Restore subscriptions, WIP

Body type: `SubscriptionRestorationRequest` (undocumented).

Response type: `SubscriptionRestorationResponse` (undocumented).

```
POST /v1/store/googleplay/subscriptions/restorations
```

## Get subscriptions, WIP

Response type: `SubscriptionResponse` (undocumented).

```
GET /v3/me/subscriptions
```

Query (optional):

- `status` — string, optional
- `platform` — string, optional

## Get store products profile drawer, WIP

Response type: `StoreResponse` (undocumented).

```
GET /v1/store/googleplay/products/profile_drawer
```
