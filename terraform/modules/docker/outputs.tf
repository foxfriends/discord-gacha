output "port" {
  value = docker_container.discord-gacha.ports[0].external
}
