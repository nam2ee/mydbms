Pure rust sqlite implementation

Example command:

`cargo run sample.db .dbinfo`

`cargo run sample.db .tables`

`cargo run sample.db "SELECT COUNT(*) FROM apples"`

`cargo run sample.db "SELECT name FROM apples"`

`cargo run sample.db "SELECT name, color FROM apples"`

`cargo run sample.db "SELECT name, color FROM apples WHERE color = 'Yellow'"`