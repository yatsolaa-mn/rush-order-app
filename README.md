# Rush Order App (extension-only)

This is an [extension-only Shopify app](https://shopify.dev/docs/apps/build/app-extensions/build-extension-only-app): it ships **app extensions** (here, a **Cart Transform** Shopify Function) and does **not** include a custom app server or an embedded Admin UI by default. For a full hosted app with OAuth routes and Admin embedding, use a template such as the [Remix app template](https://github.com/Shopify/shopify-app-template-remix).

## How it works

1. **Cart Transform function** (`extensions/demo-cart-transform-extension`) is a **Rust** function compiled to **WebAssembly**. It runs on Shopify’s infrastructure when the cart is evaluated (target `cart.transform.run`).
2. **Input** is defined in `extensions/demo-cart-transform-extension/src/run.graphql`: cart lines, the line property `_rush_order`, per-line cost, and the product metafield `custom.rush_order_cost`.
3. **Logic** (in `src/run.rs`): if a line has `_rush_order` set to `"true"` (case-insensitive) and the product has a valid positive percentage in `rush_order_cost`, the function emits a **`lineUpdate`** operation with **`fixedPricePerUnit`** so the unit price becomes  
   `currentUnitPrice × (1 + percentage / 100)`.
4. Lines without rush selection, non-variant merchandise, or missing/invalid metafields are left unchanged.

**Store setup (outside this repo):** merchants must define the product metafield **`custom.rush_order_cost`** (decimal, e.g. `20` for 20%), and the storefront or cart integration must set the **line item property** **`_rush_order`** to `"true"` when the buyer chooses a rush order.

**Plan / API note:** price override via `lineUpdate` is subject to Shopify’s rules for your store plan and environment (see [Cart Transform](https://shopify.dev/docs/api/functions/reference/cart-transform) documentation).

## Activating the function on a shop (Admin GraphQL)

Deploying the extension or running `shopify app dev` **does not** permanently attach the function to a store as the active cart transform. You must **register** it with the **Shopify Admin GraphQL API** using an access token from your app (OAuth install) that includes the **`write_cart_transforms`** scope (see `shopify.app.toml`).

Use the **`cartTransformCreate`** mutation (and related queries/mutations if you need to update or remove it). Example shape:

```graphql
mutation CartTransformRegister($cartTransform: CartTransformInput!) {
  cartTransformCreate(cartTransform: $cartTransform) {
    cartTransform {
      id
      functionId
    }
    userErrors {
      field
      message
    }
  }
}
```

Variables typically include either **`functionId`** (Function GID) or **`functionHandle`** (your extension handle, e.g. `demo-cart-transform-extension`), depending on API version—see current [Admin API mutation docs](https://shopify.dev/docs/api/admin-graphql/latest/mutations/cartTransformCreate).

## Local development

- **Requirements:** [Node.js](https://nodejs.org/), [Shopify Partner account](https://partners.shopify.com/signup), a [development store](https://help.shopify.com/en/partners/dashboard/development-stores#create-a-development-store) (or appropriate test store). For this extension, you also need **Rust** and the **`wasm32-unknown-unknown`** target so `cargo build` succeeds when the CLI builds the function.
- Install dependencies: `npm install`
- Run the app in dev mode: `npm run dev` (runs `shopify app dev`).

The [Shopify CLI](https://shopify.dev/docs/apps/tools/cli) links the project to your Partner app and can update preview URLs on the dev store.

## Original template install (reference)

If you are creating a **new** app from the official template instead of cloning this repo:

```shell
npm init @shopify/app@latest
```

Then follow Shopify’s [create an app](https://shopify.dev/docs/apps/getting-started/create) guide.

## Developer resources

- [Introduction to Shopify apps](https://shopify.dev/docs/apps/getting-started)
- [App extensions](https://shopify.dev/docs/apps/build/app-extensions)
- [Extension only apps](https://shopify.dev/docs/apps/build/app-extensions/build-extension-only-app)
- [Shopify CLI](https://shopify.dev/docs/apps/tools/cli)
- [Cart Transform Function API](https://shopify.dev/docs/api/functions/reference/cart-transform)
