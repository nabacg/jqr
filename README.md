

* Use 

```bash
cargo run test1.json

[{"models":["Fiesta","Focus","Mustang"],"name":"Ford"},{"models":["320","X3","X5"],"name":"BMW"},{"models":["500","Panda"],"name":"Fiat"}]


cargo run test1.json "[0,2]"

{"models":["Fiesta","Focus","Mustang"],"name":"Ford"}
{"models":["500","Panda"],"name":"Fiat"}

```