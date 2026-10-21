// IVE's module: the editor and its tools.
module github.com/warsawdynamics/ive

go 1.23.4

toolchain go1.24.1

require (
	github.com/stretchr/testify v1.9.0
	golang.org/x/text v0.21.0 // indirect
)

require example.com/single v0.1.0

replace github.com/old/thing v1.2.3 => ../thing

replace golang.org/x/net => golang.org/x/net v0.33.0

exclude github.com/broken/dep v0.4.0

retract [v1.0.0, v1.0.2]
