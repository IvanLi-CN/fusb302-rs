# Select the I2C execution model at compile time

The crate defaults to synchronous `embedded-hal` I2C and exposes an equivalent
async API when the optional `async` feature selects `embedded-hal-async`. This
keeps one semantic API surface without a runtime dependency or a permanently
duplicated pair of public driver types.
