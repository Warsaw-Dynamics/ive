# Web servers behind one security group
/* Sizes come from var.instance_count */
variable "instance_count" {
  type    = number
  default = 2
}

locals {
  name_prefix = "ive-${terraform.workspace}"
  ports       = [80, 443]
  enabled     = true
}

resource "aws_security_group" "web" {
  name = "${local.name_prefix}-web"

  dynamic "ingress" {
    for_each = local.ports
    content {
      from_port = ingress.value
      to_port   = ingress.value
    }
  }
}

resource "aws_instance" "web" {
  count         = local.enabled ? var.instance_count : 0
  ami           = data.aws_ami.ubuntu.id
  tags          = { Name = format("%s-%d", local.name_prefix, count.index + 1) }
  subnet_ids    = [for s in aws_subnet.private[*] : s.id if s.tags != null]
  user_data     = <<-EOT
    #!/bin/sh
    echo "Hello from %{ if local.enabled }IVE%{ else }nobody%{ endif }"
  EOT
}
