
# Install 
```bash
cd jqr

cargo install --path .

jqr sample-github.json "[0] | {parents} | [0] | { html_url }"
```


# Building 

```bash
cd jqr

cargo build

./target/release/jqr sample-github.json "[0] | {parents} | [0] | { html_url }"
```
# Sample use 
## No Cmd Args
No argument just pretty prints json contents
```bash
cargo run test1.json
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
cargo run test1.json "[0,2]"
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
cargo run --release sample-github.json "[0] | {author}"
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

### Access single field
```bash
cargo run --release sample-github.json "[0] | { parents } | [0] | {url}"
Cmd=MultiCmd([MultiArrayIndex([0]), KeywordAccess(["parents"]), MultiArrayIndex([0]), KeywordAccess(["url"])])
```
```json
"https://api.github.com/repos/stedolan/jq/commits/9163e09605383a88f6e953d6cb5cc2aebe18c84f"
```

### Keyword Access to nested sub-document using key1.key2

```bash 
cargo run sample-github.json "[0] | { commit.author } "
```
```json
{
  "date": "2020-05-09T01:39:38Z",
  "email": "itchyny@hatena.ne.jp",
  "name": "itchyny"
}
```


### Pipes work
```bash
cat sample-github.json | jqr "[3] | { committer } "
```
```json
{
  "avatar_url": "https://avatars2.githubusercontent.com/u/3422295?v=4",
  "events_url": "https://api.github.com/users/wtlangford/events{/privacy}",
  "followers_url": "https://api.github.com/users/wtlangford/followers",
  "following_url": "https://api.github.com/users/wtlangford/following{/other_user}",
  "gists_url": "https://api.github.com/users/wtlangford/gists{/gist_id}",
  "gravatar_id": "",
  "html_url": "https://github.com/wtlangford",
  "id": 3422295,
  "login": "wtlangford",
  "node_id": "MDQ6VXNlcjM0MjIyOTU=",
  "organizations_url": "https://api.github.com/users/wtlangford/orgs",
  "received_events_url": "https://api.github.com/users/wtlangford/received_events",
  "repos_url": "https://api.github.com/users/wtlangford/repos",
  "site_admin": false,
  "starred_url": "https://api.github.com/users/wtlangford/starred{/owner}{/repo}",
  "subscriptions_url": "https://api.github.com/users/wtlangford/subscriptions",
  "type": "User",
  "url": "https://api.github.com/users/wtlangford"
}
```


400 Mb json file
 cargo run --release ~/Adzerk/Reporting/hoenir/it/data/json/005e9b4c-06de-4ad7-831d-ddd1ed924297 "{Records} | [0] | { Details } | [0]"

and running on large json
 cargo run --release 004ff2c5-7ed0-433b-8638-e6ceeceb1d09-7 "{Records} | [0] | { Details } | [0]"

 {"CTR":0.007064,"Clicks":7,"Date":"2020-04-24T00:00:00Z","DateType":null,"Details":[],"DuplicateIPBucketClicks":0,"DuplicateImpressionBucketClicks":0,"Events":{},"FirstDate":"2020-04-24T00:00:00Z","Grouping":{"AdTypeId":0,"BrandId":0,"CampaignId":548278,"ChannelId":0,"City":null,"CountryCode":null,"CreativeId":0,"Date":0,"DateType":null,"Keyword":null,"MetroCode":0,"OptionId":11091079,"Price":"0","PriorityId":0,"PublisherAccountId":0,"RateTypeId":2,"Region":null,"SiteId":681017,"ZoneId":0},"Impressions":99092,"InvalidUABucketClicks":0,"LastDate":"2020-04-24T00:00:00Z","RawBucketClicks":7,"Revenue":0.0,"SuspiciousBucketClicks":0,"TestBucketClicks":0,"Title":"UK - Sainsbury's Nectar - Feb 2020 - AdylicPortfolio IO-016066 Companion","TrueRevenue":0.0,"UniqueBucketClicks":7,"UniqueCTR":0.007064,"_title":null,"eCPM":0.0}


 cargo run --release 004ff2c5-7ed0-433b-8638-e6ceeceb1d09-7 "{Records} | [0] | { Details } | [0] | { Grouping}"

 Cmd=MultiCmd([KeywordAccess(["Records"]), MultiArrayIndex([0]), KeywordAccess(["Details"]), MultiArrayIndex([0]), KeywordAccess(["Grouping"])])
{
  "AdTypeId": 0,
  "BrandId": 0,
  "CampaignId": 548278,
  "ChannelId": 0,
  "City": null,
  "CountryCode": null,
  "CreativeId": 0,
  "Date": 0,
  "DateType": null,
  "Keyword": null,
  "MetroCode": 0,
  "OptionId": 11091079,
  "Price": "0",
  "PriorityId": 0,
  "PublisherAccountId": 0,
  "RateTypeId": 2,
  "Region": null,
  "SiteId": 681017,
  "ZoneId": 0
}
```