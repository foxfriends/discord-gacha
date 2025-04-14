terraform {
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "~> 3.0.2"
    }
  }
}

locals {
  image = "${var.image_name}:${var.image_version}"
}

data "docker_registry_image" "discord-gacha" {
  name = local.image
}

resource "docker_image" "discord-gacha" {
  name          = local.image
  pull_triggers = [data.docker_registry_image.discord-gacha.sha256_digest]
}

resource "docker_container" "discord-gacha" {
  image   = docker_image.discord-gacha.image_id
  name    = var.name
  restart = var.restart

  network_mode = "bridge"

  volumes {
    container_path = "/app/assets"
    host_path      = var.assets_dir
    read_only      = true
  }

  volumes {
    container_path = "/app/products.toml"
    host_path      = var.products_file
    read_only      = true
  }

  env = [
    "DISCORD_APPLICATION_ID=${var.discord_application_id}",
    "DISCORD_TOKEN=${var.discord_token}",
    "SHEETS_SHEET_ID=${var.sheets_sheet_id}",
    "SHEETS_CLIENT_ID=${var.sheets_client_id}",
    "SHEETS_CLIENT_SECRET=${var.sheets_client_secret}",
    "SHEETS_REDIRECT_URI=${var.sheets_redirect_uri}",
    "SHEETS_ACCESS_TOKEN=placeholder",
    "SHEETS_REFRESH_TOKEN=${var.sheets_refresh_token}",
    "SHOPIFY_SHOP=${var.shopify_shop}",
    "SHOPIFY_TOKEN=${var.shopify_token}",
    "INVENTORY_URL=${var.inventory_url}",
    "INVENTORY_ENABLED=${var.inventory_enabled}",
    "RUST_LOG=discord_gacha=${var.log_level}",
  ]
}
