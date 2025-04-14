variable "name" {
  type = string
}

variable "image_name" {
  type    = string
  default = "ghcr.io/foxfriends/discord-gacha"
}

variable "image_version" {
  type    = string
  default = "main"
}

variable "restart" {
  type    = string
  default = "unless-stopped"
}

variable "assets_dir" {
  type = string
}

variable "products_file" {
  type = string
}

variable "discord_application_id" {
  type = string
}

variable "discord_token" {
  type      = string
  sensitive = true
}

variable "sheets_sheet_id" {
  type = string
}

variable "sheets_client_id" {
  type = string
}

variable "sheets_client_secret" {
  type      = string
  sensitive = true
}

variable "sheets_redirect_uri" {
  type = string
}

variable "sheets_access_token" {
  type      = string
  sensitive = true
}

variable "sheets_refresh_token" {
  type      = string
  sensitive = true
}

variable "shopify_shop" {
  type = string
}

variable "shopify_token" {
  type      = string
  sensitive = true
}

variable "inventory_url" {
  type = string
}

variable "inventory_enabled" {
  type    = bool
  default = true
}

variable "log_level" {
  type    = string
  default = "debug"
}
