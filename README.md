

* Use 

```bash
cargo run test1.json

[{"models":["Fiesta","Focus","Mustang"],"name":"Ford"},{"models":["320","X3","X5"],"name":"BMW"},{"models":["500","Panda"],"name":"Fiat"}]


cargo run test1.json "[0,2]"

{"models":["Fiesta","Focus","Mustang"],"name":"Ford"}
{"models":["500","Panda"],"name":"Fiat"}


Multi cmd

jqr % cargo run --release test1.json "[1,2]|{models}|[0]"


cargo run --release sample-github.json "[0]|{parents}|[0]|{url}"
Cmd=MultiCmd([MultiArrayIndex([0]), KeywordAccess(["parents"]), MultiArrayIndex([0]), KeywordAccess(["url"])])
"https://api.github.com/repos/stedolan/jq/commits/9163e09605383a88f6e953d6cb5cc2aebe18c84f"


cargo run --release sample-github.json "[0]|{parents}"
Cmd=MultiCmd([MultiArrayIndex([0]), KeywordAccess(["parents"])])
[{"html_url":"https://github.com/stedolan/jq/commit/9163e09605383a88f6e953d6cb5cc2aebe18c84f","sha":"9163e09605383a88f6e953d6cb5cc2aebe18c84f","url":"https://api.github.com/repos/stedolan/jq/commits/9163e09605383a88f6e953d6cb5cc2aebe18c84f"}]
```