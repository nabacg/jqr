JQR is command line tool for exploring JSON documents, heavily inspired by [jq](https://github.com/stedolan/jq), but written in [Rust](https://www.rust-lang.org). It's very much work in progress! 

# Install
Easiest way to use jqr is to install it as a Rust binary on your OS path, which can be done very easily with `cargo install`. For this and all following steps you'll need rust tools installed, please follow steps detailed on  [official install Rust](https://www.rust-lang.org/tools/install) page.

```bash
cd jqr

cargo install --path .

jqr sample-github.json "[0] | {parents} | [0] | { html_url }"
```


# Sample use 
## No Cmd Args
No argument just pretty prints json contents
```bash
jqr test1.json
```

```json
[
  {
    "models": [
      "Fiesta",
      "Focus",
      "Mustang"
    ],
    "name": "Ford"
  },
  {
    "models": [
      "320",
      "X3",
      "X5"
    ],
    "name": "BMW"
  },
  {
    "models": [
      "500",
      "Panda"
    ],
    "name": "Fiat"
  }
]
```

## Array index access 
ToDo - neex fixing, currently it's aways parsed as MultiCmd and only first index is returned
```bash
jqr test1.json "[0,2]"
```
```json
{
  "models": [
    "Fiesta",
    "Focus",
    "Mustang"
  ],
  "name": "Ford"
}
```

## Multi cmd
mixing array based and keyword access to slice and dice complex json documents

### Json doc 
This document is borrowed from [JQ tutorial](https://stedolan.github.io/jq/tutorial/) and can be access like below 

```bash
curl 'https://api.github.com/repos/stedolan/jq/commits?per_page=5' > sample-github.com
```

```bash
jqr sample-github.json "[0] | {author}"
```
```json 
{
  "avatar_url": "https://avatars2.githubusercontent.com/u/375258?v=4",
  "events_url": "https://api.github.com/users/itchyny/events{/privacy}",
  "followers_url": "https://api.github.com/users/itchyny/followers",
  "following_url": "https://api.github.com/users/itchyny/following{/other_user}",
  "gists_url": "https://api.github.com/users/itchyny/gists{/gist_id}",
  "gravatar_id": "",
  "html_url": "https://github.com/itchyny",
  "id": 375258,
  "login": "itchyny",
  "node_id": "MDQ6VXNlcjM3NTI1OA==",
  "organizations_url": "https://api.github.com/users/itchyny/orgs",
  "received_events_url": "https://api.github.com/users/itchyny/received_events",
  "repos_url": "https://api.github.com/users/itchyny/repos",
  "site_admin": false,
  "starred_url": "https://api.github.com/users/itchyny/starred{/owner}{/repo}",
  "subscriptions_url": "https://api.github.com/users/itchyny/subscriptions",
  "type": "User",
  "url": "https://api.github.com/users/itchyny"
}
```
### List keys or values 
```bash
jqr sample-github.json "[0] | committer | .vals"

```
```json
[
  "avatar_url",
  "events_url",
  "followers_url",
  "following_url",
  "gists_url",
  "gravatar_id",
  "html_url",
  "id",
  "login",
  "node_id",
  "organizations_url",
  "received_events_url",
  "repos_url",
  "site_admin",
  "starred_url",
  "subscriptions_url",
  "type",
  "url"
]
```
```bash
jqr sample-github.json "[0] | committer | .vals"

```
```json
[
  "avatar_url",
  "events_url",
  "followers_url",
  "following_url",
  "gists_url",
  "gravatar_id",
  "html_url",
  "id",
  "login",
  "node_id",
  "organizations_url",
  "received_events_url",
  "repos_url",
  "site_admin",
  "starred_url",
  "subscriptions_url",
  "type",
  "url"
]
```
### Access single field
```bash
jqr --release sample-github.json "[0] | { parents } | [0] | {url}"
Cmd=MultiCmd([MultiArrayIndex([0]), KeywordAccess(["parents"]), MultiArrayIndex([0]), KeywordAccess(["url"])])
```
```json
"https://api.github.com/repos/stedolan/jq/commits/9163e09605383a88f6e953d6cb5cc2aebe18c84f"
```

### Keyword Access to nested sub-document using key1.key2

```bash 
jqr sample-github.json "[0] | { commit.author } "
```
```json
{
  "date": "2020-05-09T01:39:38Z",
  "email": "itchyny@hatena.ne.jp",
  "name": "itchyny"
}
```

```bash
jqr sample-github.json "[0] | author | .keys"
```


```bash
jqr sample-github.json "[0] | { myEventName = author.events_url; author = commit.author.name}"
```

```json
{
  "author": "itchyny",
  "myEventName": "https://api.github.com/users/itchyny/events{/privacy}"
}
```

### linux pipes
```bash
cat large-json-lines-file.json | head -n 1 | jqr "[3] | { committer } "
```
