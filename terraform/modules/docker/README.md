This module enables deploying the `discord-gacha` app via Docker.

The file `products.toml` must be loaded into the container at `/app/products.toml`,
and then the `/app/assets/` directory mounted containing the images corresponding to
the `sku` field of each product (e.g. `sku=PR123` requires `/app/assets/PR123.png`
to exist).
